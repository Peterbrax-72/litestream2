# LiteStream Studio

Publisher: pinchezmedia254 (pinchezmedia254.netlify.app)

LiteStream is a lightweight, OBS-inspired capture studio with a dual-canvas **Preview / Program** workflow. Edits are staged in Preview; the Transition button (Fade or Cut) publishes a snapshot to Program, and the RT option makes Program follow Preview live. Recording and streaming use Program, so later Preview edits do not silently alter the output already on air.

## Features

- **Workspace that remembers:** scenes and sources (images, videos, slide decks, Bible verse decks, text and color backdrops) are saved automatically on this PC and reopen as you left them. Screen, camera and microphone sources reappear in the list but must be shared again, because Windows requires a fresh permission each launch. Program always starts empty, so nothing goes on air by accident.
- Screen/window/tab capture, camera, microphone, local images and video clips (with play/pause, seek, loop and volume), color backdrops, and editable text.
- **Transitions:** Fade, Cut, or **RT** (realtime), where Program follows Preview instantly. RT is off at every launch.
- Text overlays always draw above other sources. Drag and resize text and images in Preview with the mouse; scrolling text (horizontal or vertical, adjustable speed) is available in Text formatting.
- **Source lock:** each source has a padlock that blocks moving, resizing, editing, renaming and removing, to prevent accidents. Click a source's name to rename it when unlocked.
- Source Fit: Fit to screen (default), Fill, Stretch, Original size and 50-200% scale.
- **Scripture panel:** a movable, collapsible panel in the right column, like Presentation and Program monitor. Books are split into Old and New Testament lists, with chapters and verses below, plus quick reference search and book autocomplete (`1kings 3:5`, `pet`). Click a verse, Shift-click for a range, or double-click to send that verse and the rest of the chapter as a verse deck (one slide per verse). Choose fullscreen with built-in or uploaded wallpapers, or a lower third over live video. Verse decks can be dragged and resized in Preview. Works with downloaded or imported editions and the online NLT.
- **Bible editions** (Quick actions → Bible editions / import; used only to add editions): download public-domain editions in the app, or import your own file (.zip, .txt verse-per-line, .usfm or .json) from sources such as eBible.org. The NLT can be added as an online edition through Tyndale's official API (internet required, non-commercial use). NIV and NKJV are copyrighted and are not included.
- Import `.pptx` slide decks and navigate slides in Preview. The lightweight parser renders text, basic shapes and embedded images; advanced effects, fonts, transitions, audio/video and complex charts may not match PowerPoint.
- Reorder and collapse sidebar panels (the order is saved).
- **Program Projector:** right-click the Program window to send the live output fullscreen to any connected display, or use the Projector button. Shows what is on air in a separate window.
- Local WebM recording and a native RTMP sender for one Restream-compatible ingest.
- Contact details for the developer (GitHub, WhatsApp, email, website) are in Settings.

## Important: RTMP requirements

The native RTMP command uses **FFmpeg** installed on the machine and available as `ffmpeg` on `PATH`. If it is installed elsewhere, set `LITESTREAM_FFMPEG` to its executable path before launching LiteStream. The current installer does not bundle FFmpeg, so package and license the binary separately if distributing it. RTMP is not available in the plain browser preview. Enter the restream service's RTMP/RTMPS server URL and stream key in the app; keep that key private.

The native FFmpeg integration is source-implemented but has not been compiled or verified in a Windows build here. Validate it against your selected restream provider before a live event.

## Run the UI in a browser

For a quick UI preview, serve `ui/` from a local HTTP server or open `ui/index.html` in a current browser. Screen, camera, and microphone permissions generally require a secure context (`https://` or `localhost`). Internet access is needed to search/download Bible editions; imported texts are stored locally by the browser/webview. Screen sharing is started only after the user chooses the capture action.

## Build the Windows installer

The configured bundle target is a Windows NSIS installer. Build on Windows or in a correctly configured Windows environment:

1. Install Node.js LTS, current stable Rust (1.90 or newer) with the Windows MSVC target, Microsoft C++ Build Tools/Windows SDK, and the WebView2 Runtime.
2. Install FFmpeg separately if you need RTMP output and make it available on `PATH` or set `LITESTREAM_FFMPEG`.
3. From this folder run:

   ```powershell
   npm install --include=dev
   npm run dev
   npm run build:windows
   ```

The Windows NSIS installer is placed under `src-tauri/target/release/bundle/nsis/`. The default `npm run build` also targets only NSIS for this release. A GitHub Actions workflow in `.github/workflows/windows-installer.yml` builds the same installer on a Windows runner and publishes it as a downloadable workflow artifact when this project is pushed to GitHub.

### Cross-build the Windows installer from Linux

A Windows host is not required: Tauri supports a Linux-to-Windows MSVC cross-build using `cargo-xwin`. This builds the Windows target directly and does **not** build or link the Linux GTK target. On Debian/Ubuntu, install the Linux-host tools (`nsis`, `llvm`, `lld`, `clang`, `build-essential`, `pkg-config`, `libssl-dev`, `curl`, `file`), Rust stable with `x86_64-pc-windows-msvc`, and `cargo-xwin`; then install Node dependencies and run:

```sh
npm install --include=dev
cargo install --locked cargo-xwin
rustup target add x86_64-pc-windows-msvc
npm run build:windows:cross
```

The first cross-build downloads the Windows MSVC CRT/SDK and can take substantial time and disk space. These Linux packages are build-host tooling, not Windows application dependencies. GTK remains relevant only if you also build LiteStream for Linux; there is no need to remove Tauri's platform-specific dependencies to produce the Windows installer.

## Release platform: Windows only

This release is configured to build a Windows NSIS installer (`.exe`) only. macOS and Linux release targets are intentionally deferred; the app can be revised and packaged for those systems later. Build on Windows with the MSVC Rust toolchain, Visual Studio C++ Build Tools/Windows SDK, Node.js LTS, and WebView2. Screen capture, camera access, MediaRecorder codecs, monitor enumeration, and audio routing still depend on the Windows device and runtime. Program Projector mirrors video, not audio. Validate the installer and actual capture devices on Windows before distribution. Recordings are WebM, with codecs determined by the system runtime.

## Privacy and security

Capture permissions are requested after the user starts a source. Media composition and recording are local. When broadcasting, Program media is sent to the RTMP/RTMPS endpoint and key supplied by the user. Bible catalog requests go to GetBible and NLT lookups to api.nlt.to; downloaded and imported editions, and your saved workspace, remain in local webview storage on this PC. Do not share stream keys.
