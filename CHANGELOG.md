# Changelog

Every version is one build sprint. Newest first. Versions are tagged in git, and
each tag publishes a GitHub Release with downloads.

## [0.6.1] - 2026-10-11 - The rumble stops

### Fixed
- **A controller could keep rumbling after the game closed.** An Xbox-style
  controller holds the last strength it was given, and the desktop game could quit
  in the middle of a rumble (closing the window, or an automated test run ending),
  leaving it buzzing until something else reset it. The game now zeroes the motors
  and waits for that to reach the controller before it exits. The browser version
  resets the rumble when you close or hide its tab.
- Automated runs (`PG_SHOTS` screenshots and performance checks) never rumble a
  real controller.

### Checked
- Closing the window mid-stage exits in under 0.3 s, motors zeroed first.
- In the browser, with a recording stand-in controller: 15 rumbles while firing,
  then a reset the moment the page goes away.

## [0.6.0] - 2026-10-11 - Stage 1

Sprint 5: OUTER DISK, a stage of two and a half minutes, with enemies, a mid-boss
and a breather.

### Added
- **STAGE 1 // OUTER DISK.** The title now offers STAGE 1 or the PATTERN SANDBOX.
  The stage runs to a timeline:
  - 0:00 the fly-in: the stage title types itself in, the first popcorn drones arrive,
    and nothing fires for 6 seconds;
  - 0:10 gunships alternate sides with popcorn around them, their aimed fans pushing
    you back and forth across the screen;
  - 0:40 turret buoys drift down among them: static rings and spirals that make lanes;
  - 1:10 a breather: the black hole's disk flares, a field of debris drifts past, and
    a tiny figure tumbles into the hole trailing red, green and blue streaks (a nod
    to your image);
  - 1:25 the mid-boss, **the derelict**: a gothic capital ship rises out of the disk,
    its three red-cored turrets firing aimed fans and rings. Break them (they hang
    below the hull, so your shots reach them first) and the hull cracks open, its
    seams burning red, into a reversing spiral with needle bursts. It leaves at
    25 s. Destroy it and every bullet it fired is cancelled, the burning wreck
    shudders as blasts ripple across it, then it goes up in a boss-sized blast
    ([the sequence](docs/screenshots/derelict-death.png)), and the next section
    comes forward a little;
  - 1:55 heavies, one at a time, through swarms: slow rings of big orbs, needle
    bursts from both towers, a random sprinkle. Killing one turns all of its bullets
    into teal shards;
  - 2:30 calm, and WARNING: red bands crawl across, a siren sounds;
  - 2:40 the end of the stage for now, with your kills, misses, bombs and grazes,
    and RETRY or QUIT TO TITLE. The boss arrives in v0.7.
- **The enemy framework:** spline flight paths (or fly in, hover and leave, or a
  slow drift), big hurtboxes, a hit flash at most every 5 frames, deaths sized to the
  enemy (they shatter into their own pixels), engine exhaust, smoke once they're
  damaged. Nothing can be hurt in the top 12 px of the screen, nothing fires from
  the bottom 60 px, and crashing into an enemy is as bad as a bullet. Every enemy has
  an optimal, average and slow kill (in `sim/enemy.rs`): ignore one and it fires
  until it leaves.
- **Telegraphs:** every enemy gun glows red as it charges, then flashes as it fires.
  Popcorn that's going to shoot has a dim red eye until it does.
