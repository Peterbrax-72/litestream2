# Changelog

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
