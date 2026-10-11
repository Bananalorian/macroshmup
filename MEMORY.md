# MEMORY

Session notes for Claude Code: what shipped, tuning that changed, known issues, what's next.
Newest first.

## v0.4.0 Light & particles (Sprint 3) - 2026-10-11

**Jason's calls (after v0.3):** loves the patterns; keep all tuning options; hitbox dot stays
ABOVE bullets (standard, Touhou); commit AGENTS.md (done in this sprint). He hadn't noticed
graze sparks: made them bigger (7 teal sparks + a flare beside the hull). Controller pattern
switch = LB (or Start -> PATTERN).

**Shipped**
- Pipeline (render/mod.rs): scene RT -> post pass into a second RT -> HUD drawn on top ->
  integer upscale (+ optional gray material). Title also runs through it.
- `render/post.rs`: POST_FRAG (pixel-locked via gl_FragCoord: shake, 4 shock rings, radial CA,
  8-tap bloom onto dark pixels, vignette, grain; cap = CORE for core pixels, CORE*0.93 for the
  rest), HOLE_FRAG (premultiplied, alpha = max(shadow, light) so it never exceeds its ceiling),
  ADD_FRAG (additive particles). `Fx` state: shocks (max 4, oldest replaced), ca (decay 0.84),
  shake (refuses while bullets live; Off/Low/Full; `calm()` each tick bullets exist), `Flash`.
- In RT passes gl_FragCoord.y = 0.5 is the canvas TOP row (verified by screenshots).
- `render/particles.rs` rewritten: cap 8000; kinds Spark/Ember/Shard/Ring/Spectral/Dust/Dot/
  Cross/Core; two draw passes (normal, additive).
- `render/explode.rs`: Size Small/Medium/Large/Boss from `tune::BLASTS`; CA scales with
  distance to the Pilgrim (CA_RANGE 320); REDUCED_CA 0.35 and no flash with Reduce flashing.
- Sounds: synth `pop` (small), `explosion` light/heavy (medium/large), `boss_blast`; Ids
  BlastSmall..BlastBoss (Break/BigBreak removed). Blast sounds play from main's effects().
- Settings (saved): reduce_flash, shake. View: boom (BOOM TEST size). Menu now 19 rows.
- Starfield: Tint Plain/Red/Teal (4% of the two nearer layers), same luma.
- Readback lesson: RT get_texture_data comes back top-down; `Image::export_png` flips rows
  (expects bottom-up). Compose top-down, flip once when exporting.
- Hooks: PG_FXSHEET, PG_BOOM=<0-3>, PG_INVINCIBLE. 137 tests.

**Tuning (tune.rs)**: BLOOM .14, VIGNETTE .24, GRAIN .032; SHAKE_PX 4, decay .85, death .9;
FLASH_GAP 20 frames, 8 frames long, alpha .35 (drawn under bullets). Hole at (135, 92),
radius 11 -> 30 over STAGE_FRAMES 9000, tilt .18, brightness (0.30 - grain/2) * dim.
BLASTS small/medium/large/boss: shock reach 34/64/110/190 px, CA 0.8/1.6/2.6/3.6 px.
Stress presets also keep 6000 particles + a blast every 40 frames.

**Verified**
- 1080p desktop, every effect on: 2000 bullets + 6000 particles 60 fps, cpu 2.9 ms (with bot);
  4000 bullets 2.7 ms. 500 player shots 60 fps.
- Browser (Edge, RTX 2070) at phone size: stress 2000 + 6000 fx 60 fps, wasm cpu 2.0 ms;
  shaders compile with no console errors. Real phone still unmeasured.
- Grayscale: cores 241 even mid-flash; other max 224; background max 0.297 (fx sheet cells).
- Crisp check passes at 1280/1920/390x844 and the 4x web phone.

**Known issues / notes**
- Death flash is a 35% cream wash for 8 frames; Reduce flashing removes it.
- The hole's grooves still read a little ring-like at full size; ask Jason.

