# MEMORY

Session notes for Claude Code: what shipped, tuning that changed, known issues, what's next.
Newest first.

## v0.6.5 XInput, and the Cathedral mock-up - 2026-10-11

Jason: lag happens only with the controller (keyboard fine); other games are fine with it
over Bluetooth. So: Windows reads Xbox controllers via XInput (raw-dylib xinput1_4,
`pad::native` cfg(windows): connected slots every frame, empty slots once a second, map to
the standard layout); gilrs only on Linux/macOS (Cargo.toml target cfg). Not yet confirmed
on hardware (controller asleep). Browser version still depends on the browser's gamepad code.
He asked for a Cathedral mock-up: `render/cathedral.rs` (native-only for now; shapes with the
foes.rs painter, which is now pub(crate), new STAINED material) and `sheet::export_mockup`
(PG_MOCKUP=1) -> docs/screenshots/cathedral-mockup.png. Waiting on his approval for Sprint 6.

## v0.6.4 Colors (after v0.6.3 feedback) - 2026-10-11

**Jason's calls:** "I don't want every bullet red; should be some teal and other colors"
(CLAUDE.md/AGENTS.md rule 4 and the art bible updated to say so). The game is hard ("probably
a good thing"). Asked whether the drones were removed (no: redrawn as the Clerk in v0.6.3).
Still sees lag after a while in every build, but not in my runs.

**Shipped**: `bullets::Hue` (Red/Teal/Violet/Amber/Blue, Default Red) in Spawn, the SoA and
Gun (`gun(..).hue(Hue::X)` in tune.rs); splits inherit it; stress fill random hues.
`palette::hue_rims(h)` -> (rim, wobble rim); BulletArt.frames [hue][kind][wobble][turn];
bloom ring and trails use the rim. Assignment: aimed singles/needles red, rings teal/blue,
spirals violet, fans and sprinkles amber. PG_PERF interval lines every 30 s + summary on
exit; PG_AUTO auto-retries 2 s after game over / stage end (`Game::retry`).

**Lag investigation**: 10-min soak (scratchpad/soak/soak.ps1, bot + sound + his controller
connected): memory/handles flat, cpu 0.3 ms/frame flat, avg 16.67 ms; sporadic present stalls
(up to 1.6 s) with low cpu = outside the game. No accumulation in the game. Remaining suspect:
his Bluetooth controller link (XInputSetState was 8-62 ms earlier). Asked him to test keyboard
only / USB cable / FPS overlay (pause menu FPS: ON) when it lags, or run with PG_PERF=1.

## v0.6.3 Bone white (after v0.6 feedback) - 2026-10-11

**Jason's calls:** remove rumble altogether for now (he had to restart the PC; he thinks
rumble causes lag over time; revisit later). Not enough contrast: make enemies bone white.
New references in docs/reference/: "Six lunar gothic fodder ships.png" (CLERK, COURIER,
INSPECTOR, USHER, NOTARY, BAILIFF; ignore the names) and "minisboss_stage1.png" (the
mid-boss look). Wants a couple more fodder enemies.

