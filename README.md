<p align="center">
  <img src="assets/logo.svg" width="88" alt="PiStation" />
</p>

<h1 align="center">PiStation</h1>

<p align="center">
  Turn any TV or monitor into a screen the whole room can use.
</p>

<p align="center">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-4f7cff?style=flat-square" />
  <img alt="Self hosted" src="https://img.shields.io/badge/self%20hosted-yes-21c17a?style=flat-square" />
  <img alt="Docker" src="https://img.shields.io/badge/deploy-Docker%20Compose-2496ed?style=flat-square&logo=docker&logoColor=white" />
  <img alt="Raspberry Pi" src="https://img.shields.io/badge/runs%20on-Raspberry%20Pi-c51a4a?style=flat-square&logo=raspberrypi&logoColor=white" />
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/server-Rust-dea584?style=flat-square&logo=rust&logoColor=white" />
  <img alt="SvelteKit" src="https://img.shields.io/badge/web-SvelteKit-ff3e00?style=flat-square&logo=svelte&logoColor=white" />
  <img alt="Tauri" src="https://img.shields.io/badge/kiosk-Tauri-24c8db?style=flat-square&logo=tauri&logoColor=white" />
  <img alt="LiveKit" src="https://img.shields.io/badge/media-LiveKit-1f8fff?style=flat-square" />
</p>

---

A Raspberry Pi drives a TV or monitor and shows a six digit code. Anyone on the network types
that code into the website to take the screen. They can share a screen or a phone camera, draw
over what is being shown, or open a shared whiteboard. When nobody is presenting, the display
becomes a dashboard.

No accounts. No cloud. Nothing leaves the network it runs on.

## What it does

- **Screen sharing** from any browser, no install and no cable. Anyone in the room can take a
  turn, one at a time.
- **Camera sharing** from phones and laptops. Point a phone at a whiteboard or a workbench and
  it goes on the big screen, with front and rear cameras and a flip button. A screen share always
  takes priority over a camera.
- **Live annotation** over whatever is on screen, with an adjustable pen, arrows, shapes and a
  highlighter. Everyone's cursor carries their name.
- **Shared whiteboard** powered by Excalidraw, mirrored onto the display.
- **Idle dashboard** with clock, weather, agenda and rotating wallpapers, plus a builder for
  your own widgets.
- **Night mode** that drops the screen to just the time and the join code outside working hours.
- **Rotating codes**, changed every minute, so an old photo of the screen is worthless. Sessions
  survive the rotation.
- **Health reporting** from every kiosk: CPU, memory, Wi-Fi signal, temperature and uptime.

## Self hosting

Everything ships as a Docker Compose stack: a LiveKit media server, a Rust backend and the
website.

```
git clone <your-repo> pistation
cd pistation
cp infra/.env.example infra/.env
```

Edit `infra/.env` and set at least `LIVEKIT_API_SECRET`, `SESSION_SECRET` and
`BOOTSTRAP_ADMIN_PASSWORD`. Then:

```
docker compose -f infra/docker-compose.yml up -d
```

| Service | Port |
| --- | --- |
| Website | 3000 |
| API | 8080 |
| LiveKit | 7880 |

If the host runs a firewall, open the LiveKit ports:

```
sudo ./infra/firewall.sh
```

It allows 7880 and 7881 TCP, 3478 UDP and 50000-50100 UDP from your LAN, handling ufw and
firewalld. Pass `--dry-run` to see the commands first, `--subnet` to name a different network or
`--open-to-all` to drop the source restriction. The website and API need no rules, since Docker
publishes those ports with its own firewall rules. LiveKit runs on the host network and does not
get that treatment, so a firewalled host will load the site and then fail to connect to a room.

Sign in at `/admin` with the bootstrap admin details from your `.env`.

If you are serving anything other than localhost, set `PUBLIC_API_URL`, `PUBLIC_LIVEKIT_URL` and
`PUBLIC_WEB_URL` in `infra/.env` to addresses your devices can actually reach, add
`PUBLIC_WEB_URL` to `CORS_ORIGINS`, and change `turn.domain` in `infra/livekit.yaml` to the same
host. LiveKit reads that file literally, so it does not pick up environment variables.

