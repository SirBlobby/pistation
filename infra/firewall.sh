#!/usr/bin/env bash
set -euo pipefail

SUBNET=""
OPEN_TO_ALL="no"
DRY_RUN="no"

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m==>\033[0m %s\n' "$*" >&2; }
fail() { printf '\033[1;31m==>\033[0m %s\n' "$*" >&2; exit 1; }

usage() {
  cat <<'USAGE'
PiStation host firewall setup

  sudo ./infra/firewall.sh [options]

Opens the ports LiveKit needs. The web and API containers publish their ports
through Docker, which writes its own rules and is not affected by the host
firewall. LiveKit runs on the host network, so it is.

Options
  --subnet <cidr>   Allow only this network, defaults to the LAN behind your default route
  --open-to-all     Allow from any address instead of a single network
  --dry-run         Print the commands without running them
  --help            Show this message
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --subnet) SUBNET="${2:-}"; shift 2 ;;
    --subnet=*) SUBNET="${1#*=}"; shift ;;
    --open-to-all) OPEN_TO_ALL="yes"; shift ;;
    --dry-run) DRY_RUN="yes"; shift ;;
    --help|-h) usage; exit 0 ;;
    *) fail "unknown option: $1" ;;
  esac
done

PORTS_TCP=(7880 7881)
PORTS_UDP=(3478)
RANGE_UDP_START=50000
RANGE_UDP_END=50100

detect_subnet() {
  local interface
  interface="$(ip route show default 2>/dev/null | awk '/default/ {print $5; exit}')"
  [[ -n "$interface" ]] || return 1
  ip -o -f inet route show scope link dev "$interface" 2>/dev/null | awk '{print $1; exit}'
}

run() {
  if [[ "$DRY_RUN" == "yes" ]]; then
    printf '  %s\n' "$*"
  else
    "$@" >/dev/null
  fi
}

if [[ "$OPEN_TO_ALL" == "no" && -z "$SUBNET" ]]; then
  SUBNET="$(detect_subnet || true)"
  [[ -n "$SUBNET" ]] || fail "could not work out your LAN subnet, pass --subnet or --open-to-all"
fi

if [[ "$DRY_RUN" == "no" && $EUID -ne 0 ]]; then
  fail "run this with sudo, or pass --dry-run to see what it would do"
fi

apply_ufw() {
  if [[ "$OPEN_TO_ALL" == "yes" ]]; then
    for port in "${PORTS_TCP[@]}"; do
      run ufw allow "${port}/tcp"
    done
    for port in "${PORTS_UDP[@]}"; do
      run ufw allow "${port}/udp"
    done
    run ufw allow "${RANGE_UDP_START}:${RANGE_UDP_END}/udp"
    return
  fi

  for port in "${PORTS_TCP[@]}"; do
    run ufw allow from "$SUBNET" to any port "$port" proto tcp
  done
  for port in "${PORTS_UDP[@]}"; do
    run ufw allow from "$SUBNET" to any port "$port" proto udp
  done
  run ufw allow from "$SUBNET" to any port "${RANGE_UDP_START}:${RANGE_UDP_END}" proto udp
}

apply_firewalld() {
  if [[ "$OPEN_TO_ALL" == "yes" ]]; then
    for port in "${PORTS_TCP[@]}"; do
      run firewall-cmd --permanent "--add-port=${port}/tcp"
    done
    for port in "${PORTS_UDP[@]}"; do
      run firewall-cmd --permanent "--add-port=${port}/udp"
    done
    run firewall-cmd --permanent "--add-port=${RANGE_UDP_START}-${RANGE_UDP_END}/udp"
  else
    for port in "${PORTS_TCP[@]}"; do
      run firewall-cmd --permanent "--add-rich-rule=rule family=ipv4 source address=${SUBNET} port port=${port} protocol=tcp accept"
    done
    for port in "${PORTS_UDP[@]}"; do
      run firewall-cmd --permanent "--add-rich-rule=rule family=ipv4 source address=${SUBNET} port port=${port} protocol=udp accept"
    done
    run firewall-cmd --permanent "--add-rich-rule=rule family=ipv4 source address=${SUBNET} port port=${RANGE_UDP_START}-${RANGE_UDP_END} protocol=udp accept"
  fi
  run firewall-cmd --reload
}

print_manual() {
  cat <<MANUAL
No supported firewall front end was found. If you are running nftables or
iptables directly, allow these from ${SUBNET:-any address}:

  tcp ${PORTS_TCP[*]}
  udp ${PORTS_UDP[*]}
  udp ${RANGE_UDP_START}-${RANGE_UDP_END}
MANUAL
}

if command -v ufw >/dev/null 2>&1; then
  BACKEND="ufw"
elif command -v firewall-cmd >/dev/null 2>&1; then
  BACKEND="firewalld"
else
  BACKEND="none"
fi

if [[ "$BACKEND" == "ufw" && $EUID -eq 0 ]] && ! ufw status 2>/dev/null | grep -q "^Status: active"; then
  warn "ufw is installed but not active, the rules will apply once you run: ufw enable"
fi

if [[ "$OPEN_TO_ALL" == "yes" ]]; then
  log "opening LiveKit ports to any address"
else
  log "opening LiveKit ports to $SUBNET"
fi

case "$BACKEND" in
  ufw) apply_ufw ;;
  firewalld) apply_firewalld ;;
  none) print_manual; exit 0 ;;
esac

log "done via $BACKEND"
echo
echo "  tcp 7880        LiveKit signalling, the websocket browsers connect to"
echo "  tcp 7881        LiveKit media over TCP, used where UDP is blocked"
echo "  udp 3478        TURN relay"
echo "  udp 50000-50100 LiveKit media, the normal path for audio and video"
echo
