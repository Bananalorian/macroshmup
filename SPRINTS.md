# PILGRIM — Sprint Plan

*Working title, taken from your ship sheet (TYPE-07 "PILGRIM"). Rename it when you name the repo.*

A vertical bullet-hell shmup in Rust + macroquad: **Stage 1 and its boss, built to ship.**
Same setup as Last Train to Shinjuku. Everything is generated in code, GitHub Actions
builds the browser version for Pages plus Windows, macOS and Linux downloads, and every
sprint ends in a tagged version.

> SOMEWHERE BEYOND THE EVENT HORIZON. A SMALL SHIP, A LONG WAY FROM HOME.

**How to use this file**
- Claude Code works one sprint at a time, top to bottom, and stops at the end of each one for your review. Say "Sprint N" or "continue".
- Every "Done when" check is proven with a test, a screenshot or a bot run, never just stated.
- Anything marked *proposal* is waiting on your call. Your notes override this file.

---

## 1. Pillars

### Contrast
The idea everything hangs on, taken straight from your two reference images.
- **Value:** a warm black field with cream highlights. The brightest pixels on screen are always enemy bullet cores.
- **Color:** only two hues exist, red and teal, and only on the Pilgrim, the boss, enemy bullets and explosion fringes. Everything else is engraved monochrome.
- **Energy:** a slow, calm, engraved background against a violent, fast foreground. Breathers between barrages.
- **Scale:** a tiny ship against a black hole and a cathedral-sized boss.

### The ship is instant
No inertia, ever. It moves on the frame you press, it's fast, and the focus mode gives precision when you need it.

### Fair chaos
Boghog's rules throughout: readable bullets, lanes, telegraphs, generous recovery. When you die, you should know it was your mistake.

### Spectacle that never hides a bullet
Particles, shaders and the RGB split all sit under the bullets and ease off when the screen gets busy. Housemarque describes the same goal for Nex Machina: a balance between spectacle and readability.

---

## 2. Art bible

### Palette (sampled from your PILGRIM sheet)

| Token | Hex | Use |
|---|---|---|
| `VOID` | `#080907` | the warm black behind everything |
| `CREAM` | `#DAD7C3` | highlights, text, hull lights; pushed to `#F4F1E6` for bullet cores |
| `ASH` | `#7F7F71` | light hull tones |
| `SLATE` | `#54564C` | mid tones, enemies |
| `SOOT` | `#2B2B28` | dark tones, outlines on light shapes |
| `DUSK` | `#363531` | panel lines, rules |
| `SIGNAL` | `#F83C30` | red: enemy bullet rims, the Pilgrim's running lights, boss highlights |
| `TEAL` | `#32DFB3` | teal: the Pilgrim's engines and trails, shards, graze sparks, the hitbox dot |

The black hole image uses the same family: disk highlights around `#DFDBD1`, space around `#10110C`.
Its RGB streaks are mostly red, with thin hints of green and blue.

### Color rules
- **Enemy bullets** are the only things with a white-hot core, a colored rim and a black outline. Each pattern has its hue (red, teal, violet, amber, blue; Jason, v0.6.4), so layered patterns read as separate families. Red still marks danger everywhere else: telegraphs and enemy lights.
- **Teal means you.** Engines, trails, collected shards, graze sparks, the hitbox.
- The Pilgrim's red bolts stay (they're on your sheet), but they're drawn translucent (60% or less), streaked and without an outline, so they never read as enemy bullets.
- **The boss** gets red highlights: seams and a core that glow brighter as it charges an attack, so the color doubles as a telegraph.
- **Enemies, background, HUD:** monochrome. Exceptions are red muzzle flashes (telegraphs), enemies' red running lights and hit flashes, and red/teal fringes in explosions.
- Because readability comes from value first, color-blind players can still read every bullet.

### Value budget (verified with the grayscale debug view, F2)

| Layer | Max brightness |
|---|---|
| Background (black hole, stars) | about 30%, auto-dimming toward 15% as the bullet count rises |
| Enemies | bone white, up to cream (about 84%); never as bright as a bullet core (Jason, v0.6.3) |
| The Pilgrim | up to 85% |
| Player shots | 60% opacity or less |
| Enemy bullet cores | 100%, with a 0% outline |

### Draw order, bottom to top (Boghog)
Background → enemies → player shots → the Pilgrim → explosions and particles → shards and items →
enemy bullets (big, slow ones below small, fast ones) → HUD.

