# Microsoft Store submission kit – LiteStream Studio

## Partner Center values
- Product type: **EXE or MSI app**
- Name to reserve: LiteStream Studio
- Package URL: the "Package URL for Partner Center" line in the workflow run summary (tag build)
- Architecture: x64 · Installer type: NSIS/EXE · Silent install parameter: `/S`
- Languages: English
- Category: Photo & video
- Privacy policy URL: https://pinchezmedia254.netlify.app/privacy/  (deploy `store/privacy/` to your site)
- Support / website: https://pinchezmedia254.netlify.app · sirpeterbaraka@gmail.com

## Short description (<= 100 chars)
Lightweight capture, presentation and streaming studio with scenes, scripture and a projector.

## Description
LiteStream Studio is a lightweight studio for recording, presenting and live streaming.

- Build scenes from screen, window and camera capture, video, images and text
- Program and Preview monitors with Transition, Fade, Cut and real-time (RT) modes
- Scripture panel: pick a book and chapter, send verses to the screen as full-screen slides or lower thirds, with built-in wallpapers
- Import your own Bible editions; optional online NLT lookup
- Presentations (PPTX) with live control
- Audio mixer, local recording and RTMP streaming
- Full-screen projector output to a second display
- Lock sources, rename by clicking, and your workspace reopens as you left it

## Features (up to 20 short bullets)
Scenes and sources · Preview/Program · RT transitions · Scripture lower thirds · Bible import · PPTX presentations · Audio mixer · RTMP streaming · Projector output · Workspace memory

## Keywords (7 max)
streaming, recording, capture, church, presentation, scripture, projector

## Notes for certification
LiteStream Studio is a local capture and streaming studio. No account or login is needed. Screen/camera/microphone capture asks for permission inside the app. The optional NLT Bible feature uses the official api.nlt.to and needs an internet connection; the app works fully offline without it. Install silently with /S (per-user install, no admin needed).

## Screenshots to capture (PNG, 1366x768 or larger, 4-6)
1. Main studio with scenes and sources
2. Scripture panel with verse preview
3. Program view with a lower-third verse
4. Presentation panel
5. Audio mixer / settings
6. Projector on a second display (optional)

## Age rating questionnaire
No violence, no mature content, no user-generated content sharing, no purchases, no location. Streams are user-initiated to third-party services.

## Release checklist
1. (Optional) Add GitHub secrets for signing: AZURE_CLIENT_ID, AZURE_TENANT_ID, AZURE_CLIENT_SECRET, AZURE_SIGN_ENDPOINT, AZURE_SIGN_ACCOUNT, AZURE_SIGN_PROFILE
2. Bump version in package.json and src-tauri/tauri.conf.json
3. `git tag v0.1.0 && git push origin v0.1.0` -> workflow builds the Store installer and publishes a GitHub Release
4. Test on a clean Windows PC under a standard user: `LiteStream-Store_0.1.0_x64-setup.exe /S`, check Start menu + Apps list, uninstall cleanly
5. Create the product in Partner Center, paste the Package URL, submit

## MSIX route (current)
1. In Partner Center create a new **MSIX or PWA app** and reserve the name "Litestream" (delete the old Win32 app first).
2. Product management > Product identity: copy Package/Identity/Name, Package/Identity/Publisher (CN=...), Package/Properties/PublisherDisplayName.
3. GitHub repo > Settings > Secrets and variables > Actions > **Variables**: add MSIX_IDENTITY_NAME, MSIX_PUBLISHER, MSIX_PUBLISHER_DISPLAY.
4. Run the "Windows installer" workflow; download artifact **LiteStream-Store-MSIX** (LiteStream_0.1.0.0_x64.msix).
5. Submission > Packages: upload the .msix. Microsoft signs it. No URL or hosting needed.