**Shipped**
- Rumble removed: pad.rs has no Gate/motor/effects, gilrs `with_force_feedback(false)`,
  storage/menu rows gone (old saves' `rumble=` is ignored), web template has no
  vibrationActuator/vibrate. Guard test `the_game_never_talks_to_a_controllers_motors`.
- `render/foes.rs` rewritten: sprites from shapes (`P::Poly/Rect/Disc/Ring`, mirrored) with
  materials (PLATE, SIDE, RECESSED, SLOT, CORE, GRILLE, CRATER, LIGHT, IRIS) painted by
  `bone()` (outline, top/left lip, bottom/right shade, regolith speckle via hash, craters,
  iris blades). Palette BONE = 3E3B33/6E6858/9D9682/C4BDA8/DDD6C2 (hi luma 0.84).
  Drone=CLERK 17x17, Courier 17x25, Usher 25x21, Buoy=INSPECTOR 25x25 (4 frames, eye notch
  turns), Gunship=BAILIFF 31x27, Heavy=NOTARY 49x35, Hull 121x105 (+ pod 19x37, nose 13x17).
  Hit flash = red silhouette, alpha by area (0.55 / 0.35 / 0.18). Cyan/green lights from
  the refs drawn red (teal = Pilgrim) - told Jason.
- New kinds Courier (6 HP, spline dive at 3.4, one aimed needle, fire share like drones)
  and Usher (16 HP, Hover enter 35 / stay 70, aimed pair from each prong every 34 frames).
  `EnemyKind::fodder()`. Script cues `Couriers{lane,n,fire}`, `Usher{lane,y}`.
- Derelict: HULL (2400, 112x70); HULL_TURRETS now (dx, dy, w, h): pods (+-44, 22, 16x34),
  nose (0, 38, 12x16), HULL_TURRET_HP 300; guns from pods (+-44, 41), nose (0, 46), iris
  (0, -2), stream (0, 26).

**Verified**: 182 tests; PG_SIM=200 100% clear, derelict median 21.0 s; grayscale: cores
still brightest.

## v0.6.2 Quiet radio (hotfix) - 2026-10-11

Jason: lag in all three builds (exe, cargo run, browser). His controller: Xbox Wireless
over **Bluetooth**; XInputSetState there costs avg 7.8 ms, worst 62 ms. gilrs ff (WGI)
calls SetVibration every 50 ms for every ff device even at zero; the game re-sent the fire
hum every 4 frames. Fix: `pad::Gate` (send only on change / top-up within RUMBLE_REFRESH of
the command's end / explicit off at the end; RUMBLE_TAIL 0.6 s); Windows: gilrs built
`with_force_feedback(false)`, rumble via XInputSetState (raw-dylib xinput1_4) on a motor
thread that collapses a backlog to the latest level; `Pads::stop()` waits for the write.
Frame pacing on his PC was already perfect (real clock, 60 Hz both monitors), so "lag" is
input latency. If it persists: USB cable, or Rumble OFF in the pause menu.

## v0.6.1 The rumble stops (hotfix) - 2026-10-11

Jason's controller kept rumbling (and Chrome/PC felt laggy; restart fixed it before).
Cause: gilrs's ff thread writes the motor level every 50 ms and XInput has no timeout,
so the process exiting mid-rumble leaves the controller buzzing. My PG_SHOTS + PG_AUTO runs
did exactly that with his controller plugged in. Fixes: `Pads::stop()` + 150 ms wait at the
end of amain; `prevent_quit()` so window close goes through the loop; scripted runs force
Rumble Off; web template resets actuators on pagehide / hidden. To unstick a controller by
hand: XInputSetState(slot, {0,0}) (P/Invoke from PowerShell) or just run the game once
(gilrs writes zero while idle). The lag was not found on the machine afterwards (Chrome idle,
NordVPN had a momentary spike); asked Jason where he feels it.

## v0.6.0 Stage 1 (Sprint 5) - 2026-10-11

**Jason's calls (before the sprint):** mid-boss = the derelict hull (a wreck rising out of
the disk, engraved hull, red-cored weak-point turrets that break off in blasts); include the
tiny tumbling figure with RGB streaks in the breather; stage name OUTER DISK ("STAGE 1 //
OUTER DISK").

**Shipped**
- `sim/enemy.rs`: EnemyKind Drone/Gunship/Buoy/Heavy/Hull; `Path` Spline (Catmull-Rom,
  arc-length walked at a speed) / Hover{from, at, enter, stay, sway} (leaves upward at
  LEAVE_ACCEL) / Drift; all pure functions of age (the bot predicts them). `Part`s with
  their own hurtbox + HP (part 0 = body; the hull's 3 turrets); `Arm{part, phase, gun}`
  guns from the pattern library riding the enemy (`Mount::Owner{dx,dy}`, mounts[0] = the
  enemy); `may_fire`: settled, below DEAD_ZONE, above the ceasefire (buoys: BUOY_QUIET).
  Hull phase 2 ("Crack") when its turrets are gone or at HULL_PHASE2. `Foes` has its own
  seeded rng (gun start angles; which popcorn fire).
- `sim/stage1.rs`: SCRIPT (seconds, Cue) on lanes 1-5; `arrivals()` expands it; `Stage`
  clock/queue/section/began/deaths/spawned/killed/over; `skip_to(t)` = next tick is frame t;
  `hurry()` brings the next section forward to SECTION_EARLY (4 s) early when the derelict
  dies; `unfair()` checks edges + heavy gaps (debug_assert in Stage::new and the tests).
- World: `World::stage(seed)`, `stage_at(seed, secs)`, `clock()` (stage time; the hole grows
  on it), `skip_section()` (F6), `bombs_used`. Shots/beam/aura/bomb all go through
  `boxes: Vec<(Target, Rect)>` (Target::Dummy | Foe(i, part); weak points listed first).
  Enemy kill -> Event::Kill{kind}; heavy kinds cancel their own bullets (bullet `owner`) into
  shards. Body contact (shrunk 0.7 body vs hitbox) = struck. Events Kill, PartBreak, Crack,
  Section, Warning, StageEnd. Bomb caps only the hull body at 40%.
- Bot: clones foes too, avoids bodies, aims under the heavy-kind / lowest enemy above it
  (BOT_AIM only with a target, so the sandbox is unchanged). `stage_trial(seed, bombs, shoot)`.
- Render: `render/foes.rs` (art: drone + gunship half-grids mirrored with a shade map; buoy,
  heavy, hull, turret procedural via `engrave()` = rounded fall-off, bayer-stippled tones,
  seams + rivets; crack seams; debris; the figure), enemy drawing with charge-up glow (red
  only, no cream center = no fake bullet core) + muzzle flash, hull emerging tint, sockets.
  Breather: `scene::flare()` -> hole shader Look.z boosts light before quantization;
  debris + figure drawn after the hole. HUD: title menu (STAGE 1 / PATTERN SANDBOX), stage
  panel, title card, derelict bar, WARNING bands, stage-end panel (`end_box`, `ended_tap`).
  main: `Chain` for the derelict's rippling death, `foe_effects` (exhaust, wound smoke,
  flakes). The world freezes behind the stage-end panel.
- Synth `warning()` siren (Id::Warning). PG_SIM -> stage report (+ pacifist line);
  `PG_SIM=patterns:N` for the old one. Enemy sheet in PG_SHEET/F9. 183 tests.

**Tuning (tune.rs, Stage 1 block)**: sections 0/10/40/70/85/115/150, end 160; FIRST_FIRE 6;
DEAD_ZONE 12, CEASEFIRE 60, SPAWN_EDGE 20, HEAVY_GAP 2; HP drone 4, gunship 140, buoy 220,
heavy 1100, hull 2400 + turrets 300 (hang at dy 20/26/20 so shots reach them first);
GUNSHIP_STAY 240, HEAVY_STAY 450, HULL_STAY 1500, HULL_EMERGE 150, HULL_PHASE2 720;
DRONE_SPEED 2.2, BUOY_DRIFT 0.38, DRONE_FIRE_SHARE 0.8; FLARE peak 1.6.

**Verified**: PG_SIM=200 bot 100% clear, 100% no-miss, 0 bombs, kills 89%, avg 47 bullets
(peak 199), derelict median 20.5 s; pacifist 100% (avg 70, peak 200). Sections exactly on the
timeline. CPU < 1.4 ms/frame in the busiest parts; web 60 fps, no console errors.

**Known issues / for Jason**: the bot never dies (perfect foresight), so the 90% bar is
easy; real difficulty needs Jason's play. If it's too gentle: HP, fan counts, ring sizes and
the script are all in tune.rs / stage1.rs. The pause menu's PATTERN row does nothing in the
stage. No music yet (Sprint 8).

**Next: Sprint 6, the boss** (v0.7.0): proposal "the Cathedral"; needs Jason's approval of
the concept before starting. Stage end currently stops at 2:40 with a panel.

## v0.5.0 The special weapon (Sprint 4) - 2026-10-11

**Jason's calls (after v0.4):** the black hole was meant as mood reference only, but he
loves it: keep it, it may become a gimmick later. Super happy with the look. "Always think
particle effects" for every new feature. PILGRIM is going to Steam as a full game (beyond
the Stage 1 slice).

**Shipped**
- Sim (`sim/mod.rs`): `Life` Alive / Struck{left} / Dead{t} / Respawning{t} / GameOver;
  lives 3, bombs 3; `invuln` (+ `invuln_blink` for respawns); `BombBlast` ring radius
  `BOMB_REACH * (t+1)/BOMB_GROW`, cancels bullets inside (`cancel_shards` = teal shards),
  hurts each target once (big ones capped at 40%), active 70 frames; `dip` (24 ticks run at
  half speed in main). Order in tick: bomb input -> advance_life -> move -> hazards -> bomb.
  Hit -> Struck{8}; bomb inside the window saves; else death at hit+8; cancel at hit+30;
  respawn at hit+40 (fly-in 36 frames, no control); invuln 210 from respawn, 180 from bomb.
- Events: Bomb, BombSwept (shake), NoBomb, Struck, Death, Cancel, Respawn, GameOver.
  `hits` counts strikes, `deaths` final deaths. `debug_strike()` for PG_DIE.
- `sim/lives.rs`: the Sprint 4 tests. Bot: deathbombs, bombs when every path collides
  within 2 frames; `trial(.., bombs)` (fairness and PG_SIM run without bombs, lives = MAX).
- Render: ship by life state (red ring when struck, teal blink on respawn invuln, teal
  shimmer on bomb invuln), bomb ring (clipped to the field). Particles: Pixel (shatter =
  `ShipArt::pixels`, every pixel of the ship), Homing teal shards (tick takes the target),
  Smoke, glow, colored rings. Discs/stipple pre-drawn in the atlas (1 blit each).
- HUD: life and bomb icons (atlas 7x7), game-over panel (RETRY / QUIT TO TITLE, taps).
- Sounds: bomb, struck, respawn chime, game over; NotYet = no bombs.
- Perf fix: macroquad creates new GPU buffers mid-frame when a frame needs more batches than
  ever (~17 ms stall). Now `draw_call_vertex_capacity` 16384 / index 24576 and the loading
  frame `prewarm()`s 40 batches. `default_filter_mode` is Nearest. 154 tests.
- docs/screenshots/bomb-vs-grenade.png: Shinjuku built from the scratch clone (screenshot
  path patched to ./shots there) with SJ_STATION=0 SJ_AUTO=1; blast around frame 179.

**Tuning**: DEATHBOMB 8, DEATH_CANCEL 30, RESPAWN_AT 40, RESPAWN_FLY 36, RESPAWN_INVULN 210,
BOMB_INVULN 180, BOMB_GROW 24, BOMB_REACH 460, BOMB_ACTIVE 70, BOMB_DAMAGE 600 (cap 0.4),
BOMB_DIP 24, BOMB_SURGE 4.5 px x 20 frames, DEATH_SURGE 2.0 x 6, SHAKE_BOMB 1.0.

**Verified**: bomb in quiet scene cpu worst 0.73 ms (was 17); in stress 2000 ~4 ms, 60 fps;
web bomb 60 fps, no errors; PG_SIM=200 no bombs still 100% survival on all patterns.

**Next: Sprint 5, Stage 1** (v0.6.0): enemy framework, roster, stage script, the mid-boss.
Remember: particles for every enemy spawn/death/telegraph; black hole as a possible gimmick.

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
