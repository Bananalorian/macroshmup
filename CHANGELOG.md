# Changelog

Every version is one build sprint. Newest first. Versions are tagged in git, and
each tag publishes a GitHub Release with downloads.

## [0.1.0] - 2026-10-10 - First light

Sprint 0: the empty game runs everywhere and ships itself.

### Added
- **A title screen** in the style of the PILGRIM sheet: the name in spaced
  capitals, SOMEWHERE BEYOND THE EVENT HORIZON, the palette strip, the corner
  sparkle, and the stamps along the bottom (`V0.1 // PILGRIM`, HUMANITY STILL
  LOOKS UP, A SMALL SHIP / A LONG WAY FROM HOME). Press Z or Enter, or tap, to start.
- **An empty playfield** (270x360) with a three-layer parallax starfield falling
  past it, framed by the HUD panels. Esc goes back to the title.
- **Pixel-perfect scaling.** Everything draws into a 640x360 canvas, which is
  scaled up by a whole number (2x at 720p, 3x at 1080p, 4x at 1440p, 6x at 4K) with
  nearest filtering and letterboxed. Every screenshot is checked block by block.
- **A portrait layout for phones:** the same 270x360 playfield on a 270x480 canvas,
  with a HUD strip above and a thumb zone below. On a 390x844 phone at 3x pixel
  density, the canvas is drawn at 4x.
- **A fixed 60 Hz simulation.** The game advances in whole ticks; the screen draws
  every display frame. Vsync jitter never drops or doubles a tick, 120 and 144 Hz
  screens still get 60 ticks a second, and after a stall the game catches up at
  most 4 ticks instead of fast-forwarding.
- **Fullscreen** with F11 or Alt+Enter.
- The pixel font from Last Train to Shinjuku, a sprite atlas built in code,
  the palette from the art bible, and `tune.rs` for every tunable number.
- **Debug hooks:** `PG_SCENE=title` / `stage:<seconds>`, `PG_SHOTS` (saves to
  `./shots/`), `PG_SEED`, `PG_GRAY`, `PG_SIZE`, `PG_PERF`, and F8 for the
  performance overlay in debug builds.
- **Builds and releases:** every push to `main` publishes the browser version to
  GitHub Pages; every version tag publishes a Release with the browser file and
  Windows, macOS (universal) and Linux downloads. CI runs the tests first.

### Fixed
- Tagged builds no longer fail at the Pages step: only `main` deploys to Pages
  (Shinjuku's workflow tried to deploy tags too, and GitHub rejected them).
