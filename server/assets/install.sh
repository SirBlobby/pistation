#!/usr/bin/env bash
set -euo pipefail

SERVER_URL="__PISTATION_SERVER_URL__"
PACKAGE_URL=""
ENROLLMENT_KEY=""
JOIN_URL=""
KIOSK_USER="pistation"
SKIP_REBOOT="no"
FORCED_PROFILE=""
FORCED_SWAP=""

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m==>\033[0m %s\n' "$*" >&2; }
fail() { printf '\033[1;31m==>\033[0m %s\n' "$*" >&2; exit 1; }

usage() {
  cat <<'USAGE'
PiStation kiosk installer

  curl -fsSL <server>/install.sh | sudo bash -s -- --key <enrollment-token>

Options
  --key <token>          Enrollment token from the admin panel, required
  --server <url>         PiStation API base URL, defaults to the host this script came from
  --join-url <host>      Address shown on screen for people joining, defaults to the server host
  --package-url <url>    Override where the kiosk .deb is downloaded from
  --user <name>          System user to run the kiosk as, defaults to pistation
  --profile <name>       Force a video profile: zero2, pi4, pi5 or generic
  --swap <mb>            Swap size in MB, defaults to the profile's value
  --skip-reboot          Install and enable, but do not reboot at the end
  --help                 Show this message
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --key) ENROLLMENT_KEY="${2:-}"; shift 2 ;;
    --key=*) ENROLLMENT_KEY="${1#*=}"; shift ;;
    --server) SERVER_URL="${2:-}"; shift 2 ;;
    --server=*) SERVER_URL="${1#*=}"; shift ;;
    --join-url) JOIN_URL="${2:-}"; shift 2 ;;
    --join-url=*) JOIN_URL="${1#*=}"; shift ;;
    --package-url) PACKAGE_URL="${2:-}"; shift 2 ;;
    --package-url=*) PACKAGE_URL="${1#*=}"; shift ;;
    --user) KIOSK_USER="${2:-}"; shift 2 ;;
    --user=*) KIOSK_USER="${1#*=}"; shift ;;
    --profile) FORCED_PROFILE="${2:-}"; shift 2 ;;
    --profile=*) FORCED_PROFILE="${1#*=}"; shift ;;
    --swap) FORCED_SWAP="${2:-}"; shift 2 ;;
    --swap=*) FORCED_SWAP="${1#*=}"; shift ;;
    --skip-reboot) SKIP_REBOOT="yes"; shift ;;
    --help|-h) usage; exit 0 ;;
    *) fail "unknown option: $1" ;;
  esac
done

[[ $EUID -eq 0 ]] || fail "run this with sudo, for example: curl -fsSL $SERVER_URL/install.sh | sudo bash -s -- --key ..."
[[ -n "$ENROLLMENT_KEY" ]] || { usage; fail "missing --key"; }
[[ -n "$SERVER_URL" ]] || fail "missing --server"

# The media stack ships prebuilt for arm64 and x86_64 only. A 32 bit Raspberry Pi OS will
# never work, and failing here is far kinder than failing halfway through a build.
ARCH="$(uname -m)"
case "$ARCH" in
  aarch64 | arm64 | x86_64) ;;
  armv7l | armv6l)
    fail "this is a 32 bit system ($ARCH). PiStation needs the 64 bit Raspberry Pi OS."
    ;;
  *) warn "unrecognised architecture $ARCH, continuing anyway" ;;
esac

# Model decides how hard the kiosk may push video. A Zero 2W has four slow cores and half a
# gigabyte of memory, so it gets very different settings from a Pi 5.
MODEL="$(tr -d '\0' </proc/device-tree/model 2>/dev/null || echo unknown)"

case "$FORCED_PROFILE" in
  zero2) MODEL="Raspberry Pi Zero 2 W" ;;
  pi4) MODEL="Raspberry Pi 4 Model B" ;;
  pi5) MODEL="Raspberry Pi 5" ;;
  generic) MODEL="generic" ;;
  "") ;;
  *) fail "unknown profile: $FORCED_PROFILE" ;;
esac