Leaving `CORS_ORIGINS` on its localhost default while browsing by LAN address is the usual cause
of requests failing with a missing `Access-Control-Allow-Origin` header.

The LiveKit container uses host networking, because WebRTC needs to advertise an address your
devices can reach and a container on a bridge network has none. That is Linux only; on macOS or
Windows, run LiveKit outside Docker.

## Setting up a screen

Works on the **Raspberry Pi Zero 2 W, Pi 4 and Pi 5**, and needs the **64 bit** Raspberry Pi OS.
The media stack has no 32 bit ARM build, so the installer stops with an explanation rather than
failing halfway if it finds a 32 bit system.

Upload a kiosk build once on the admin home page, then create a kiosk. The admin panel hands you
a single command to paste into an SSH session on a fresh Raspberry Pi OS Lite install. It **must
be run with sudo**, since it installs packages, writes to the boot partition and adds a service:

```
curl -fsSL http://your-server:8080/install.sh | sudo bash -s -- --key <enrollment-token>
```

That installs the kiosk, writes its configuration, enlarges swap, tunes the boot config and
reboots straight into the display. It is safe to re-run, which is also how you repoint a kiosk at
a new server.

The installer reads the board model and picks video settings and swap to match:

| Board | Video | Swap |
| --- | --- | --- |
| Pi Zero 2 W | 960px at 12fps, quality 70 | 2 GB |
| Pi 4 | 1600px at 30fps, quality 80 | 1 GB |
| Pi 5 | 1920px at 60fps, quality 85 | 512 MB |

Raspberry Pi OS ships with 100 MB of swap, which a 512 MB Zero 2 W exhausts as soon as the
browser engine and the media stack are both running. Without more, the kernel starts killing
processes and the kiosk appears to restart at random.

Useful flags: `--server`, `--join-url`, `--package-url`, `--user`, `--profile`, `--swap`,
`--skip-reboot`.

## Configuration

Set in `infra/.env`:

| Variable | What it does |
| --- | --- |
| `LIVEKIT_API_KEY` / `LIVEKIT_API_SECRET` | Credentials shared by the server and LiveKit |
| `SESSION_SECRET` | Signs admin sessions, make it long and random |
| `BOOTSTRAP_ADMIN_EMAIL` / `_PASSWORD` | Creates the first admin account on startup |
| `PIN_ROTATION_SECONDS` | How often join codes change, 30 to 60 |
| `PIN_GRACE_SECONDS` | How long an old code keeps working after it changes |
| `PUBLIC_API_URL` / `PUBLIC_LIVEKIT_URL` | Addresses browsers and kiosks use to reach you |
| `PUBLIC_WEB_URL` | The address the website is served on. Must match what you type in the browser, scheme included, or server rendered pages disagree with the browser |
| `CORS_ORIGINS` | Browser origins allowed to call the API, comma separated. Must include `PUBLIC_WEB_URL`. Kiosks are exempt, since a desktop app has no fixed web origin |

Video encoding on the kiosk is tunable per device, without rebuilding:

| Variable | Default | Notes |
| --- | --- | --- |
| `PISTATION_VIDEO_MAX_WIDTH` | `1920` | Incoming video is scaled down past this |
| `PISTATION_VIDEO_FPS` | `60` | Frames beyond this are dropped, never queued |
| `PISTATION_VIDEO_QUALITY` | `85` | 1 to 100 |

A Pi Zero 2W wants roughly `960`, `12` and `70`. Frames are dropped rather than queued at every
stage, so an overloaded kiosk loses frames instead of falling further behind.

## Images

Published to GitHub Container Registry on every push to `main` and every `v*` tag:

```
docker pull ghcr.io/sirblobby/pistation-server:latest
docker pull ghcr.io/sirblobby/pistation-web:latest
```

The Compose stack builds from source by default. To run the published images instead, replace the
`build:` block on the `server` and `web` services in `infra/docker-compose.yml` with an `image:`
line pointing at the tag above.

## Licence

MIT. Source at [github.com/SirBlobby/pistation](https://github.com/SirBlobby/pistation).