**Next: Sprint 4, The special weapon** (v0.5.0): the bomb (Shinjuku's grenade scaled up),
deathbomb, death, lives.

## v0.3.0 Danmaku engine (Sprint 2) - 2026-10-10

**Jason's calls (after v0.2):** keep every tuning option so he can circle back; teal plume
stays; the bump hit-flash stays. Focus ratio still undecided (default 0.55). AGENTS.md
still untracked (not answered; leave it out of commits).

**Shipped**
- `sim/bullets.rs`: SoA pool (cap 8192, never reallocates), kinds Small/Orb/Large/Rice/Needle,
  behaviors Straight/Accel/SlowAim/Curve/Split/Pull, capsule hit test (`seg_dist`).
- `sim/pattern.rs`: Gun {mount, shape, kind, speed, every, delay, behavior}; shapes Fan/Ring/
  Spiral/Stream/Cone; mounts At/Dummy(i)/Sweep; `Hazards` (emitters + bullets + own rng, hold
  after death, fill for stress). Preset data lives in `tune::SANDBOX` (14 entries, 0 = OFF).
- `sim/bot.rs`: 17 candidates (9 dirs x full/focus) over a 12-frame lookahead of cloned hazards;
  `trial()` for tests and PG_SIM. `sim/fairness.rs`: the Sprint 2 checks.
- World: bullets vs hitbox after the bloom (age >= 4), graze once per bullet (14 px), placeholder
  death (Event::Death, cancel all, reset player, guns hold 60 frames), `invincible`.
- Render: `render/bullets.rs` (draw order, bloom ring, wobble every 4 frames, 8 pre-drawn turns
  for rice/needle, trails on Curve/Pull), turrets with red muzzle telegraph, `Flash` limiter
  (>= 20 frames apart), starfield dims to 50% at 1500 bullets. Atlas is now 512x512.
- Palette: CORE #F4F1E6 (luma .946), EMBER #FF6A50 (wobble rim), INK #000.
- Pause menu = sandbox switches (pattern, autopilot, slow-mo, invincible, hitboxes, grayscale,
  fps; `Settings.view`, not saved) + tuning. `[ ]` / LB / NEXT touch button step patterns.
- `main()` is hand-written now (no #[macroquad::main]) so PG_SIM runs before a window opens.
- Cargo: dev opt-level 1, test opt-level 3 (fairness tests play ~300k frames). 124 tests, ~7 s.

**Verified**
- PG_SIM=200: 100% survival on all 11 gun patterns (47 s on 16 threads). Peaks: PULL 379 bullets.
- Desktop 1080p: 2000 bullets 60 fps, cpu 0.79 ms (1.75 ms with the bot); 4000: 60 fps, 1.39 ms.
  500 player shots still 60 fps / 0.31 ms.
- Browser, phone viewport 390x844 @3x (canvas x4), real GPU (RTX 2070): 60 fps at 2000 and 4000.
  SwiftShader (software GL): ~28-30 fps even with an empty screen, so not bullet-bound.
  A real phone is unmeasured: Jason can check with SHOW FPS in the pause menu.
- Grayscale shot: max value 241 = bullet cores (550 px for 50 bullets), next 214 (cream).
- Web: tapping NEXT x3 goes OFF -> SPIRAL 4/14 (screenshot).

**Known issues / notes**
- The sandbox patterns are gentle on purpose (the bot never dies); real difficulty comes with
  the stage and boss. "Standing still gets hit" test proves they're threats.
- Hitbox dot draws above enemy bullets (treated as HUD).
- A real phone's frame rate hasn't been measured by me.

**Next: Sprint 3, Light & particles** (v0.4.0): post shader, particles, explosions, the black hole.

## v0.2.0 The Pilgrim (Sprint 1) - 2026-10-10

**Shipped**
- `sim/`: `player.rs` (Controls, Tuning, Player: focus blend, no-inertia move, touch drag capped
  at speed, pod lag, bank, history), `shots.rs` (wide volley: 2 bolts + 2 angled pod shots),
  `dummy.rs` (still/moving/big, DPS log, rate-limited flash, respawn), `geom.rs`, World events
  (Volley, Hit, Break, Bump, BombStub), beam (stops at nearest target in its lane) + aura.
- `render/ship.rs`: the Pilgrim as half-grids mirrored (BODY 11x34, anchor = cross arms (10,11);
  POD 7x18, spine col 4). Bank frames made by dropping wing columns + darkening the dipping side.
  Lights ('L','W','M','P','N' chars) are drawn unlit in the atlas and lit at draw time.
- `render/art.rs` is now a shelf packer (512x256 atlas) with silhouettes for flashes/afterimages.
- `render/particles.rs` (minimal: sparks, dots, rings; cosmetic RNG; cap 4096), `render/scene.rs`.
- Ported from Shinjuku: `pad.rs` (gilrs + web plugin, `pg_*`), `synth.rs` (core + tick/hit/
  explosion/select), `audio.rs` (throttle, beam loop), `storage.rs` (now holds Settings).
  New: `input.rs`, `menu.rs` (pause = tuning panel), `rumble.rs`, `sheet.rs` (native only).
- Web: `pg_vibrate` added to the template's plugin (phone buzz).
- 93 tests.

**Tuning (tune.rs)**: speed 4.0, focus 0.55 over 5 frames, pods ±11 (focus ±7), follow 0.6,
max lag 3; volley every 3 frames; bolt 14 px/f, 18 long, dmg 3; pods 13 px/f at 4/9 deg, dmg 1.5;
beam 12 wide, 3.2/frame (1.6x); aura 28 px, 1/frame. Dummy HP 120/80/1500.
Rumble: fire (0, .07), beam (.10, .18) every 4 frames; break (.35,.45,140ms); big (.8,.7,380);
bomb (1,.8,450); connect (.3,.3,160). Sounds: shot gap 95 ms, vol .16.

**Verified**
- 500 shots: 60.0 fps at 1080p, cpu 0.40 ms/frame (2000: 0.80 ms) (`PG_SCENE=shots:500 PG_PERF=1`).
- Browser via Playwright/Edge: keyboard (menu change saved to localStorage), a fake gamepad
  (stick, A, RB, B; 31 rumble calls recorded incl. connect, fire hum, beam hum, bomb kick), touch
  drag + second-finger beam on a 390x844 @3x phone. No console errors.
- Crisp check on all screenshots (title/stage at 1280, 1920, 390x844 and the web phone at 4x).

**Known issues / decisions pending**
- Jason to pick the focus ratio (pause menu, presets 0.45-0.75).
- The thrust plume is teal (art bible) though the sheet's thrust frames show a red-white plume.
- Bumping a dummy flashes the hull (a stand-in so the hit animation shows; Sprint 5 makes contact lethal).
- Bank frames are subtle; touch steering cap = ship speed (fast flicks get clipped, no lag).
- `AGENTS.md` (Jason's Codex copy of CLAUDE.md) is untracked; not committed.
- Native rumble untested on real hardware by me (gilrs did detect a controller on this PC).

**Next: Sprint 2, Danmaku engine** (v0.3.0): bullet pool, bullet types, patterns, the bot, sandbox, stress.

## v0.1.0 First light (Sprint 0) - 2026-10-10

**Shipped**
- Scaffold from Shinjuku: `Cargo.toml` (crate `pilgrim`, edition 2024, macroquad 0.4.16 + audio,
  gilrs 0.11 desktop-only, LTO release), `rust-toolchain.toml` (1.91.1 + wasm target), CI, `build_web.py`,
  `web/template.html` (JS plugin names renamed `sj_*` -> `pg_*`, localStorage key `pilgrim`).
- `clock.rs` fixed-step accumulator (vsync snap, max 4 catch-up ticks). `render/layout.rs` integer-scale
  fit: landscape 640x360 with the field at x=185, portrait 270x480 (HUD strip 24 px, field, 96 px thumb zone);
  portrait when the screen is taller than wide. `high_dpi: true` so phones get physical pixels.
- `render/`: `art.rs` (one 256x32 atlas: font glyphs, px, plus, sparkle), `bg.rs` (3-layer starfield,
  cosmetic seed, positions a pure function of the frame), `hud.rs` (title + stage panels), `font.rs`, `palette.rs`.
- `sim/`: `World { seed, frame }`, `rng.rs` (SplitMix64). A test bans macroquad from `sim/`.
- `debug.rs`: PG_SCENE (title, stage:<s>), PG_SHOTS, PG_SEED, PG_GRAY, PG_SIZE, PG_PERF; F8 perf overlay.
- 37 tests.

**Tuning (tune.rs)**: star layers density 14/6/2 per 10k px², speed 0.10/0.28/0.70 px/frame,
value 0.11/0.19/0.29 (bg cap 0.30); text tracking 2; title scale 3, tracking 6.

**Verification tools**
- Crispness: a script checks every s x s block of a screenshot is one colour (all 6 native shots
  and the 4 browser shots pass).
- Browser: Playwright (`playwright-core`, `channel: 'msedge'`, swiftshader flags) in the scratchpad loads
  `dist/index.html`. Plain `msedge --headless --screenshot` silently does nothing on this machine.
- Locally Python is `py` (no `python` on PATH).

**Known issues / notes**
- Shinjuku's CI failed every tag run at the Pages step (environment protection rejects tags). Fixed
  here: Pages deploys only from `main`.
- `PG_SCENE` midboss/boss/sandbox/stress print "arrives in a later sprint" and start at the title.
- Not yet ported: input.rs, pad.rs, audio.rs, synth.rs, storage.rs (Sprint 1 / 7). The web template
  already carries their JS plumbing.

**Next: Sprint 1, The Pilgrim** (v0.2.0): the ship, movement, shots, input, first sounds.
