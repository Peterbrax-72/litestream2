# LiteStream Studio

Publisher: pinchezmedia254 (pinchezmedia254.netlify.app)

LiteStream Studio is a lightweight, OBS-inspired live capture and presentation tool built around a dual-canvas Preview/Program workflow. Edits are staged in Preview, and the Transition button (Fade or Cut) publishes a snapshot to Program when you're ready.

## Features

- Workspace that remembers: scenes and sources (images, videos, slide decks, Bible verse decks, text, and color backdrops) are saved automatically on this PC and reopen as you left them.
- Screen, window, and tab capture, plus camera and microphone inputs.
- Local media sources such as images and video clips with play/pause, seek, loop, and volume controls.
- Color backdrops and editable text layers.
- Transitions: Fade, Cut, or RT (realtime), where Program follows Preview instantly. RT starts disabled on every launch.
- Text overlays always render above other sources. Drag and resize text and images directly in Preview with the mouse.
- Scrolling text (horizontal or vertical, with adjustable speed) is available in the text formatting controls.
- Source lock: each source has a padlock that blocks moving, resizing, editing, renaming, and removing to prevent accidental changes. Click a source name to rename it when unlocked.
- Source fit modes: Fit to screen (default), Fill, Stretch, Original size, and 50-200% scaling.
- Scripture panel: a movable, collapsible right-column panel with Old and New Testament books, chapters, and verses.
- Bible editions: download public-domain editions from inside the app or import your own files (`.zip`, `.txt` verse-per-line, `.usfm`, or `.db` formats).
- Import `.pptx` slide decks and navigate slides in Preview. The lightweight parser renders text, basic shapes, and embedded images.
- Reorder and collapse sidebar panels; order is saved automatically.
- Program Projector: right-click the Program window to send the live output to a fullscreen display, or use the Projector button to show what is on air in a separate window.
- Local WebM recording and a native RTMP sender for a single Restream-compatible ingest.
- Contact details for the developer (GitHub, WhatsApp, email, website) are available in Settings.

## Quick start

### Run the UI in a browser

For a quick UI preview, serve `ui/` from a local HTTP server or open `ui/index.html` in a browser. Screen, camera, and microphone permissions typically require a secure context (`https://` or localhost).

## Important: RTMP requirements

The native RTMP command uses FFmpeg installed on the machine and available as `ffmpeg` on `PATH`. If it is installed elsewhere, set `LITESTREAM_FFMPEG` to the executable path before launching LiteStream.

The FFmpeg integration is source-implemented but has not been compiled or verified in a Windows build here. Validate it against your selected restream provider before a live event.

## Build the Windows installer

This release is configured to build a Windows NSIS installer (`.exe`) only. macOS and Linux release targets are intentionally deferred; those platforms can be revisited later.

Build on Windows or in a correctly configured Windows environment:

1. Install Node.js LTS.
2. Install the current stable Rust (1.90 or newer) with the Windows MSVC target.
3. Install the Microsoft C++ Build Tools and Windows SDK.
4. Install the WebView2 Runtime.
5. Install FFmpeg separately if you need RTMP output and make it available on `PATH`, or set `LITESTREAM_FFMPEG`.
6. From this folder, run:

```powershell
npm install --include=dev
npm run dev
npm run build:windows
```

The Windows NSIS installer is placed under `src-tauri/target/release/bundle/nsis/`.

The default `npm run build` also targets only NSIS for this release.

### Cross-build the Windows installer from Linux

A Windows host is not required. Tauri supports a Linux-to-Windows MSVC cross-build using `cargo-xwin`. This builds the Windows target directly and does not build or link the Linux GTK target.

```sh
npm install --include=dev
cargo install --locked cargo-xwin
rustup target add x86_64-pc-windows-msvc
npm run build:windows:cross
```

The first cross-build downloads the Windows MSVC CRT/SDK and can take substantial time and disk space. These Linux packages are build-host tooling, not Windows application dependencies. GTK remains a Linux-only dependency and is not bundled into the Windows build.

## Development notes

- The project is built with Node.js and Tauri.
- The app uses a local workspace model that saves scenes and sources automatically.
- For browser previews, ensure you are using a local secure context for media permissions.

## Privacy and security

Capture permissions are requested only after the user starts a source. Media composition and recording remain local to the machine. When broadcasting, Program media is sent to the RTMP/RTMPS endpoint and key supplied by the user or platform.

## License and support

This project is intended for local use and Windows deployment. For developer contact details and support information, see the Settings page in the application.
