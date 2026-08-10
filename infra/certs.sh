#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$SCRIPT_DIR/.env"
OUT_DIR="$SCRIPT_DIR/certs"
LEAF_DAYS=825
CA_DAYS=3650
NEW_CA="no"
EXTRA_HOSTS=()

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m==>\033[0m %s\n' "$*" >&2; }
fail() { printf '\033[1;31m==>\033[0m %s\n' "$*" >&2; exit 1; }

usage() {
  cat <<'USAGE'
PiStation self signed certificates

  ./infra/certs.sh [options]

Creates a local certificate authority and a server certificate covering every
address in your .env, so browsers treat the site as secure and allow camera and
screen sharing. Install certs/ca.crt on each device once and every PiStation
address is trusted.

Options
  --env <file>     Read addresses from this file, defaults to infra/.env
  --out <dir>      Write certificates here, defaults to infra/certs
  --host <name>    Cover an extra hostname or IP, repeatable
  --days <n>       How long the server certificate lasts, defaults to 825
  --new-ca         Replace the authority instead of reusing it
  --help           Show this message
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --env) ENV_FILE="${2:-}"; shift 2 ;;
    --env=*) ENV_FILE="${1#*=}"; shift ;;
    --out) OUT_DIR="${2:-}"; shift 2 ;;
    --out=*) OUT_DIR="${1#*=}"; shift ;;
    --host) EXTRA_HOSTS+=("${2:-}"); shift 2 ;;
    --host=*) EXTRA_HOSTS+=("${1#*=}"); shift ;;
    --days) LEAF_DAYS="${2:-}"; shift 2 ;;
    --days=*) LEAF_DAYS="${1#*=}"; shift ;;
    --new-ca) NEW_CA="yes"; shift ;;
    --help|-h) usage; exit 0 ;;
    *) fail "unknown option: $1" ;;
  esac
done

command -v openssl >/dev/null 2>&1 || fail "openssl is not installed"

env_value() {
  [[ -f "$ENV_FILE" ]] || return 0
  sed -n "s/^[[:space:]]*$1[[:space:]]*=[[:space:]]*//p" "$ENV_FILE" |
    tail -n 1 |
    tr -d '"'"'"'\r'
}

url_host() {
  local value="${1#*://}"
  value="${value%%/*}"
  value="${value%%\?*}"
  printf '%s' "${value%%:*}"
}

lan_address() {
  ip -4 route get 1.1.1.1 2>/dev/null | sed -n 's/.*src \([0-9.]*\).*/\1/p' | head -n 1
}

is_ip() {
  [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]
}

HOSTS=(localhost 127.0.0.1)

add_host() {
  local candidate="$1"
  [[ -n "$candidate" ]] || return 0
  local existing
  for existing in "${HOSTS[@]}"; do
    [[ "$existing" == "$candidate" ]] && return 0
  done
  HOSTS+=("$candidate")
}

PRIMARY=""

for key in PUBLIC_WEB_URL PUBLIC_API_URL PUBLIC_LIVEKIT_URL; do
  host="$(url_host "$(env_value "$key")")"
  [[ -n "$host" ]] || continue
  [[ -n "$PRIMARY" || "$host" == "localhost" ]] || PRIMARY="$host"
  add_host "$host"
done

for host in ${EXTRA_HOSTS[@]+"${EXTRA_HOSTS[@]}"}; do
  [[ -n "$PRIMARY" ]] || PRIMARY="$host"
  add_host "$host"
done

add_host "$(lan_address)"

if [[ -z "$PRIMARY" ]]; then
  PRIMARY="$(lan_address)"
  [[ -n "$PRIMARY" ]] || PRIMARY="localhost"
  warn "no LAN address found in $ENV_FILE, falling back to $PRIMARY"
fi

mkdir -p "$OUT_DIR"