### Pixel rules
- Logical canvas **640×360**, playfield **360×360** (square, full height) in the middle, with 140 px HUD panels either side. Integer scaling only (2× at 720p, 3× at 1080p, 4× at 1440p, 6× at 4K), nearest filtering. At 1080p the play area is 1080×1080. (It was 270×360, 3:4, until v0.7.2: your call, for a bigger play area and smaller panels.)
- The desktop game opens full screen; F11 switches to a window and the choice is remembered.
- Everything snaps to the low-res grid, shader distortion included, as Shinjuku's post shader does.
- All art is generated in code at startup, like Shinjuku. A debug key exports every sprite sheet as PNG so you can paint over it in Aseprite; a PNG placed back in `art/` replaces the coded version at build time.

### The Pilgrim
**The split sword** (v0.7.1, from [your sheet](docs/reference/ship/Split-sword%20starship_%20flight%20to%20laser.png); redrawn in v0.8.4 from your [parts atlas](docs/reference/ship/Modular%20sword%20spacecraft%20sprite%20atlas.png) and [firing strip](docs/reference/ship/Closed%20Swordship%20Firing%20Animation%20Strip.png)), 29 × 45 px, bone white like Stage 1's enemies:
- **Flight form:** two blades, each with a 45° bevelled top, an antenna, three columns of white stone, a teal channel near its inner edge with a red heart, and two plates stepping out down its outer edge; a crossbar with an engine nacelle at each end, and a capsule below on struts.
- **Laser form:** focus slides the blades together a pixel a frame, folding their plates and antennas and tucking the nacelles in, until they lock into the sword: your long seamed point, two stepped shoulders, one channel, 15 px across at the base (the closed blades' exact width, so there's no jump). The beam fires from the tip.
- **Built from parts, never scaled:** every frame is put together from the drawn parts. Banking folds the dipping side's plates (the sword's shoulders) and tucks its nacelle in, so no frame ever cuts the white stone.
- **Hitbox:** in the gap between the blades' red hearts (on the single heart in the laser form).
- **Animations:**
  - from your first sheet: idle (4 frames), thrust (4), fire (4), hit flash (4), destroy (6), the twin red bolt (4);
  - added: bank left and right (2 frames each) and the 5 frames from flight to laser.
- The first design (gothic spires, the cross on the spine, wing pods that trailed the hull; 30 × 34 px, sized in Sprint 1 so bullets around the ship stay readable) flew from v0.1 to v0.7.0.

### Explosions and the RGB effect
- **Chromatic aberration** in the post shader sits near zero, spikes with each explosion (scaled by its size and distance), and fades in about 0.3 s.
- **Shards:** pixel debris with 1 px red/teal fringes, like the destroy frames on your sheet.
- **Spectral streaks:** thin red-dominant lines with hints of green and blue trailing fast debris, like the streaks behind the falling figure.
- **Shock rings:** Shinjuku's ring distortion, up to 4 at once.

### The black hole
Drawn by a shader, after your falling-figure image: the disk lensed over the top of the shadow, a thin photon ring,
one side brighter than the other, engraved stipple and halftone dithering in cream, film grain, stars bending around the edge.
It grows across Stage 1 as you approach, and it stays inside the value budget.

### Type
The pixel font from Shinjuku, set in spaced capitals like your sheet: thin rule lines, small labels,
a version stamp bottom-left (`V0.1 // PILGRIM`) and a tagline bottom-right.

---

## 3. Design rules (Boghog's 101, condensed)

**Movement and space**
- No inertia. Normalized diagonals. The same speed on both axes.
- Focus means slower movement plus a concentrated beam, with the speed change blended over a few frames.
- The hitbox sits exactly in the center of the ship and shows while focused.
- Smooth movement comes from visuals: banking matched to speed, afterimages, trails, the blades closing for focus. (Since v0.8.3 a bank never takes the white stone outside a blade's channel, so the dipping side stays clean.)

**Shooting**
- Player shots are fast, long, dense and a little messy. Big shot hitboxes, big enemy hurtboxes, low emitters, so you can hit an enemy you're sitting right under.
- Point-blank play is rewarded: a beam aura (from DoDonPachi) and a point-blank score bonus.
- No power-ups in v1; you start at full power. If they're added later, the gains are mostly visual (Boghog's "lie about power").

