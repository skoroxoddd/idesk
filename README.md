# iDesk — Remote Desktop (AnyDesk Analog)

Cross-platform remote desktop application built with Rust + Tauri. Connect to any computer using a 9-digit ID.

## Architecture

```
Controlled (Rust/Tauri)              Controller (Browser/Tauri)
┌──────────────────┐                 ┌────────────────────┐
│ Screen Capture   │ → H.264 →      │ <video> element    │
│ (scap/x11/PW)    │   WebRTC →     │ (GPU decode)       │
│ openh264 encode  │                 │                    │
│ enigo inject     │ ← DataChannel ← │ Mouse/Keyboard     │
└──────────────────┘                 └────────────────────┘
         ↕ WebSocket (SDP/ICE signaling only)
    Signaling Server (actix-web)
```

### Key Design Decisions

- **Asymmetric WebRTC**: The controlled side encodes H.264 in Rust; the controller side decodes natively via `<video>` element. No decoder written in Rust.
- **9-digit session IDs**: Simple, human-readable IDs (XXX-XXX-XXX) for connection.
- **Adaptive bitrate**: Pipeline monitors encode time and adjusts bitrate automatically.
- **Cross-platform**: Linux (X11/Wayland), Windows, macOS.

## Project Structure

```
remote-desktop/
├── Cargo.toml                          # Workspace root
├── .github/workflows/build.yml         # CI: Windows build
├── crates/
│   ├── remote_desktop_core/            # Core library — all platform logic
│   │   └── src/
│   │       ├── capture/                # Screen capture (per-platform)
│   │       │   ├── capturer.rs         # Capturer trait
│   │       │   ├── frame.rs            # CaptureFrame (BGRA→RGBA)
│   │       │   ├── factory.rs          # Platform capturer factory
│   │       │   ├── mock_capturer.rs    # Test capturer
│   │       │   ├── scap_capturer.rs    # macOS/Windows (scap)
│   │       │   ├── x11_capturer.rs     # Linux X11 (x11rb)
│   │       │   └── pipewire_capturer.rs # Linux Wayland (ashpd/PipeWire)
│   │       ├── encode/                 # H.264 video encoding
│   │       │   ├── encoder.rs          # Encoder trait + EncodedFrame
│   │       │   ├── openh264_encoder.rs # Cisco OpenH264 (low-latency)
│   │       │   └── ffmpeg_encoder.rs   # FFmpeg sidecar fallback
│   │       ├── input/                  # Remote input injection
│   │       │   ├── events.rs           # InputEvent enum (mouse/keyboard)
│   │       │   ├── injector.rs         # InputInjector trait
│   │       │   ├── enigo_injector.rs   # enigo-based implementation
│   │       │   └── clipboard.rs        # Clipboard sync
│   │       ├── network/                # WebRTC networking
│   │       │   ├── signaling.rs        # Signaling client (SDP/ICE)
│   │       │   └── ice.rs              # ICE/STUN configuration
│   │       ├── connection/             # Session management
│   │       │   ├── id.rs               # 9-digit SessionId (XXX-XXX-XXX)
│   │       │   └── auth.rs             # PIN-based auth
│   │       ├── stream/                 # Capture→Encode pipeline
│   │       │   ├── pipeline.rs         # StreamPipeline (tokio channels)
│   │       │   └── rate_control.rs     # Adaptive bitrate controller
│   │       ├── error.rs                # Unified AppError type
│   │       └── lib.rs                  # Crate entry point
│   └── signaling_server/               # Standalone WebSocket relay
│       └── src/
│           ├── main.rs                 # actix-web entry point
│           ├── server.rs               # HTTP/WS server setup
│           ├── ws_handler.rs           # WebSocket peer handler
│           ├── registry.rs             # PeerRegistry (online/offline)
│           └── messages.rs             # Signaling message types
├── src-tauri/                          # Tauri 2.x desktop app
│   ├── src/
│   │   ├── main.rs                     # Tauri builder + commands
│   │   ├── state.rs                    # Shared AppState
│   │   ├── commands/
│   │   │   ├── connection.rs           # get_session_id, connect_to_peer...
│   │   │   └── settings.rs             # set_quality, set_fps
│   │   └── platform/                   # Platform-specific init
│   │       ├── linux.rs
│   │       ├── macos.rs
│   │       └── windows.rs
│   ├── tauri.conf.json                 # Tauri config
│   └── capabilities/default.json       # Tauri permissions
├── src/                                # React + TypeScript frontend
│   ├── App.tsx                         # Root: ConnectionScreen ↔ RemoteDesktopView
│   ├── main.tsx                        # React entry point
│   ├── components/
│   │   ├── ConnectionScreen.tsx        # ID display + connect input
│   │   └── RemoteDesktopView.tsx       # Remote video + toolbar
│   └── styles/globals.css              # App styles
├── package.json                        # Frontend deps (React, Vite, Zustand)
├── tsconfig.json
├── vite.config.ts
└── index.html
```