case "$MODEL" in
  *"Zero 2"*)
    PROFILE="Pi Zero 2 W"
    VIDEO_MAX_WIDTH=960
    VIDEO_FPS=12
    VIDEO_QUALITY=70
    GPU_MEM=128
    SWAP_MB=2048
    ;;
  *"Pi 5"*)
    PROFILE="Pi 5"
    VIDEO_MAX_WIDTH=1920
    VIDEO_FPS=60
    VIDEO_QUALITY=85
    GPU_MEM=""
    SWAP_MB=512
    ;;
  *"Pi 4"*|*"Compute Module 4"*)
    PROFILE="Pi 4"
    VIDEO_MAX_WIDTH=1600
    VIDEO_FPS=30
    VIDEO_QUALITY=80
    GPU_MEM=128
    SWAP_MB=1024
    ;;
  *)
    PROFILE="generic"
    VIDEO_MAX_WIDTH=1280
    VIDEO_FPS=30
    VIDEO_QUALITY=80
    GPU_MEM=128
    SWAP_MB=1024
    ;;
esac

[[ -n "$FORCED_SWAP" ]] && SWAP_MB="$FORCED_SWAP"

SERVER_URL="${SERVER_URL%/}"
[[ -n "$PACKAGE_URL" ]] || PACKAGE_URL="$SERVER_URL/downloads/pistation-kiosk.deb"
[[ -n "$JOIN_URL" ]] || JOIN_URL="$(printf '%s' "$SERVER_URL" | sed -e 's|^https\?://||' -e 's|:.*$||')"

if [[ -d /boot/firmware ]]; then
  BOOT_DIR="/boot/firmware"
elif [[ -d /boot ]]; then
  BOOT_DIR="/boot"
else
  fail "could not find the boot partition"
fi

log "installing runtime packages"
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y --no-install-recommends \
  ca-certificates \
  curl \
  cage \
  seatd \
  libwebkit2gtk-4.1-0 \
  libgtk-3-0 \
  gstreamer1.0-plugins-base \
  gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad \
  gstreamer1.0-libav \
  gstreamer1.0-nice \
  libnice10 \
  libgles2

log "downloading the kiosk package from $PACKAGE_URL"
PACKAGE_FILE="$(mktemp /tmp/pistation-kiosk.XXXXXX.deb)"
trap 'rm -f "$PACKAGE_FILE"' EXIT

if ! curl -fsSL "$PACKAGE_URL" -o "$PACKAGE_FILE"; then
  fail "could not download the kiosk package. Upload a build in the admin panel, or pass --package-url"
fi

[[ -s "$PACKAGE_FILE" ]] || fail "the downloaded package was empty"

log "installing the kiosk application"
apt-get install -y "$PACKAGE_FILE"

# Raspberry Pi OS ships with 100 MB of swap. WebKit plus a media stack will exhaust that on
# a 512 MB Zero 2W and the kernel will start killing processes, which looks like the kiosk
# randomly restarting.
configure_swap() {
  local wanted="$1"
  local current_mb=0

  if [[ -f /proc/meminfo ]]; then
    current_mb=$(($(awk '/SwapTotal/ {print $2}' /proc/meminfo 2>/dev/null || echo 0) / 1024))
  fi

  if [[ "$current_mb" -ge "$wanted" ]]; then
    log "swap is already ${current_mb} MB, leaving it alone"
    return
  fi

  if [[ -f /etc/dphys-swapfile ]]; then
    log "raising swap from ${current_mb} MB to ${wanted} MB"
    sed -i "/^CONF_SWAPSIZE=/d;/^CONF_MAXSWAP=/d" /etc/dphys-swapfile
    {
      echo "CONF_SWAPSIZE=$wanted"
      echo "CONF_MAXSWAP=$wanted"
    } >>/etc/dphys-swapfile

    dphys-swapfile swapoff >/dev/null 2>&1 || true
    dphys-swapfile setup >/dev/null 2>&1 || warn "could not resize the swap file"
    dphys-swapfile swapon >/dev/null 2>&1 || warn "could not enable swap"
    return
  fi

  if [[ -f /swapfile ]]; then
    log "a swap file already exists, leaving it alone"
    return
  fi

  log "creating a ${wanted} MB swap file"
  fallocate -l "${wanted}M" /swapfile 2>/dev/null ||
    dd if=/dev/zero of=/swapfile bs=1M count="$wanted" status=none
  chmod 600 /swapfile
  mkswap /swapfile >/dev/null
  swapon /swapfile || warn "could not enable the swap file"
  grep -q '^/swapfile' /etc/fstab || echo "/swapfile none swap sw 0 0" >>/etc/fstab
}