**Lives and bombs**
- A death cancels every bullet, then gives generous invincibility so you can reposition.
- A deathbomb buffer: bombing a few frames after a hit still saves you.
- Bombs make you invincible while active, and they work both ways: panic button and boss damage.

**Bullets**
- A bright core beside a dark border (value contrast), in the red family.
- Grouped into lines, rings and chunks. No lone strays. Odd trajectories get trails. Every bullet wobbles a little.
- Patterns are aimed, static or random, layered together. Design in lanes, and never wall off big chunks of the screen.

**Enemies**
- Roles: pressure, area denial, direct challenge.
- Killing fast leaves a cleaner screen; ignoring enemies is dangerous. Each enemy has a defined optimal kill, average kill and slow kill (a safety net, like leaving).
- Popcorn has just enough HP for its job. Big hurtboxes, predictable movement. It's never better to let an enemy leave than to kill it.
- A hit flash and a subtle hit sound on every hit. Explosions much bigger than whatever died.
- A dead zone at the top (no killing what you can't see) and a ceasefire zone at the bottom.

**Level design**
- Flow: zig-zag movement across the whole width, never camping one side.
- The Toaplan pattern on 7 lanes with no edge lanes: big enemies alternate sides, popcorn fills in around them, waves overlap.
- High-HP enemies arrive one at a time, never together. Nothing spawns hugging the edges.
- Intensity rises and falls. Set pieces act as landmarks. The background reacts to the stage.

**Scoring**
- Clear before deep: a simple plan you understand on the first play, with nuance to discover later.

---

## 4. Borrowed from the best

| Game | What we take |
|---|---|
| DoDonPachi (CAVE, 1997) | Tap for spread, hold for a laser that slows you; the laser's point-blank aura; bombs give invincibility; 3 bombs at the start; a tiny hitbox (6×7 px in DoDonPachi) |
| Ketsui | Dynamic enemy design: aggression keeps the screen clean (Boghog's example) |
| Touhou (Embodiment of Scarlet Devil) | The hitbox shown while focused; a 6-frame deathbomb window and 240 frames of respawn invincibility (per the pytouhou reimplementation) |
| Raiden II | Point-blanking the mid-boss (Boghog's example) |
| Danmaku Unlimited 3 | Ship trails that make movement feel smooth (Boghog's example) |
| Nex Machina (Housemarque) | Spectacle balanced against readability, telegraphed spawns, explosions that send out shockwaves |
| Ikaruga | Contrast as a mechanic (black/white polarity). Not planned; see Decisions |

---

## 5. Tech spec

**Stack:** Rust (edition 2024, toolchain pinned to 1.91.1 like Shinjuku), macroquad 0.4.16 with the `audio`
feature, gilrs 0.11 on desktop, and the browser Gamepad API through the JS plugin in `web/template.html`.
No asset files: sprites, font, sound and music are generated in code at startup.

**Loop:** a fixed 60 Hz simulation. Every gameplay timing is in frames. Input is read right before each tick,
the screen renders every display frame, and vsync stays on.

**Determinism:** gameplay uses a seeded RNG; cosmetic particles use a separate one, so effects can never change gameplay.
The same seed with the same inputs gives the same run, which the bot tests rely on (and replays, later).

**Separation:** `sim/` is pure Rust with no drawing, no audio and no macroquad globals, so `cargo test` can play the
stage and the boss headlessly in CI. The sim pushes events (explode, cancel, hit, die) that `render/` and `audio.rs`
consume, the same cue pattern as Shinjuku's `Sfx` queue.

**Performance targets:** a locked 60 fps with 2,000 enemy bullets and 6,000 particles on a mid-range laptop and a recent
phone browser, plus a stress scene at 4,000 bullets. One texture atlas, batched draws, additive particles in a single pass,
no per-frame allocation in hot loops.

**Layouts**
- **Landscape:** the 640×360 canvas, playfield centered, side panels for the HUD (styled after your sheet), the black hole behind everything.
- **Portrait (phones):** the same playfield (360×360 since v0.7.2), so the game is identical on every device, with a HUD strip above and an empty thumb zone below. Touch steering is relative, so your thumb drags in that zone and never covers the ship.

**Modules**
```
src/
  main.rs            window, fixed-step loop, scenes
  tune.rs            every tunable number in one place
  sim/               pure, deterministic, tested headlessly
    mod.rs world.rs player.rs bullets.rs pattern.rs enemy.rs
    stage1.rs boss.rs score.rs rng.rs bot.rs
  render/
    mod.rs post.rs bg.rs particles.rs art.rs font.rs hud.rs
  audio.rs synth.rs  ported from Shinjuku
  input.rs pad.rs    ported from Shinjuku (keyboard, gamepad, touch)
  storage.rs         ported from Shinjuku (localStorage / save file)
  debug.rs           env hooks, debug keys, screenshots
web/template.html  build_web.py  .github/workflows/build.yml  .github/package.py  .github/notes.py
```

**Debug hooks (native)**

| Hook | What it does |
|---|---|
| `PG_SCENE=title` / `stage:<seconds>` / `midboss` / `boss:<phase>` / `sandbox:<pattern>` / `stress` | jump straight there |
| `PG_SHOTS=60,300` | save those frames to `./shots/` and quit (not `/tmp`, so it works on Windows) |
| `PG_AUTO=1` | the dodging bot plays |
| `PG_SIM=200` | headless: 200 seeded bot runs; reports deaths per section and phase, phase kill times, bullet peaks |
| `PG_SEED=<n>` | fixed seed |
| `PG_GRAY=1` | grayscale screenshots for the value check |
| `PG_WAV=1` | dump every synthesized sound to `./shots/sfx/` |

Debug keys (debug builds only): F1 hitboxes · F2 grayscale · F3 quarter speed · F4 frame step · F5 invincible ·
F6 skip to the next section · F7 next sandbox pattern · F8 performance overlay · F9 export sprite sheets.

---

## 6. Starting tunables (`tune.rs`)

| Thing | Start value | Notes |
|---|---|---|
| Move speed | 4.0 px/frame (240 px/s) | crosses the playfield in about 1.1 s |
| Focus speed | 0.55× (2.2 px/frame) | Boghog calls ⅔ a good middle ground; A/B test 0.5 / 0.55 / 0.67 in Sprint 1 |
| Focus blend | 5 frames | |
| Hitbox | 2 px radius, between the blades' red hearts | on the heart in the laser form |
| Graze radius | 14 px | |
| Wide shot | twin center bolts + angled pod streams, every 3 frames, 14 px/frame | |
| Focus beam | 12 px wide, about 1.6× the wide shot's single-target damage | plus an aura, 28 px radius |
| Bomb | 180 frames of invincibility; the cancel ring covers the screen in about 24 frames | |
| Deathbomb window | 8 frames | a little more generous than EoSD's 6 |
| Respawn | bullets cancel 30 frames after the hit; 210 frames of invincibility | EoSD: 240 |
| Stock | 3 lives, 3 bombs per life, refilled on death | DoDonPachi adds 1 per death instead |
| Enemy bullet hitbox | about 55% of the sprite radius | leans in the player's favor |
| Dead zone / ceasefire | top 12 px / bottom 60 px | |
| Flash limit | at most 3 full-screen flashes per second | WCAG 2.3.1 |

---

## 7. Workflow (same as Shinjuku)

- **You:** create the repo (public), set Settings → Pages → Source to **GitHub Actions**, and copy your three reference images into `docs/reference/` so Claude Code can match them.
- **Claude Code** runs on your machine with your own GitHub login, so no tokens get pasted anywhere.
- Every push to `main` rebuilds the browser version on Pages. Every `vX.Y.Z` tag publishes a Release (the browser HTML plus Windows, macOS and Linux zips) with notes pulled from CHANGELOG.md.
- **Every sprint ends the same way:** bump the version → CHANGELOG entry → README and screenshots → commit (`vX.Y.0 Title: why`) → tag → push → confirm CI is green → update MEMORY.md → report and stop.

---

## 8. Decisions (defaults in bold; change any)

1. **Music:** **synthesized in code, like Shinjuku**, or your own tracks dropped into the repo (from Suno or anywhere).
2. **Contrast as a mechanic** (Ikaruga-style polarity): **no, contrast stays visual.** A yes would reshape Sprints 2, 4 and 6.
3. **Power-ups:** **none; full power from the start.**
4. **Boss:** **JAWSTHIC**, your concept (Sprint 6, v0.7.0). The Cathedral proposal is retired.
5. **Shot controls:** **a separate Focus button** (Touhou-style), with a CAVE-style "hold Fire to beam" option (both in the options since v0.8.0).
6. **Continues:** **none (arcade retry)**, plus Boss Practice once you've reached the boss.

---

## 9. Sprints

### Sprint 0 — First light · v0.1.0
**Goal:** the empty game runs everywhere and ships itself.

Build
- Scaffold from Shinjuku: `Cargo.toml` (edition 2024; macroquad 0.4 with audio; gilrs 0.11 desktop-only; the LTO release profile), `rust-toolchain.toml` pinned to 1.91.1 to match CI, `.gitignore` (adds `/shots`), `CHANGELOG.md`, a README skeleton, `MEMORY.md` for session notes.
- CI copied and renamed: `build.yml` (browser → Pages; Windows, macOS universal and Linux builds; tag → Release), `package.py`, `notes.py`, `build_web.py`, `web/template.html` (keeping Shinjuku's touch, gamepad, audio-unlock and localStorage plumbing).
- The fixed 60 Hz loop with an accumulator; the 640×360 low-res target; integer scaling with letterboxing; landscape and portrait layouts; a fullscreen toggle.
- `tune.rs`, the palette constants, the ported pixel font, the code-built sprite atlas.
- `debug.rs`: `PG_SCENE`, `PG_SHOTS`, `PG_SEED`, the performance overlay.
- A placeholder title (PILGRIM and the tagline) and an empty playfield with a three-layer parallax starfield.

Done when
- Native and wasm builds are warning-free and `cargo test` passes.
- CI is green on every job, the Pages link shows the title, and tagging v0.1.0 produces a Release with four downloads.
- Screenshots at 1280×720, 1920×1080 and a 390×844 phone size all show crisp, integer-scaled pixels.

You check: open the Pages link on your phone and your computer.

---

### Sprint 1 — The Pilgrim · v0.2.0
**Goal:** the ship feels instant and fast before anything shoots back.

Build
- The Pilgrim in code, from your sheet: idle, thrust, fire and hit animations, plus bank and focus poses, the engine plume, blinking red running lights and a teal trail.
- Movement: no inertia, normalized diagonals, a sub-pixel position drawn snapped to the grid, clamped to the playfield. Focus slows you over 5 frames and shows the hitbox dot and ring.
- Shots: the wide shot (twin bolts plus pod streams) and the focus beam with its aura. Long streaked sprites, hit sparks, damage values in `tune.rs`. Target dummies to shoot: still, moving and big.
- Feel: afterimages at full speed, wing pods that lag slightly, muzzle flashes.
- Input
  - Keyboard: arrows or WASD to move, Z fire, Shift focus, X bomb (stub), Esc pause.
  - Controller (ported `pad.rs`): D-pad or left stick (snapped to 8 directions by default), A fire, RB/RT focus, B bomb.
  - Touch: relative drag anywhere, auto-fire, a second finger for focus, a bomb button.
- First sounds: port `synth.rs` and `audio.rs`; a soft, throttled shot sound, the beam hum, a hit tick.
- A live tuning overlay for speed, focus ratio and blend frames.

Done when
- Unit tests prove: input moves the ship on the same tick it's read; diagonal speed equals straight speed; there's no acceleration; the focus blend finishes in 5 frames.
- Shots hit a target the ship is overlapping (no dead zone at the muzzle).
- 60 fps with 500 player shots on screen, and a screenshot sheet of every ship animation.

You check: play with keyboard, controller and phone. Is it fast enough? Pick the focus ratio.

---

### Sprint 2 — Danmaku engine · v0.3.0
**Goal:** bullets that are beautiful, readable and fast, plus the tools to design with them.

Build
- The bullet pool: struct-of-arrays, swap-remove, no allocation per frame.
- Bullet types: small orb, orb, large orb, rice, needle. Each has a white core, a red rim, a black outline and a 2-frame wobble. A spawn bloom marks each new bullet, harmless for its first 4 frames. Draw order goes by size and speed.
- Collision: the player's hitbox against bullet cores, graze detection, and a placeholder death (flash and reset).
- The pattern library, frame-based and seeded
  - Patterns: aimed n-way fans, rings, spirals (multi-arm, reversible), streams, random cones.
  - Behaviors: accelerate, slow-pause-aim, curve, split on a timer, pull toward a point.
  - Emitters fixed in place or riding a moving enemy.
- **The bot:** a dodger that tries 9 directions at two speeds over a 12-frame lookahead, using the same deterministic sim. It powers fairness checks from here on.
- The pattern sandbox: cycle patterns (F7), slow time (F3), step frames (F4), show hitboxes (F1), check values in grayscale (F2), count bullets.
- The stress scene: 4,000 bullets.

Done when
- 60 fps at 2,000 bullets on desktop and in a phone browser; stress results logged in MEMORY.md.
- Grayscale screenshots show bullet cores as the brightest pixels on screen.
- Tests: the same seed gives an identical bullet field after 600 frames; no bullet can spawn within 24 px of the player; every horizontal band in the bottom half keeps a gap of at least 12 px (no walls).
- The bot survives each sandbox pattern for 60 s on most seeds.

You check: try the sandbox on desktop and phone. Do bullets read instantly?

---

### Sprint 3 — Light & particles · v0.4.0
**Goal:** the look of your references: shaders, particles, the RGB split.

Build
- The render pipeline: game → low-res target → post shader → integer upscale. The post shader, locked to the pixel grid, does
  - up to 4 shock rings (Shinjuku's ring distortion);
  - chromatic aberration that rests near zero, spikes with explosions and fades in about 0.3 s;
  - bloom from bullet cores, sparks and engines (Shinjuku's cheap in-pass bloom first, a two-pass blur only if phones hold 60 fps);
  - film grain, a light vignette and screen flashes.
- The particle system (pooled, 8,000 max, cosmetic RNG): additive spark streaks, embers, pixel shards with red/teal fringes, rings, spectral streaks (red-dominant with hints of green and blue), stipple dust.
- The explosion kit: small, medium, large and boss sizes, each much bigger than what died, each with its sound (ported `synth::explosion`).
- The black hole background shader (see the art bible). It grows across the stage and dims as the bullet count rises.
- The parallax starfield, with the odd red or teal star.
- Settings stubs: Reduce flashing; Screen shake (off / low / full).
- A rule enforced in code: no camera shake while enemy bullets are live. Shake only happens on bombs, deaths and boss breaks, after the screen has been cancelled.

Done when
- Every effect on at 60 fps with 2,000 bullets and 6,000 particles, on desktop and phone.
- The grayscale check still passes with every effect on.
- A test proves the flash limiter never allows more than 3 full-screen flashes per second.
- A screenshot sheet: each explosion size at 4 moments, and the black hole at the start and end of the stage.

You check: is this your aesthetic? More RGB or less?

---

### Sprint 4 — The special weapon · v0.5.0
**Goal:** the Shinjuku explosion becomes the bomb, along with lives, death and recovery.

Build
- **The bomb** ports Shinjuku's grenade blast (`World::explode`: shake, flash, the shock ring, a temporary light, sparks, embers and debris, the `synth::explosion` boom, here in its heavy variant) and scales it to fill the screen, then adds
  - an expanding cancel ring that turns every enemy bullet it crosses into teal shards that fly to you;
  - heavy damage to everything inside the ring (strong against bosses, never a one-shot);
  - an RGB surge, a brief slow-motion dip, 180 frames of invincibility and rumble.
- **Deathbomb:** a bomb within 8 frames of a hit saves you.
- **Death:** your sheet's 6-frame destroy animation with a shard burst and the RGB split; every bullet cancels; you respawn from the bottom, blinking and invincible, with bombs refilled to 3.
- Lives (3), a placeholder bomb and life display, game over → retry.
- The sandbox gains the bomb and the death loop for testing.

Done when
- Tests prove: the bomb cancels every enemy bullet inside its ring; a bomb on frame 8 after a hit saves you and frame 9 doesn't; invincibility windows are exact; you can't die during respawn invincibility.
- Side-by-side screenshots against Shinjuku's grenade blast show the same character, scaled up.

You check: does the bomb feel like that Shinjuku explosion?

---

### Sprint 5 — Stage 1 · v0.6.0
**Goal:** a stage of about two and a half minutes that flows.

Build
- The enemy framework: spline paths, HP, big hurtboxes, a rate-limited hit flash, deaths sized to the enemy, optimal/average/slow-kill behavior per enemy, the top dead zone, the bottom ceasefire zone, body contact kills.
- The roster: engraved monochrome, with red muzzle flashes as telegraphs.

  | Enemy | Role | Attack | If you ignore it |
  |---|---|---|---|
  | Popcorn drones | obstacles, light pressure | a single aimed shot (some fire nothing) | flies off along its path |
  | Gunships | pressure | aimed 3- and 5-way fans | leaves after about 4 s |
  | Turret buoys | area denial | static rings and spirals | drifts out; stops firing low on the screen |
  | Heavies | direct challenge | layered static + aimed fire; killing one cancels its bullets into shards | leaves after about 8 s |
  | Mid-boss (*proposal:* a derelict hull drifting out of the disk) | set piece | two patterns, breakable weak-point turrets | leaves at 25 s |

- The stage script: data, frame-timed, 7 lanes. Toaplan alternation, popcorn layered around the core spawns, heavies staggered, no spawns in edge lanes.
- The timeline

  | Time | Beat |
  |---|---|
  | 0:00 | Fly-in, the stage title, the first drones (no bullets for 6 s) |
  | 0:10 | Gunships alternate sides with popcorn around them: aimed bullets, side-to-side flow |
  | 0:40 | Turret buoys join: static plus aimed makes lanes |
  | 1:10 | A breather. The disk flares and a debris field drifts by (*proposal:* a tiny tumbling figure with RGB streaks passes in the far background, a nod to your image) |
  | 1:25 | The mid-boss |
  | 1:55 | Heavies one at a time through swarms: the peak overlap |
  | 2:30 | Calm. WARNING. The music drops |

Done when
- The bot clears the stage on at least 90% of 200 seeds with invincibility off; deaths are logged per section.
- Automated checks: nothing spawns within 20 px of an edge; no two heavies spawn within 2 s of each other; no enemy takes damage in the top dead zone.
- Each section lands within 5 s of the timeline.

You check: play it. Where is it boring? Where is it unfair?

---

### Sprint 6 — The boss · v0.7.0
**Goal:** a four-phase boss worth the trip, and a death worth watching.

**The boss: JAWSTHIC** (Jason's concept, approved before the sprint, drawn 100% in code from [his reference](docs/reference/stage%201%20bosss/jawsthic.jpg)). A great white carved from moon rock, a gothic cathedral for a body: lancet windows with red and teal light, a dorsal fin with an arched window, a segmented stone tail, a red diamond eye, and jaws that drop open on a red crosshair. Glitch smears trail its tail. It swims side-on across the top of the screen and glitches round to face the other way ([the boss sheet](docs/screenshots/jawsthic-sheet.png)). The Cathedral proposal ([mock-up](docs/screenshots/cathedral-mockup.png), v0.6.5) is retired.

Build
- The boss framework: several parts with their own hurtboxes and HP, a segmented health bar, a phase timer, a phase-break cancel (bullets turn to shards), a short pause between phases. A timeout ends the phase with no bonus.
- **Intro:** WARNING, then its fin cuts through the disk like a shark's through water, it breaches and roars, the music switches, and it can't be hurt until it's in position.
- **Phases** (each mixes aimed, static and random fire, each built around lanes)
  1. TEETH: aimed 5-way fans from its jaws over slow static rings from its windows. 30 s.
  2. FINS: its pectoral fin and tail fin, turrets with red cores, spin opposing spirals while its jaws fire aimed needle bursts. Breaking a fin removes its spiral (and a quarter of the phase); with both gone, its windows open fire. 40 s. (The turrets hang below and behind the body, so shots can reach them.)
  3. GLITCH: a shifting curtain of rice, each row slipping sideways, a lane drifting through it, with random sprinkles off its tail, plus telegraphed laser sweeps from its jaws (a red warning line for 45 frames, then the beam for 60, sweeping toward your nearer wall). 40 s.
  4. EVENT HORIZON, at 15% HP: a rotating flower and spiral arms that bend toward the black hole, every curved bullet with a trail, and aimed bursts. 35 s. If time runs out, it falls into the hole and you get no kill bonus.
- **Death:** 8–12 chained blasts across the hull with shock rings, slow motion at 0.25, an RGB surge, a final white-out and a full-screen cancel, then the stage clear tally.
- Boss practice: `PG_SCENE=boss:<phase>`.

Done when
- Every phase passes the fairness checks, and the bot survives each phase without bombs on at least 80% of seeds.
- With full power and no bombs, each phase takes 20–35 s to kill.
- Screenshots of the death sequence at 6 moments, holding 60 fps throughout.

You check: play the boss (`PG_SCENE=boss`, or the whole stage). Is it worth the trip? Which phase is dull, unfair or too easy? And give the music a listen.

---

### Sprint 7 — Score & shell · v0.8.0
**Goal:** a complete arcade loop from title to stage clear, with a reason to play again.

Build
- **Scoring** (simple first, depth later)
  - Kill points, doubled for kills within 48 px (a teal ring flashes).
  - Cancelled bullets become shards that fly to you for points.
  - Graze sparks with small points.
  - A capture bonus for each boss phase cleared without dying or bombing.
  - No-miss and no-bomb stage bonuses.
  - One or two score extends.
  - Big flashing numbers for big kills.
- **Grades, arcade style** (your call, added before the sprint): each boss phase ends on S, A, B or C, slamming in with its bonus, and the stage tally ends on a rank, S to D, with a flare.
- **HUD styled after your sheet.** Left panel: score, hi-score, lives, bombs. Right panel: stage name, the palette strip, `V0.x // PILGRIM`, HUMANITY STILL LOOKS UP. Portrait gets a compact top strip.
- **Screens:** title (the black hole, the tagline, press start, a bot-played attract demo after 20 s idle), options (volumes, shake, flashing, always-show hitbox, fire scheme, touch sensitivity), pause, game over, and the stage clear tally ending on SOMEWHERE BEYOND THE EVENT HORIZON.
- **Saves** (ported `storage.rs`): hi-score and options. Boss Practice unlocks once you've reached the boss.

Done when
- The full loop plays with keyboard, controller and touch, and the hi-score survives a restart in the browser and on desktop.
- Tests cover the point-blank bonus, capture bonuses and extends.

You check: does the HUD look like your sheet?

---

### Sprint 8 — Music & mix · v0.9.0
**Goal:** it sounds as good as it looks.

Build
- Music: a stage theme, a boss theme and a title ambience, synthesized or your own tracks (Decision 1), crossfaded between scenes. The bomb ducks the music.
- The remaining sounds: warning siren, phase break, extend, cancel shimmer, graze tick, death crunch, menus.
- A mix pass: shot sounds throttled so they never fatigue, enemy bullets silent, a whoosh telegraph before big patterns.
- Rumble, back with care (removed in v0.6.3: over time it lagged a Bluetooth controller): only on big moments, only when the strength changes, and tested on Bluetooth.

Done when
- Every sound dumped with `PG_WAV=1` has been reviewed, and a 10-minute bot run never clips.

You check: listen with headphones.

---

### Sprint 9 — Tuning · v0.10.0
**Goal:** fair, hard, readable, smooth.

Build
- Balance runs with `PG_SIM`: deaths per section and phase, kill times, bullet peaks. HP and densities tuned in `tune.rs`.
- A readability pass: grayscale screenshots of the 10 densest moments, fixing anything that blends in.
- A performance pass on your phone and the slowest machine you have.
- An accessibility review: flashing, shake, hitbox, fire scheme, touch.
- A bug bash, with a regression test for every fix.

Done when
- Your playtests and the bot data agree, and the CHANGELOG lists every tuning change with before and after numbers.

---

### Sprint 10 — 1.0 · v1.0.0
**Goal:** ship it.

Build
- A README with GIFs and screenshots, release notes, credits, the final version stamp, the 1.0 Release.
- Optional: an itch.io page for the browser build.
- Stretch: replays (seed plus per-frame inputs, watchable from the title).

Done when
- The v1.0.0 Release has every download and the Pages version is live.

---

## Later (outside this plan)
Stage 2 and beyond, power-ups, replays (if not done in Sprint 10), more ships, online leaderboards, rank, a second loop.

## References
- [Boghog's bullet hell shmup 101](https://shmups.wiki/library/Boghog%27s_bullet_hell_shmup_101): the design bible for this project
- [DoDonPachi on Shmups Wiki](https://shmups.wiki/library/DoDonPachi): shot/laser split, aura, bombs, hitbox size
- [pytouhou's player code](https://hg.linkmauve.fr/touhou/file/777544d11578/pytouhou/game/player.pyx): Embodiment of Scarlet Devil's deathbomb window and invincibility timings
- [Housemarque on Nex Machina (Engadget)](https://www.engadget.com/2016-12-03-nex-machina-housemarque-resogun-sequel-interview-preview.html): spectacle versus readability
- [Ten Great Bullet Hell Shooters (USgamer)](https://www.usgamer.net/articles/ten-great-bullet-hell-shooters)
- [Last Train to Shinjuku](https://github.com/Bananalorian/shinjuku): the setup, the synth, and the grenade blast that becomes the bomb