## Components

### remote_desktop_core

The core library contains all platform-independent logic:

**Capture** — `Capturer` trait with platform-specific implementations:
- `scap_capturer`: macOS/Windows via the `scap` crate
- `x11_capturer`: Linux X11 via `x11rb` + XShm
- `pipewire_capturer`: Linux Wayland via `ashpd` + PipeWire (feature flag `wayland`)
- `mock_capturer`: Returns solid-color frames for testing

**Encode** — `Encoder` trait returning `EncodedFrame`:
- `openh264_encoder`: Cisco OpenH264, low-latency config (keyframe every 30 frames)
- `ffmpeg_encoder`: FFmpeg sidecar fallback for systems without OpenH264

**Input** — Remote control via `InputEvent` enum:
- Mouse: Move, Down, Up, Wheel
- Keyboard: KeyDown, KeyUp with modifiers (Ctrl/Alt/Shift/Meta)
- Clipboard: ClipboardSet for sync
- Binary serialization via `serde_json`
- `EnigoInjector` injects events via the `enigo` crate

**Stream** — `StreamPipeline` connects capture → encode → output:
- Async tokio loop with bounded channel (256 capacity)
- SPS/PPS prepended to keyframes for decoder initialization
- Adaptive bitrate: if encode time > 80% of frame budget, reduce by 10%
- `RateController`: separate adaptive bitrate controller (min/max/step)

**Connection** — Session management:
- `SessionId`: 9-digit random ID, formatted as `XXX-XXX-XXX`
- `AuthManager`: PIN-based authentication with FNV-1a hashing

**Error** — Unified `AppError` enum with variants for each subsystem.

### signaling_server

Standalone WebSocket relay server (actix-web):
- `POST/GET /ws` — WebSocket endpoint for SDP/ICE exchange
- `GET /health` — Health check
- `PeerRegistry`: tracks online peers, routes messages between peers
- Messages: `Register`, `Offer`, `Answer`, `IceCandidate`, `Disconnect`

Run: `cargo run --bin signaling_server`
Env: `BIND_ADDR` (default `0.0.0.0:8080`), `RUST_LOG`

### src-tauri (Desktop App)

Tauri 2.x application exposing core functionality via IPC commands:

| Command | Description |
|---------|-------------|
| `get_session_id` | Returns this machine's 9-digit ID |
| `connect_to_peer` | Initiates WebRTC connection to peer ID |
| `disconnect` | Closes active connection |
| `check_peer_online` | Checks if peer is registered on signaling server |
| `set_quality` | Adjusts encoding quality |
| `set_fps` | Adjusts frames per second |

### Frontend (React + TypeScript)

Two-screen UI:
- **ConnectionScreen**: Shows your ID, input for peer ID, copy button
- **RemoteDesktopView**: `<video>` element for remote stream + toolbar (disconnect, fullscreen, quality)

## Building

### Prerequisites

- Rust 1.70+
- Node.js 18+
- Tauri CLI: `cargo install tauri-cli --locked`

### Linux

```bash
# X11 (default)
cargo build

# Wayland
cargo build --features wayland
```

### Windows

```bash
cargo tauri build
```

### macOS

```bash
cargo tauri build
```

### CI (GitHub Actions)

Push to `master` triggers automatic Windows build. Download `.exe`/`.msi` from [Actions](https://github.com/skoroxoddd/idesk/actions).

## Running

### 1. Start Signaling Server

```bash
cd crates/signaling_server
cargo run
# Server starts on 0.0.0.0:8080
```

### 2. Start Desktop App (Dev Mode)

```bash
cargo tauri dev
```

### 3. Connect

- App shows your 9-digit ID (e.g., `123-456-789`)
- Enter peer ID on the other machine
- Click Connect

## Testing

```bash
cargo test
cargo clippy
```

Mock capturer returns color-cycling frames — useful for encode pipeline testing without display.

## Configuration

| Env Variable | Default | Description |
|-------------|---------|-------------|
| `BIND_ADDR` | `0.0.0.0:8080` | Signaling server bind address |
| `RUST_LOG` | — | Tracing log level |

## Roadmap

- [ ] Complete WebRTC connection (Phase 4)
- [ ] Full input injection pipeline (Phase 5)
- [ ] Loopback test (two instances on same machine)
- [ ] Performance: target < 200ms latency on LAN
- [ ] TURN server support for NAT traversal
- [ ] Packaging: DMG, NSIS, AppImage
