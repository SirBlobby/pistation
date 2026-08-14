#!/usr/bin/env bash
set -euo pipefail

TOKEN="${GITEA_TOKEN:-}"
SERVER="${GITEA_SERVER_URL:-${GITHUB_SERVER_URL:-}}"
REPOSITORY="${GITEA_REPOSITORY:-${GITHUB_REPOSITORY:-}}"

log() { printf '\033[1;34m==>\033[0m %s\n' "$*" >&2; }
fail() { printf '\033[1;31m==>\033[0m %s\n' "$*" >&2; exit 1; }

usage() {
  cat <<'USAGE'
PiStation Gitea release assets

  ./.gitea/scripts/release-assets.sh <tag> <file> [file...]

Finds or creates the Gitea release for <tag>, then attaches each file to it.
An asset already carrying the same name is replaced, so re-running a release
does not leave duplicates behind.

Environment
  GITEA_TOKEN        API token allowed to write releases, required
  GITEA_SERVER_URL   Gitea base URL, defaults to GITHUB_SERVER_URL
  GITEA_REPOSITORY   owner/name, defaults to GITHUB_REPOSITORY
USAGE
}

if [[ $# -lt 2 ]]; then
  usage
  exit 1
fi

TAG="$1"
shift

[[ -n "$TOKEN" ]] || fail "GITEA_TOKEN is not set"
[[ -n "$SERVER" ]] || fail "GITEA_SERVER_URL is not set"
[[ -n "$REPOSITORY" ]] || fail "GITEA_REPOSITORY is not set"

API="${SERVER%/}/api/v1/repos/${REPOSITORY}"

request() {
  local method="$1" path="$2"
  shift 2
  curl -fsS -X "$method" \
    -H "Authorization: token ${TOKEN}" \
    -H "Accept: application/json" \
    "$@" \
    "${API}${path}"
}

first_id() {
  grep -o '"id":[0-9]*' | head -n 1 | cut -d: -f2
}

find_or_create_release() {
  local found
  if found="$(request GET "/releases/tags/${TAG}" 2>/dev/null)"; then
    printf '%s' "$found" | first_id
    return 0
  fi

  log "creating release ${TAG}"
  request POST "/releases" \
    -H "Content-Type: application/json" \
    -d "{\"tag_name\":\"${TAG}\",\"name\":\"${TAG}\"}" | first_id
}

asset_named() {
  local release="$1" name="$2" listing
  listing="$(request GET "/releases/${release}/assets")"
  printf '%s' "$listing" | tr '}' '\n' | grep -F "\"name\":\"${name}\"" | first_id || true
}

RELEASE="$(find_or_create_release)"
[[ -n "$RELEASE" ]] || fail "could not resolve a release for ${TAG}"

for file in "$@"; do
  [[ -f "$file" ]] || fail "no such file: $file"

  name="$(basename "$file")"
  existing="$(asset_named "$RELEASE" "$name")"

  if [[ -n "$existing" ]]; then
    log "replacing $name"
    request DELETE "/releases/${RELEASE}/assets/${existing}" >/dev/null
  else
    log "uploading $name"
  fi

  request POST "/releases/${RELEASE}/assets?name=${name}" \
    -F "attachment=@${file}" >/dev/null
done

log "attached $# file(s) to ${TAG}"
