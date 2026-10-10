# Changelog

Every version is one build sprint. Newest first. Versions are tagged in git, and
each tag publishes a GitHub Release with downloads.

## [0.2.0] - 2026-10-10 - The Pilgrim

Sprint 1: the ship feels instant and fast before anything shoots back.

### Added
- **The Pilgrim**, rebuilt pixel by pixel from the sheet: the gothic spire, the
  cross on the spine (the hitbox sits on it), three red reactor dots that glow
  brighter while you fire, hatched buttress wings, and two wing pods with red tips
  and red/teal wingtip lights. Idle, thrust, fire and hit-flash animations, a roll
  left and right as you move sideways, a teal engine plume and trail, and teal
  afterimages at full speed. The pods trail a couple of pixels behind the hull and
  tuck in when you focus. `PG_SHEET=1` saves the whole animation sheet.
- **Movement with no inertia.** The ship moves on the frame the input arrives,
  diagonals are the same speed as straight lines (4 px a frame), and it stops dead
  when you let go. **Focus** slows it to 0.55x over 5 frames and shows the hitbox
  dot and its ring.
- **The wide shot:** twin red bolts down the middle and angled streams from the
  pods, a volley every 3 frames. Shots start at the ship, so a target you're
  sitting on still gets hit. **The focus beam:** fire while focused for a 12 px
  beam that stops at the first thing in its lane, does 1.6x the wide shot's
  damage, and burns anything within 28 px with its aura.
- **Three target dummies** (still, moving, big) that flash, break, come back, and
  show the damage per second they're taking, for comparing the wide shot and the
  beam. Flying into one flashes the hull (nothing can hurt you yet).
- **Controllers**, on desktop and in the browser: the D-pad or left stick (8-way
  by default, analog in the options), A fire, RB/RT focus, B bomb, Start pause.
- **Rumble**, on desktop and in the browser: a soft hum while firing, a stronger
  one for the beam, kicks when targets break, and a big one on the bomb button.
  Off, low or full in the options. Phones playing by touch get a short buzz on the
  big moments (Android; iPhones don't allow it).
- **Touch:** drag anywhere to steer (the ship follows your finger 1:1, up to its
  speed, so your thumb never covers it), it fires while you steer, a second finger
  focuses, and there are bomb and pause buttons.
- **Keyboard:** arrows or WASD, Z fire, Shift focus, X bomb, Esc pause, M sound.
- **The live tuning panel.** Pause to change speed, focus ratio (stepping through
  0.45, 0.50, 0.55, 0.60, 0.67, 0.75), blend frames, touch sensitivity, stick mode
  and rumble. The current values show beside the playfield, keys 1 to 6 change
  them while you fly, and everything is saved (in the browser too).
- **First sounds**, synthesized like Shinjuku's: a soft, throttled shot, the beam's
  hum, a hit tick, breaking targets, and a "not yet" for the bomb button (the bomb
  arrives in v0.5).
- Debug: `PG_AUTO=1` autopilot, `PG_SCENE=shots:<n>`, `PG_SHEET`, `PG_WAV`,
  `PG_MENU`, `PG_FRESH`, and in debug builds F1 hitboxes, F2 grayscale, F3 quarter
  speed, F4 frame step, F9 sprite sheet.

### Performance
- 500 player shots: 60.0 fps at 1080p, 0.40 ms of CPU per frame (2,000 shots: 0.80 ms).

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
