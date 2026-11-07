# Changelog

## 0.6.0
- New: LiteStream Studio for Android. A new "Android APK" GitHub workflow builds an installable `.apk` (artifact `LiteStream-Android-APK`). Phone layout, camera and microphone inputs, text, clock, scripture and slides, and recordings saved to Downloads/LiteStream.
- Not on Android yet: screen capture (Android's built-in web view does not support it), RTMP live streaming, and the second Program monitor window.
- Phone landscape layout: Preview and Program side by side with a compact Transition button.
- Windows: no behavior change. Tag releases now attach each platform's file to the same GitHub Release.

## 0.5.0
- New: Frames. Give a camera, screen or video a rectangle, rounded or circle frame, and lay sources out side by side, top and bottom, or as an overlay you drag to any corner of Preview. Quick layouts, size slider and colored border included. Opens automatically when you add a second capture source, or from Source fit > Frames, or by right-clicking Preview.
- New: edit sources in place. Double-click text in Preview (or use the ✎ icon next to a source, or right-click) to change its words, color and size without adding a new one. The ✎ icon also edits a clock's time zone or a countdown's time, and switches the camera, microphone or captured screen/window while keeping its fit and frame settings.
- Sources panel: the ✎ icon now comes first on every source and opens all the right-click options (edit, fit, scale, layout, frames) for text, clocks, camera, mic, screen, image, video and slides. The "Text overlay" label is gone.
- Frames: overlays and shaped images stay draggable and can be resized by dragging their corner handles.

## 0.4.0
- New: Modes. Pick Default (every panel, the standard look), Church (slides, scriptures, text, camera and mic, clock and timer), Influencer combo (text and all inputs, no slides, Bible or clock) or Custom (choose your own panels) from the mode button in the header.
- New: a + button below the side panels (Add or remove panels) to show or hide panels. Your mode and custom choices are remembered.

## 0.3.2
- Fix: after importing a video, the Video playback panel opens, moves to the top of the side panels and highlights the Play button, and the first frame now shows in Preview (it used to look frozen and blank).

## 0.3.1
- New: Add clock / countdown in Quick actions.
- New: Tutorials button in the header opens the YouTube tutorials (youtube.com/@pinchezmedia254).
- New: quick guided tour for new users on first launch, and a "What's new" window after each update. Reopen the tour any time with the ? button.

## 0.3.0
- New: Clock panel under Quick actions. A 24-hour (or 12-hour) world clock where you pick a time zone, and a countdown timer with presets, start/pause/reset and an optional beep. Both work offline. Clock and Countdown are also in the Add source menu, with per-source time zone, text color and backplate color, and can be dragged in Preview and resized from any edge or corner (text boxes in general now have 8 resize handles).

## 0.2.0
- Fix: text overlays and clocks are now vertically centred in their box.
- New: right-click in the Preview window to pick a source and set its fit (fit, fill, stretch, original, auto) and scale.
- Change: removed the right-click source-fit menu from the Preview window. Use the Source fit panel instead.
- Change: new text overlays and clocks no longer get a backplate by default (tick Backplate to add one).
- Fix: clocks and countdowns keep ticking on Program after a transition, like a live screen capture (they used to freeze at the moment of the transition).
- Change: right-click in Preview does nothing when there is nothing to fit (no sources, or only text away from the text box); no more "add a source first" popup.
- Brand: the LiteStream logo (brand/litestream-logo.svg) is now the official app icon and Store art.
- Packaging: Microsoft Store MSIX build, store listing kit and release scripts.
- New: Support button in the header with BuyMeSoda (QR code + link).
- New: Scripture panel has Prev/Next verse arrows; sending a verse now loads the whole chapter as the deck, opened at your verse, so you can go back to earlier verses as well as forward.
- Fix: the microphone toggle in Settings now turns the mic on (asks for permission and connects it) instead of doing nothing, then mutes/unmutes.

## 0.1.0
- First release: scenes and sources, Preview/Program with Transition, Fade, Cut and RT,
  Scripture panel with lower thirds and wallpapers, Bible import, presentations (PPTX),
  audio mixer, recording, RTMP streaming, projector output and workspace memory.