CA_KEY="$OUT_DIR/ca.key"
CA_CRT="$OUT_DIR/ca.crt"
LEAF_KEY="$OUT_DIR/pistation.key"
LEAF_CRT="$OUT_DIR/pistation.crt"

if [[ "$NEW_CA" == "yes" || ! -f "$CA_KEY" || ! -f "$CA_CRT" ]]; then
  log "creating certificate authority"
  openssl req -x509 -newkey rsa:4096 -sha256 -nodes \
    -days "$CA_DAYS" \
    -keyout "$CA_KEY" \
    -out "$CA_CRT" \
    -subj "/CN=PiStation Local CA/O=PiStation" \
    -addext "basicConstraints=critical,CA:TRUE,pathlen:0" \
    -addext "keyUsage=critical,keyCertSign,cRLSign" >/dev/null 2>&1
else
  log "reusing certificate authority at $CA_CRT"
fi

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

{
  printf '[req]\ndistinguished_name = dn\nprompt = no\n\n[dn]\nCN = %s\nO = PiStation\n\n' "$PRIMARY"
  printf '[ext]\nbasicConstraints = critical, CA:FALSE\n'
  printf 'keyUsage = critical, digitalSignature, keyEncipherment\n'
  printf 'extendedKeyUsage = serverAuth\nsubjectAltName = @alt\n\n[alt]\n'

  dns_index=0
  ip_index=0
  for host in "${HOSTS[@]}"; do
    if is_ip "$host"; then
      ip_index=$((ip_index + 1))
      printf 'IP.%d = %s\n' "$ip_index" "$host"
    else
      dns_index=$((dns_index + 1))
      printf 'DNS.%d = %s\n' "$dns_index" "$host"
    fi
  done
} >"$WORK_DIR/leaf.cnf"

log "issuing certificate for $PRIMARY"
openssl genrsa -out "$LEAF_KEY" 2048 >/dev/null 2>&1
openssl req -new -key "$LEAF_KEY" -out "$WORK_DIR/leaf.csr" -config "$WORK_DIR/leaf.cnf" >/dev/null 2>&1
openssl x509 -req -sha256 \
  -in "$WORK_DIR/leaf.csr" \
  -CA "$CA_CRT" \
  -CAkey "$CA_KEY" \
  -CAcreateserial \
  -days "$LEAF_DAYS" \
  -extfile "$WORK_DIR/leaf.cnf" \
  -extensions ext \
  -out "$WORK_DIR/leaf.crt" >/dev/null 2>&1

cat "$WORK_DIR/leaf.crt" "$CA_CRT" >"$LEAF_CRT"
chmod 600 "$CA_KEY" "$LEAF_KEY"
chmod 644 "$CA_CRT" "$LEAF_CRT"

WEB_PORT="$(env_value TLS_WEB_PORT)"
API_PORT="$(env_value TLS_API_PORT)"
LIVEKIT_PORT="$(env_value TLS_LIVEKIT_PORT)"
WEB_PORT="${WEB_PORT:-3443}"
API_PORT="${API_PORT:-8443}"
LIVEKIT_PORT="${LIVEKIT_PORT:-7443}"

log "done, valid for $LEAF_DAYS days"
echo
echo "  covers: ${HOSTS[*]}"
echo
echo "Start the stack with TLS:"
echo
echo "  docker compose -f infra/docker-compose.yml --profile tls up -d"
echo
echo "Then point $ENV_FILE at the encrypted ports:"
echo
echo "  PUBLIC_WEB_URL=https://${PRIMARY}:${WEB_PORT}"
echo "  PUBLIC_API_URL=https://${PRIMARY}:${API_PORT}"
echo "  PUBLIC_LIVEKIT_URL=wss://${PRIMARY}:${LIVEKIT_PORT}"
echo "  CORS_ORIGINS=https://${PRIMARY}:${WEB_PORT},http://localhost:3000,http://localhost:5173"
echo
echo "Install $CA_CRT on every device that connects, otherwise the browser"
echo "keeps warning and still refuses the camera."
echo