- The black hole now grows on the stage clock and flares during the breather
  (brighter, never past the background's value budget).
- The derelict's health bar at the top of the playfield, with a notch per turret.
  The side panel shows the section, the stage clock, kills and misses.
- Sounds and rumble: the WARNING siren; blasts sized to every kill, broken turret
  and the derelict's crack; rumble for the big ones (not for popcorn).
- Debug: `PG_SCENE=stage:<seconds>` now starts Stage 1 that far in,
  `PG_SCENE=midboss` goes straight to the derelict, F6 skips to the next section,
  `PG_SIM=200` plays the stage (`PG_SIM=patterns:200` runs the sandbox patterns),
  and F9 / `PG_SHEET=1` also saves the enemy sheet.

### Changed
- The bot simulates the enemies as well as their bullets, steers clear of their
  bodies, and drifts under whatever it should be shooting.
- Every bullet remembers which enemy fired it.

### Checked
- `PG_SIM=200`: the bot clears the stage on 200 of 200 seeds (the bar is 90%), with
  no deaths and no bombs. It kills 89% of the enemies; the derelict goes down in a
  median 20.5 s. Every section begins on the timeline. A pacifist bot that dodges
  but never shoots also survives, facing 70 bullets on average and up to 200.
- Tests prove: nothing spawns within 20 px of an edge or in an edge lane; no two
  heavies arrive within 2 s; no enemy takes damage in the top dead zone; nothing
  fires from the ceasefire zone; crashing kills; a heavy's bullets cancel when it
  dies; shots reach the turrets before the hull; the derelict can't be hurt while it
  rises and a bomb takes at most 40% of it; no bullets in the first 6 s; the bot
  clears the stage with every section within 5 s of the timeline; every enemy sprite
  sits in the 35-75% value band and the breather's extras in the background's 30%.
- 60 fps through the derelict and the heavies (CPU under 1.4 ms a frame), and 60 fps
  in a browser, where the title menu, pause and quit all work.

## [0.5.0] - 2026-10-11 - The special weapon

Sprint 4: the Shinjuku grenade becomes the bomb, with lives, death and recovery.

### Added
- **The bomb** (X, B or the BOMB button): Last Train to Shinjuku's grenade blast
  scaled up to fill the screen. A boss-sized explosion with smoke, a hot glow, a
  screen-wide shock ring, a flash and a heavy boom, then:
  - a teal cancel ring that sweeps the whole screen in 24 frames, turning every
    enemy bullet it crosses into teal shards that fly to you;
  - heavy damage to everything it passes over (big targets take at most 40% of
    their health, so a bomb is never a one-shot);
  - an RGB surge, a brief slow-motion dip, full rumble, and 180 frames of
    invincibility (with a teal shimmer). The screen shakes once the ring has
    swept it clean.
- **The deathbomb:** when a bullet hits you, a red ring flashes round the ship and
  you have 8 frames to bomb and survive.
- **Death:** the Pilgrim shatters into its own pixels (every pixel of the sprite
  becomes a particle) with a large blast and an RGB split. Every bullet on screen
  pops 30 frames after the hit. The next ship flies up from the bottom, blinking
  teal and invincible for 210 frames, with bombs refilled to 3.
- **Lives:** 3 ships, shown with bombs in the side panel (and the phone's top
  strip). After the last one: GAME OVER, with RETRY or QUIT TO TITLE.
- Sounds: the bomb (the grenade's heavy boom over a deep sweep and a rush of
  air), the hit alarm, a rising chime as the next ship arrives, a falling phrase
  for game over, and a click when you're out of bombs. Rumble for all of them.
- The bot bombs too: inside the deathbomb window, or when every path is about to
  be hit. (The fairness checks still run it without bombs.)
- Debug: `PG_BOMB=<frame>`, `PG_DIE=<frame>`, and the worst CPU frame in `PG_PERF`
  summaries.

### Fixed
- **A one-frame hitch on the first big explosion.** macroquad made new GPU buffers
  mid-frame whenever a frame needed more draw batches than ever before (a bomb in
  a quiet scene did). Batches are now 4x bigger and the loading frame creates
  enough of them up front: the bomb's worst frame went from 17 ms to under 1 ms.

### Checked
- Tests prove: the bomb cancels every bullet inside its ring as it spreads; a bomb
  on frame 8 after a hit saves you and frame 9 doesn't; bomb invincibility is
  exactly 180 frames and respawn invincibility exactly 210; you can't die during
  it; bullets cancel 30 frames after a hit; three deaths end the game.
- [Side by side with Shinjuku's grenade](docs/screenshots/bomb-vs-grenade.png):
  the same blast, scaled up.
- A bomb inside the 2,000-bullet stress scene: 60 fps, worst CPU frame 4 ms on
  desktop; 60 fps in a browser.

## [0.4.0] - 2026-10-11 - Light & particles

Sprint 3: the look of the reference art: shaders, particles, the RGB split.

### Added
- **The black hole**, drawn by a shader after the falling-figure image: the
  shadow, a thin photon ring, the accretion disk seen nearly edge-on with its far
  side lensed up over the top, the approaching side brighter, engraved stipple
  shading, and stars smeared into arcs round the edge. It grows as you approach
  (over the stage's 2.5 minutes), dims as bullets crowd in, and stays inside the
  background's 30% value budget. It's on the title too.
- **A post pass**, locked to the pixel grid: up to 4 shock rings that push the
  picture outward, chromatic aberration that rests at zero, spikes with each blast
  (harder the closer it is to you) and fades in about 0.3 s, bloom from bullet
  cores, sparks and engines, a light vignette and film grain. The HUD sits on top,
  untouched.
- **Particles**, up to 8,000: additive spark streaks, embers, chips of debris with
  red or teal fringes, rings, spectral streaks (red with hints of green and blue,
  like the falling figure's), stipple dust and little sparkle crosses.
- **Explosions** in four sizes (small, medium, large, boss), each much bigger than
  what died, with a hot core, a shock ring, an aberration spike and its own sound.
  The big ones flash the screen. Target dummies now go out with a medium or large
  blast; the placeholder death with a large one.
- **Grazing is easy to see**: seven teal sparks and a teal flare where the bullet
  passed.
- **The odd red or teal star** in the starfield.
- **Options:** REDUCE FLASH (no full-screen flashes, gentler aberration) and SHAKE
  (off, low, full), both saved, plus BOOM TEST to set off any explosion size.
- `PG_FXSHEET=1` saves every explosion size at 4 moments and the black hole at the
  start and end of the stage. `PG_BOOM=<0-3>` and `PG_INVINCIBLE=1` for screenshots.

### Rules, enforced in code
- No screen shake while enemy bullets are live: it only happens on deaths (and
  later bombs and boss breaks) after the screen's been cancelled, and stops the
  moment bullets reappear.
- At most 3 full-screen flashes a second, and the flash draws under the bullets.
- Only bullet cores can reach the brightest value on screen: however many sparks
  pile up, everything else is capped a step below.

### Checked
- Every effect on, 2,000 bullets and 6,000 particles: 60 fps on desktop (2.9 ms of
  CPU with the bot flying) and in a browser at phone size (2.0 ms).
- Grayscale with blasts going off: bullet cores are the brightest pixels (241),
  even mid-flash; nothing else passes 224. The background tops out at 0.297.

## [0.3.0] - 2026-10-10 - Danmaku engine

Sprint 2: bullets that are beautiful, readable and fast, and the tools to design with them.

### Added
- **Enemy bullets:** small orbs, orbs, large orbs, rice and needles. Each has a
  white-hot core (the brightest pixels on screen), a red rim and a black outline,
  and wobbles between two frames. Rice and needles point along their flight.
  New bullets bloom in for 4 frames and can't hurt you while they do. Big, slow
  bullets draw under small, fast ones, and all of them draw above everything
  except the HUD.
- **A bullet engine** built for speed: a fixed-size pool that never allocates
  during play. Hitboxes are about 55% of each bullet's size, and needles and rice
  hit along their length.
- **A pattern library:** aimed fans, rings, spirals (multi-arm, reversing),
  streams and random cones, from guns that are fixed, sweep side to side, or ride
  the moving target dummy (break the dummy and its gun falls silent). Bullets
  can accelerate, slow down, pause and re-aim at you, curve, split into rings, or
  get pulled toward a point. Curving and pulled bullets leave short trails. Guns
  flash red just as they fire.
- **The pattern sandbox:** 13 patterns plus OFF, cycled with `[` `]`, LB on a
  controller, or the NEXT button on a phone. The pause menu now has the sandbox
  switches too, in every build: pattern, autopilot, slow-mo, invincible,
  hitboxes, grayscale and an FPS readout. The side panel shows the pattern,
  the bullet count, hits and grazes.
- **Grazing:** a bullet's core passing within 14 px of the hitbox throws teal
  sparks, ticks, buzzes the controller lightly, and counts once per bullet.
- **A placeholder death:** a hit flashes the screen (never more than 3 times a
  second), cancels every bullet, and puts you back at the start while the guns
  hold off for a second. The real death and respawn arrive in v0.5.
- **The bot:** a dodger that plays through the same sim, trying 9 directions at
  two speeds over a 12-frame lookahead. `PG_AUTO=1` or AUTOPILOT in the pause
  menu lets it fly. `PG_SIM=200` runs 200 seeded games per pattern headless and
  prints a report.
- **Stress scenes:** STRESS 2000 and STRESS 4000 in the sandbox (`PG_SCENE=stress`).
- The starfield dims as the bullet count rises, down to half brightness at 1,500 bullets.

### Checked
- No bullet can appear within 24 px of the player; every 4 px band of the bottom
  half keeps a gap of at least 12 px (no walls); the same seed gives the same
  bullet field; bullets are harmless for exactly their first 4 frames. Tests for all.
- The bot survived every pattern for 60 seconds on 200 of 200 seeds. Standing
  still gets you hit on most patterns, so they're real threats.
- 60 fps at 2,000 bullets (1.75 ms of CPU with the bot flying) and at 4,000
  (1.39 ms) on desktop; 60 fps at both in a browser at phone resolution.
- Grayscale: bullet cores are the brightest pixels on screen (value 241, next
  brightest 214).

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