configure_swap "$SWAP_MB"

if ! id "$KIOSK_USER" >/dev/null 2>&1; then
  log "creating the $KIOSK_USER user"
  useradd --create-home --shell /bin/bash "$KIOSK_USER"
fi

for group in video render input seat; do
  getent group "$group" >/dev/null 2>&1 && usermod -aG "$group" "$KIOSK_USER"
done

systemctl enable seatd >/dev/null 2>&1 || warn "could not enable seatd"

log "writing the kiosk configuration"
cat >"$BOOT_DIR/pistation.json" <<JSON
{
  "serverUrl": "$SERVER_URL",
  "enrollmentToken": "$ENROLLMENT_KEY",
  "joinUrl": "$JOIN_URL"
}
JSON
chmod 600 "$BOOT_DIR/pistation.json"

BOOT_CONFIG="$BOOT_DIR/config.txt"
if [[ -f "$BOOT_CONFIG" ]]; then
  log "tuning video output for $PROFILE"
  sed -i -e '/^gpu_mem=/d' -e '/^dtoverlay=vc4-kms-v3d/d' -e '/^disable_overscan=/d' "$BOOT_CONFIG"
  {
    # A Pi 5 allocates graphics memory dynamically and ignores gpu_mem entirely.
    [[ -n "$GPU_MEM" ]] && echo "gpu_mem=$GPU_MEM"
    echo "dtoverlay=vc4-kms-v3d"
    echo "disable_overscan=1"
  } >>"$BOOT_CONFIG"
else
  warn "no config.txt at $BOOT_CONFIG, skipping video tuning"
fi

KIOSK_UID="$(id -u "$KIOSK_USER")"
KIOSK_BINARY="$(command -v pistation-kiosk || echo /usr/bin/pistation-kiosk)"

log "installing the systemd unit"
cat >/etc/systemd/system/pistation-kiosk.service <<UNIT
[Unit]
Description=PiStation kiosk
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$KIOSK_USER
Group=$KIOSK_USER
PAMName=login
TTYPath=/dev/tty1
StandardInput=tty
StandardOutput=journal
StandardError=journal
TTYReset=yes
TTYVHangup=yes
TTYVTDisallocate=yes

Environment=XDG_RUNTIME_DIR=/run/user/$KIOSK_UID
Environment=WLR_LIBINPUT_NO_DEVICES=1
Environment=GDK_BACKEND=wayland
Environment=PISTATION_VIDEO_MAX_WIDTH=$VIDEO_MAX_WIDTH
Environment=PISTATION_VIDEO_FPS=$VIDEO_FPS
Environment=PISTATION_VIDEO_QUALITY=$VIDEO_QUALITY

ExecStart=/usr/bin/cage -d -- $KIOSK_BINARY

Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
UNIT

systemctl daemon-reload
systemctl enable pistation-kiosk.service

log "disabling services this kiosk does not need"
systemctl disable --now bluetooth.service >/dev/null 2>&1 || true
systemctl disable --now triggerhappy.service >/dev/null 2>&1 || true

log "done"
echo
echo "  model     $MODEL"
echo "  profile   $PROFILE, ${VIDEO_MAX_WIDTH}px at ${VIDEO_FPS}fps, quality ${VIDEO_QUALITY}"
echo "  swap      ${SWAP_MB} MB"
echo "  server    $SERVER_URL"
echo "  join url  $JOIN_URL"
echo "  config    $BOOT_DIR/pistation.json"
echo

if [[ "$SKIP_REBOOT" == "yes" ]]; then
  log "reboot when ready, or run: systemctl start pistation-kiosk"
else
  log "rebooting into the kiosk in 5 seconds, press ctrl c to cancel"
  sleep 5
  reboot
fi
