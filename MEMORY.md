# MEMORY

Session notes for Claude Code: what shipped, tuning that changed, known issues, what's next.
Newest first.

## v0.10.0 Boss attacks & the Slam (Sprint 9) - 2026-10-13

**Jason, after v0.9.0:** "bosses don't constantly just fire. they have phases and patterns...
the boss just fires constantly... add this to sprint 9 and continue. the game doesn't play
like a shmup just yet." Research (in SPRINTS.md Sprint 9): CAVE bosses cycle attacks of
5-10 s; Touhou never repeats a nonspell twice; telegraphs; move between attacks; escalate;
breathers (Boghog: constant intensity wears players out).
- **Boss attacks:** `Attack {guns, wind, fire, rest, at, laser}` in sim/boss.rs; data
  `JAWS_ATTACKS` in tune.rs (3 per phase), JAWS_WIND 45, JAWS_REST 60, JAWS_HURRY 0.5 (past
  half a phase's health). Boss: `attack`, `beat`; `attack_now() -> (Attack, Beat::Wind/Fire/Rest)`;
  guns' emitters restart at each attack's first shot (gun delays now count from it); only
  the attack's guns fire (`firing()` filters broken fins / enraged); charges() glow every
  muzzle during the wind (new `others` [pectoral, belly, tail]); mouth_due() only for the
  attack's mouth guns; drift() goes to the attack's spot (`at`, facing-relative; the next
  one's while resting; sway x0.35 while firing). The laser is P3's 2nd attack
  (`start_laser`); JAWS_LASER_EVERY and next_laser removed. Turns and lunges only start
  in a rest.
- **The Slam:** src/sim/slam.rs (impl World): `Slam::{Dash, Aim}`, World.rush/slam/reticle;
  Controls.slam (key C/L, pad X = button 2, touch SLAM button). SLAM_DASH 7, REACH 170,
  DMG 260, BOSS_DMG 400, CANCEL_R 36, AIM 36 (main loop runs 1/SLAM_SLOW=4 speed while
  aiming), LINKS 3, BOUNCE 34, INVULN 60; RUSH_FILL by kind [0.08..1.5], RUSH_BOSS 0.00025
  per damage, RUSH_START 1; dummy breaks fill it too. Events SlamDash/Slam (hit-stop
  HITSTOP_SLAM 4, synth::dash whoosh, crunch+hit). Render: reticle brackets + dashed
  line, sword form + afterimages while dashing; HUD RUSH gauge row; controls lists.
- **Bot:** slams non-fodder targets in reach when rush >= 1, chains while a reticle shows;
  never panic-bombs while invulnerable (that wasted ~1.4 bombs a run).
- PG_SIM: cleared 100%, 0 deaths, 0 bombs; phases 20.5/22.6/24.4/21.3; derelict 100%
  (18.5 s); score 3.66M; rank S 100%. Pacifist bullets avg 50 peak 180.
- Screenshot script: pilgrim-forms frames 304-311 now; new boss-attacks (boss:1 seed 11
  40,150,300,385,470) and slam-chain (stage:115 seed 3 PG_RUSH=3 1138..1174).
- New hook PG_RUSH=<n>.

**Decision asked (still open from v0.9.0):** the rank thresholds (the bot is S every run now).
**Next:** Sprint 10 (Style) when he says go.

## v0.9.0 Feel (Sprint 8) - 2026-10-12

**Jason:** "look at ... Lunar gothic mechanical shark with closed jaw.png. when the mouth opens and
closes [it] cannot look disjointed ... needs to be a single shark. what you have is good, just
refine it. also the edges need to feel like small particles are being taken off of it by the
blackhole. like the particles that you absorb after a bomb. go" (go = Sprint 8).
- **The mouth:** tools/trace_jawsthic.py splits FANGS off BODY (everything on/under the GUMS_UP
  arch) and traces LIP and LOWER_LIP from the closed picture (CLOSED_SCALE 0.165/1.33, offset
  CLOSED_AT (26,13): its pixel (x+26, y+13) is the body's (x,y)); the slit is found near SLIT,
  the lower lip runs down to the shut jaw's gum line (GUMS_DOWN turned by JAW_SHUT about HINGE
  (71,54)) plus CHIN. Its background mask closes cracks first (Max/MinFilter 25) or the flood
  leaks along the slit. jawsthic.rs `Mouth::frame(k)`: jaw, body, fangs slid up by their own
  height x (1-back), lower lip slid down and turned with the jaw, lip slid up; back =
  open/LIPS.1 (0.4), lips slide LIPS.0 14 px. Open frame identical to v0.8.8.
- **Specks:** `stripped()` drawn UNDER the shark (the hole is behind it); `strip_path` lifts
  (JAWS_STRIP_LIFT 0.4 of life) then q^2 into the hole with a sin swirl (JAWS_STRIP_SWIRL 26);
  JAWS_STRIP ([90,110,140,180],280), life 64; a third off the tail (`tail_rim`).
- **Alive (sim/boss.rs):** `bend(lx)` (JAWS_BODY_WAVE (1.2, 2.6, 0.018)) is in `at()`, the tail
  hangs off bend(root) and leaves along its slope; `behave()`: flinch (JAWS_FLINCH keep 0.95,
  threshold 120, 8 px back, 4 up, cd 60; pain from hurt(), a fin break = MAX; no recoil while
  lasing), chomps (JAWS_CHOMP; only fit in P3/breaks), lunges (JAWS_LUNGE_EVERY (300,180),
  JAWS_LUNGE (6 back, 18 wind, 16 reach, 6 drop, 7 strike); no laser/turn may start mid-lunge;
  Event::BossBite -> synth::chomp). `look` = player; render `gaze()` repaints the eye socket,
  the pupil rolls white while `biting()`. Body drawn with `put_bent` (strips of columns).
- **Juice:** main.rs `hitstop(events)` -> game.stop skips stage ticks (HITSTOP [0,2,4,6],
  HITSTOP_BOSS 6); `Particles::impact` silhouettes (IMPACT [2,3,4,4]); `audio.kill` = blast +
  synth::crunch (VOL_CRUNCH, CRUNCH_BY_SIZE); foes `jolt()`; chips in `Particles::hit`.
- **Beam:** Beam.hit_box; scene.rs `draw_wrap`/`wrap_path`, bigger flare; `beam_spray`.
- **Fewer bullets:** tune.rs guns thinned (rings x0.7 n, spirals/cones every x1.4, curtain
  every 24, speeds +10-20%). PG_SIM: bot avg 80->52, peak 234->162; pacifist 101->66,
  235->166; phases 24.4/24.2/26.4/23.3; score 3.28M; rank S 88 A 12 (was S 30 A 70).
- Screenshot script: pilgrim-forms now frames 333-340 (the bot's focus moved); boss practice's
  world clock starts at ~9450, so PG_BOMB needs that offset (PG_BOMB=9650 ~ screen frame 200).

**Decision asked:** retune the rank thresholds now that the bot gets S 88%, or leave for Sprint 15.
**Next:** Sprint 9 (the Slam) when he says go.

## The UX plan - 2026-10-12

After v0.8.8 Jason laid out the game's direction: a shmup you play like Hades (and a bit of
Devil May Cry): fast, juicy, twitchy, arcade; lots of fodder, fewer bullets, overpowered
then overrun; a Slam (dash-strike chains, 3 links, slowdown to aim at the next target); a
style rank; augments and a shield; CRT shaders; a Zero Ranger-like title; WARNING as a
lottery; a living boss ("more movement... feel alive"); the laser flaring and wrapping
round what it hits; more things to shoot; a pause button. Written into SPRINTS.md as
Sprints 8-13 (Feel, the Slam, Style, the Swarm, Augments & shield, Arcade shell), all
*proposal*; Music & mix, Tuning and 1.0 moved to 14-16. Decided: Feel first; the Slam on
pad X / key C (bomb stays on B / key X); a rush gauge filled by kills; the ship's size kept,
revisited in the Swarm sprint.

**Next:** Sprint 8 (Feel) when he says go.

## v0.8.8 Back on track - 2026-10-12

**Jason:** "you just made it worse. it was better with the ontrack file. revert to that... make
the thick tail even and the tail fins actually should not move seperately. please get this
correct. I am having anxiety with this." He showed the v0.8.6-era traced tail as the good one.
So v0.8.7's drawn tail AND its cleanup pass are reverted (`git checkout v0.8.6` of
tools/trace_jawsthic.py, jaws_px.rs, jawsthic.rs, tune.rs), then:
- `even_tail()` in the tool: tail columns EVEN_FROM 45..EVEN_TO 79 get a straight top/bottom
  from the tail's height at 45 to the body's height at its cut (body col 7, mapped via
  ROOT_IN_BODY (8,36) / ROOT_IN_TAIL (80,46)); fills take the nearest stone (never lights),
  seam columns carry 'x', the old traced outline pixels are replaced, new outline + lit top.
- jawsthic.rs: `tail: [Sprite; 2]` = the whole traced TAIL (fin included), pivot JOINTS[0];
  drawn column by column with `along_tail` (clamped; fin columns ride joint 5) so the fin never
  turns on its own. Phase 2 fin flash draws the fin columns red.
- Kept from v0.8.7: `Boss::tail_boxes` (laser hits the tail) + test; FAR_FIN_AT (24,19).
- JAWS_PHASES damage P2 0.8, P3 0.8, P4 0.42 (0.40 timed out 1/200; 0.44 too quick).
  PG_SIM: 3.10M, extends 100/100, A 70 S 30, phases 24.2/24.4/28.1/23.2, 100% destroyed.

**Lesson:** when he likes a version, change the least possible on top of it; a full redraw
of something he already liked cost a release and his patience.

**Next:** his list of other fixes before Sprint 8.

## v0.8.7 One tail - 2026-10-12

**Jason** (/loop): "the thick part of the tail is not one piece. keep the tail movement... the
back fins are not congruent either. you might need to redo just the tail and tail fins. 2.
also clean up the edges... i want the whole shark to feel like it is in the game and not just
bolted on." Earlier: "my laser goes right through that part of the tail."

- **Tail drawn, not traced** (tools/trace_jawsthic.py `draw_tail`): body cut at BODY_CUT 53
  (behind the arcade), TAIL_JOINTS body-local [(52,47),(40,44),(28,40),(17,35),(7,30),(-3,24)],
  TAIL_HALF (20, 6), RINGS, FINLETS; writes PLATES (pivot = root), FIN (pivot = its joint)
  and `TAIL_JOINTS` (in PLATES) to jaws_px.rs. The fin is a crescent polygon with plate seams,
  a dark web and lights. Tones: thin rim 5, then 4/3/2/1 down, hairline cracks, speckle.
- **Bent by column** (jawsthic.rs): `along_tail(c)` (clamped) puts each plates column at
  the sim's joint polyline y; the fin turns on its joint (turn sets centered on TILT 0,
  FIN_REACH 0.84). No more per-plate rotation (that stepped at joints).
- **Sim:** JAWS_TAIL lengths ~12 with ART [0.245,0.322,0.427,0.464,0.54,0], TILT 0, BEND
  [0.02,0.035,0.06,0.08,0.11,0.17], TAIL_ROOT (-32,2), BODY (22,-2,108,52), HOME (200,160),
  TURN_SHIFT 41. `Boss::tail_boxes()` (plates + the fin outside phase 2) pushed as
  Target::Boss(0) in build_boxes, so shots and the beam hit the tail. Damage: P2 0.74, P3
  0.825, P4 0.395 (0.39 timed out 1/200; P4's S cutoff 24.5 s is a knife edge).
- **Cleanup** (`cleanup` in the tool) on every traced part: lone pixels, lone dark specks,
  spikes/notches, outline, `lit_edges` top-left rim; dither 18 -> 10. FAR_FIN_AT (24,19).
- PG_SIM: 3.06M, extends 100/96, A 78 S 22, phases 24.3/25.4/28.0/24.4, 100% destroyed.

## v0.8.6 Stars and particles - 2026-10-12

**Jason:** "the tale on the jawsthic-vs-art.png isn't lined up to look like a tale"; the tail
glitch "looks good. need to have some on the top and bottom. like the black hole is taking
away sub atomic particles"; "the bullets coming out of my ship don't line up correctly";
boss bullets "look different, add some slower moving ones. some that look like stars... take
some inspiration from the guide".

- **Tail:** his tail art rises toward the fin (centerline y 47 at the root to 33 at the fin),
  so level cuts made a staircase. Now `JOINTS` in jawsthic.rs are on the drawn centerline
  [(80,46),(72,47),(64,46),(55,43),(46,40),(39,34)] (tail traced at 0.165); the sim's
  `JAWS_TAIL_ART` (each plate's drawn slope) + `JAWS_TAIL_TILT` 0.25 + wave give each joint's
  angle; the renderer turns plate i by `ang - ART[i]` (turn sets centered on TILT). JAWS_TAIL
  lengths from the joints; TAIL_ROOT (-76,-9) inside the body's thick rear; the tool's
  `trim_cap` clears body columns < 7 (the body's end cap) and seams the cut.
- **Shots:** BOLT_GAP 3 (blade tips), POD_X 9 / POD_X_FOCUS 6 / POD_GUN_Y -20 (antenna
  tips), the pod lag removed (POD_FOLLOW/POD_MAX_LAG gone), pod streams start low on the line
  through the antenna tip; POD_ANGLES [6, 11.5] (were [4, 9]) to cover the same at range
  (a derelict weak-point test caught the narrower spread). Test ties ship.rs tips/guns to tune.
- **Particles:** `stripped()` + `JawsFrames.rim` (top/bottom silhouette, no jaw underside);
  JAWS_STRIP ([26,34,44,58], 96), life 56, pull 0.6 sideways toward HOLE_X (the hole sits
  behind the fish, so pulling at it dragged them across the stone, invisible). Drawn as
  smear-style chips and scanlines plus a few sparks.
- **Bullets:** Kind::Star (BULLET_SIZE (5.0, 0.0), spins by STAR_SPIN 5 from its fired
  direction) and Kind::Diamond ((3.2, 3.0)); 7 kinds now. Boss: P1 jaw fans Diamond, windows
  1/3 Star 0.85 amber/blue; P2 enraged windows Star 1.0 amber; P3 tail Star 0.9 every 18;
  P4 flower Star 1.9 (1.6 timed out 4%). PG_SIM: 100% clear/no-miss, boss 100%, 2.98M,
  extends 100/95, A 80 S 19, derelict 92% (94 before). 240 tests.

**Next:** his list of tuning and fixes before Sprint 8.

## v0.8.5 The new JAWSTHIC - 2026-10-12

**Jason** (via /loop): "redo the boss. the design. a major redesign. function can remain the
same... redraw like the image... the absolute best you can... keep the glitches and the
explosions and the phases. the health bar should go down from left and right equally down to
zero and then the next phase starts. put a diamond in the middle... start with green and each
phase the gem changes color. last phase being red." Also "yeah we can try them again. the led
lights" -> ship channels lit steady in flight (TEAL 0.85, no flow; flow only while beaming).

**Boss art:** `tools/trace_jawsthic.py` (Pillow) cuts 6 parts from
"docs/reference/stage 1 bosss/Right-facing shark boss sprite atlas.png" (largest 8-connected
piece, holes kept), LANCZOS-shrinks (premultiplied) at per-part scales (body/jaw 0.165, tail
0.15, dorsal 0.14, near pec 0.17, far pec 0.15), restores contrast (GAIN 1.3 about 150),
keeps cracks (ink coverage > 0.3 of the full-size block), snaps to the bone RAMP with Bayer
dither (EDGES 40,75,115,172,205), outlines; hand pass: eye diamond at (110,23), jaw crosshair
unpainted, pectoral stump; writes `src/render/jaws_px.rs` (Grid rows of letters + GLASS
window pixels). carve.rs is deleted. jawsthic.rs: decode -> RotSprite-lite (`epx` twice,
sample at 4x) `turns()`, `mirrored()`, `layered()`; body frames = jaw (turned JAW_SHUT -0.55
* (1-open) about HINGE) under the body; tail cut at CUTS [78,68,57,47,37,28] (+3 px
overlap); tail turn sets centered on JAWS_TAIL_TILT 0.18 (`turn_of(a, rest, reach)`,
clamped; a crash and a debug overflow here were caught by the screenshot batch and the debug
tests). Texture 2048x1024. Glitch smears off the tail (`smears`, JAWS_SMEARS ([4,6,9,14],
24), life 24). Rig (body pivot = traced body (84,45)): HINGE (-13,9), TAIL_ROOT (-78,-7),
TAIL [(10,22),(11,20),(10,17),(10,15),(9,13)], DORSAL_AT (-14,-14), PECTORAL_AT (-17,18),
FAR_FIN_AT (32,30) (render only), WINDOWS [-24,-13,-2,8] y 0, EYE (26,-22), MOUTH (54,-3) +
JAWS_MOUTH_OPEN (-16,17)*jaw (was MOUTH_DROP), DORSAL_CORE (-17,-26), BELLY (1,29), BODY
(4,-2,128,52), PECTORAL (-30,28,30,26), TAIL_FIN (14,24,58). JAWS_HOME (224,136),
TURN_SHIFT 88 (the fish is longer on the tail side). JAWS_PHASES damage P3 0.9 (others
unchanged) to keep the v0.8.0 calibration: PG_SIM=200 median 2.98M, extends 100/96, rank A
82 S 18, phases 24.6/26.1/27.8/24.7 s, 100% clear, 0 deaths.

**Bar:** `hud::jaws_gauge` (gem index, full, flash); each phase's bar drains both ends in to
a GEM_R 5 diamond at its center; in the calm (`JawsView.calm`, JAWS_REFILL (0.2,0.75)) the
gem turns the next color with a flash and the bar refills outward; the phase name follows the
gem. `palette::GEM` = green 3FD45A, yellow F2C230, orange F5822A, SIGNAL.

His composed picture (from chat) is saved as "docs/reference/stage 1 bosss/JAWSTHIC
composed.jpg". 239 tests.

**Next:** he said "before sprint 8 there is a ton of stuff i want to tune, fix, add"; wait for
his list.

## v0.8.4 Your sprites - 2026-10-12

**Jason** sent new ship art (docs/reference/ship/: "Modular sword spacecraft sprite atlas.png":
two blades, two nacelles, capsule, two struts; "Closed Swordship Firing Animation Strip.png": 6
laser-form frames) and asked "will this work?". A straight shrink to game size is mud (tested),
so he OK'd a pixel-by-pixel redraw. ship.rs is now parts-based: `blade(plates, antenna)` and
`sword([plates; 2])` generate pixels (`Px = (x, y, letter)`; letters o 0-4 c v h b k s n),
`NACELLE`/`POD` are grids, `compose(form, bank)` lays them on a `Canvas`, `paint` colors them,
`lights` finds the light letters. `FORM` table: inner -3/-2/-1/0/0, plates 2/1/0/0/2 (form 4 =
the sword's shoulders), antenna 4/3/1/0/0, nacelle -11/-11/-10/-10/-10. Banking (`rolled`):
the dipping side folds `steps` plates and tucks its nacelle in by `steps`; nothing is cut.
TIP -22 (was -21), FOOT 6, BAR 8 (was 9), CORNER TIP+7, PLATES [-7, 0], SHOULDERS [-4, 2],
NACELLE_TOP 7, POD_TOP 10. Speckle is hashed in each part's own coordinates so it moves with
the part. 29 x 45 flight; sword 15 wide at the base = the closed blades (test). Channels still
dark in flight (his art shows them lit; asked him). Committed his reference moves (ship/,
"stage 1 bosss/") and relinked SPRINTS/CLAUDE/AGENTS. New screenshot pilgrim-vs-art.png
(before | his art | now, flight then laser). 238 tests.

**Next:** Sprint 8 (Music & mix, v0.9.0) when he says go.

## v0.8.3 Clean sides - 2026-10-12

**Jason:** "the ship teal outlines when moving are gone now!!! and the muddy sides have not
changed ... that right side is not clean because the side loses the white ... when the ship is
together making the laser, the sides are clean." So v0.8.2 was a misread: he LIKES the
afterimages (restored from v0.8.1 byte for byte: scene pass, `draw_afterimages`,
`AFTERIMAGE_MIN 0.9`, `Player.history`, `Player::focused`, the sheet demo, the test). The real
problem all along (v0.7.3 "flickering lines", v0.8.1 "led lines", v0.8.2 screenshot): `roll()`
dropped the 2 white face columns outside each flight-form channel on the dipping side. Fix in
ship.rs `dropped()`: the outer face keeps `FACE_KEEP 2` columns of stone; then the rail gives a
column off its INNER EDGE (a mid-rail column left the tip as a lone pixel), switching below
`POD_ROWS.0` to the column just outside the capsule (drops are per row now: `type Drop =
[i32; ROWS]`, `roll_x(x, y, drop)`), keeping `RAIL_KEEP 2`; then the nacelle's plain columns.
Forms 2-4 (laser) roll exactly as before. Test `rolling_keeps_the_white_down_the_dipping_side`.
The dipping side is still a tone darker (`dip` in `paint`), which he called clean on the laser.
Channels stay dark in flight (v0.8.1). 237 tests.

**Then (done in v0.8.4):** redraw the Pilgrim from his new sprites in `docs/reference/ship/`
("Closed Swordship Firing Animation Strip.png": 6 laser-form frames; "Modular sword spacecraft
sprite atlas.png": blades, nacelles, capsule, struts; plus the split-sword ref, which he moved
there along with jawsthic.jpg into `docs/reference/stage 1 bosss/`; those moves are still
uncommitted and SPRINTS.md links the old paths). A straight shrink to ~20x46 turns to mud, so
the plan he OK'd is a pixel-by-pixel redraw at game size from his art: parts slide for the
transformation and the roll (no mesh, no resampling), his 6 frames become the laser form's
firing cycle without their baked flames and tip star; keep my thrusters, beam, bullets and
afterimages.

## v0.8.2 No ghosts - 2026-10-12

**Jason** sent a screenshot: the "led lines on both sides of the ship" that interfere when
moving left and right were the **afterimages** (teal silhouettes at history 2/4/6 at full speed),
not the channel dashes. Removed: scene's afterimage pass, `ship::draw_afterimages`,
`AFTERIMAGE_MIN`, `Player.history`/`HISTORY`, `Player::focused`, the sheet's demo. (v0.8.1's dark
channels in flight stay.) Lesson: when he describes a visual, ask for or look at a screenshot
before guessing. His game was running from target/release while I worked, so I built into
`target/work` (CARGO_TARGET_DIR) and ran the screenshot batch from there (scratchpad
shots_work.sh); never kill his running game.

## v0.8.1 Full size, dark blades - 2026-10-12

**Jason:** "return the ship to the previous size. and remove the led lines on both sides of the
ship. interferes with animation when going right and left." The v0.7.3 shrink is undone (he
asked for it, then preferred the old size): ship.rs is back to v0.7.2's numbers (SIZE 33x49,
TIP -21, SHOULDER -8, FOOT 7, ARCH -15, NACELLE_ROWS (8, 14), POD_ROWS (11, 21), KEYS as
v0.7.2, NACELLE_W/POD_W 3, pod window rows 13-16, strobes at 16), keeping the named rows and the
roll's nacelle-column fallback. Channel lights: `None` unless `pose.beaming` (then flowing
TEAL_HOT); the lock's charge band still runs. Nacelle/capsule window dots kept (asked him).

## v0.8.0 Score & shell (Sprint 7) - 2026-10-12

**Jason's calls:** the score sprint with arcade grades: "s/a/b/ etc. flare when the score
happens. think arcade like". (He called it "sprint 6"; it's Sprint 7.)

**Shipped**
- `sim/score.rs`: Grade (S A B C D), Award (Kill, Blank, Part, Shards, Grade(p, g),
  BossKill, Graze), Score {points, extends, grades, tally}, Tally, kill_points (x2 within
  POINT_BLANK 48 px), phase_grade (S clean inside GRADE_S_TIME 0.7 of the limit, A clean, B
  not, C timeout), phase_bonus (CAPTURE x2/x1/x0.2/0), rating (kills 30, grades 10/7/4/0
  each, destroyed 10, no miss 15 -5 a death, no bomb 5 -2 a bomb), rank (RANKS 90/75/55/35).
- World: `score`, `award()` (Stage 1 only; Event::Points, extends -> lives up to
  MAX_LIVES 6 + Event::Extend), kill/part/fin/boss-kill points, shards from heavy kills and
  boss breaks (not bombs), silent graze points, `grade_phase()` once per phase (Boss.took[]
  records the frames each phase took), `finish_stage()` at BossGone: no-miss/no-bomb
  bonuses added straight to points (no extend), the tally.
- Tunables (tune.rs "scoring"): KILL_POINTS [500, 750, 1.5k, 7.5k, 10k, 40k, 200k], PART 15k,
  SHARD 50, GRAZE 100, FIN 25k, BOSS_KILL 300k, CAPTURE [150k, 200k, 200k, 300k], NO_MISS
  500k, NO_BOMB 300k, EXTENDS [800k, 1.8M], BIG_POINTS 20k; TALLY_STEP 18 / COUNT 16 /
  SCORE 108 / RANK 148 / LINE 188 / DONE 228; ATTRACT_AFTER 1200; DEMO_LEN 3600; HOLD_BEAM 18;
  VOLUME_DEFAULT 8; VOL_TICK/SLAM/EXTEND.
- `render/popups.rs`: kill numbers (cream; teal point-blank; 2x flashing for big), shards,
  `draw_grade`/`slam_letter` (scale 12 -> 6, streak, ring, S teal glow, glint), EXTEND; drawn
  under the bullets. `commas()`, `grade_color()`.
- HUD: left panel score (`score_digits`, 8 digits, zeros DUSK) / hi-score / lives / bombs /
  stats / controls; right panel voyage / stage / tuning / strip / stamps. `HudView` gained
  score, hiscore, end_t, record, tally, demo, ranked (`best()` shows this run's score only
  when it counts). Tally (`stage_end`) on end_t; game over with score; DEMO overlay; menus
  generalised (`menu_box(lay, n)`, `menu_tap(lay, items, ..)`, `menu_panel`); `TitleItem`,
  `title_items(practice)`, `options()`.
- main.rs: `Run` (Stage / Practice / Sandbox / Demo), `Scene::Options`, Title `idle`,
  `begin()`, `start_boss`, `start_demo`, `to_title`, `attract()` (demo after 20 s idle, any
  button / end / a minute ends it), `new_record()`, `tally_cue()`, `focused()` (HOLD TO BEAM),
  BossArrive in a Stage run unlocks Boss Practice. PG_AUTO retries after the tally.
- storage: music, sound, hitbox (always/focus), fire (hold/button), hiscore, boss. **read() and
  write() do nothing under cfg(test)** (see below).
- synth: tally_tick, slam, extend_jingle (Id Tick, Slam, Extend); audio `volumes()`, `gain()`.
- debug: PG_SCENE=options; PG_SIM stage report prints score / extends / grades / ranks.

**Incident:** a test (`only_a_full_stage_run_sets_the_hi_score`) called Settings::save and
overwrote Jason's real `%APPDATA%\.pilgrim\save.txt` (his options and tuning reset to
defaults, hiscore=1000). I set hiscore back to 0, made storage a no-op in tests, verified the
file's timestamp doesn't change across `cargo test`, and told him to re-set his options.
Lesson: anything that touches the user's disk must be inert in tests.

**Measured:** PG_SIM=200 score median 2,971,850 (2.75M-3.27M), extends 100% / 98%, rank A 84%
S 15%, phase grades P1 A, P2 S 92%, P3 S 58%, P4 S 37%; stage still 100% clear / no-miss.
Full bot run seed 7 from the top: stage ends at screen frame 16156, rank A, NEW RECORD.

**CI note:** CI runs `cargo test` in debug (overflow checks on); the first v0.8.0 push failed on
`lives + 1` overflowing the bot's endless lives (u8::MAX). Fixed, and the unreleased tag moved.
Always run plain `cargo test` before pushing, not just `--release`.

**Open:** listen to the new sounds; human difficulty and score feel (Sprint 9); red or teal
wide shot? Next: Sprint 8, Music & mix.

## v0.7.3 A fifth smaller - 2026-10-12

**Jason:** "make the ship about 20% smaller while keeping the detail; everything about the
function is perfect"; the flowing channel dashes while shooting "mess up the animation moving
left and right", but he likes the flicker on the sword when it closes. Then go to the score
sprint (he called it "sprint 6"; it's Sprint 7) with S/A/B grades that flare, arcade-like.

**Shipped:** ship.rs rows are constants (TIP -17, SHOULDER -6, FOOT 5, ARCH -12,
NACELLE_ROWS (6, 11), POD_ROWS (9, 17), `pub BAR`), KEYS rescaled (f0 inner -2 core -5 outer
-9 nacelle -10 corner -11 antenna (-7,3); f4 inner 0 core 0 outer -7 nacelle -9 corner -8),
NACELLE_W/POD_W 2, canvas 27x41. `dropped()` takes a plain nacelle column when the face has
only one. Channel dashes static unless `pose.beaming` (then flow every frame, TEAL_HOT);
windows a steady 0.7; `flicker()` removed. main.rs trails from `ship.nozzles[2]`, lock sparks
at `ship::BAR`. Screenshot batch: scratchpad/shots.sh.

## v0.7.2 Full screen (square playfield, broad laser form) - 2026-10-12

**Jason's feedback on v0.7.1:** "love the ship!!", but the laser form "gets too small, or
close together"; "full screen 1080, bigger player area, top to bottom with smaller panels".
Asked the width (AskUserQuestion with mock-ups): he picked **360 wide, square**.

**Shipped**
- FIELD_W 360 (CLAUDE.md non-negotiable 5 updated), panels 140 px (420 at 1080p),
  PORTRAIT_W 360 (phones drop from 4x to 3x). Field-relative positions moved with it:
  HOLE_X/START_X/BOT_HOME x 180, sandbox mounts x4/3, dummies (83/180/261), DUMMY_SWING
  107, popcorn spline sweeps x4/3 (stage1::flight), boss intro pass uses FIELD_W,
  JAWS_HOME (195, 165), JAWS_SWAY 16, P3 curtain n 23. Lanes already scaled with FIELD_W.
- HUD landscape panels: STAGE 1 / OUTER DISK on two lines, rows shifted +10, flags at
  track 1, `panel_stamps` (LEFT_STAMP / RIGHT_STAMP broken lines), rule at ch-42, controls
  end above it. Touch buttons centred in the right panel.
- Full screen by default: Settings.fullscreen (default true, saved as `fullscreen=`),
  `start_fullscreen()` in window_conf (not wasm, not `debug::scripted()` = any PG_ env var);
  F11 / Alt+Enter toggles and saves. Not tried on Jason's display (didn't take over his
  screen); asked him to confirm.
- Laser form: KEYS now end at outer -8 (sword 17 px wide), nacelle -11, per-key `corner`
  (-13 flight .. -10 laser, a sharper point), a 2 px `M::Bevel` on broad faces.
- Screenshot crops for 1280x720 shots: the field is x 280..1000 now (strip.py etc).

**Measured:** PG_SIM=200 100% cleared/no-miss, kills 79% (was 86), derelict 94% (was
100), Jaws 100%; boss:200 medians 24.9/29.5/29.0/25.0 s, P2/P4 2% timeouts, P3 1% deaths;
patterns 100%. 215 tests.

**Open:** red or teal wide shot? Ship size OK? Is the wider field too easy (Sprint 9 is
balance)? Sprint 7 (Score & shell) next.

## v0.7.1 Split sword (the Pilgrim redesigned) - 2026-10-12

**Jason's call:** a new ship from his reference `docs/reference/Split-sword starship_ flight
to laser.png` (two bone-white blades with teal channels in flight, closing into one sword
that fires a teal laser): "the ship works the same but redesigned to this, along with the
animation". Done out of sprint order as a point release; Sprint 7 is still next.

**Shipped**
- `render/ship.rs` rewritten. `KEYS[FORMS=5]`: per frame (left half, mirrored) the blade's
  inner edge, channel core, outer edge (and above the shoulder), nacelle center, antenna
  (column, height), strut start. Rows from the hitbox: TIP -21, CORNER -13, SHOULDER -8,
  FOOT 7, channel ARCH -15 / STEP -1; crossbar rows 8-10; nacelles (half width 3) rows 8-14,
  nozzle 15; capsule (half width 3) rows 11-21, nozzle 22. Canvas 33x49 centered on the
  hitbox. `layout(k)` -> material map (M: Face, Rail, Wall, Core, Groove, Seam, Crack,
  Metal, Window, Nozzle, Antenna, Strut, Red; Piece: Blade, Bar, Nacelle, Pod, Strut).
- Banking: `dropped()` removes 1 or 2 plain Face columns from the middle of the dipping
  side's outer face (never Core/Wall/Window/Red/Antenna/Seam/Groove); `roll_x` maps points.
  The dipping side paints a tone darker.
- `paint()`: bone (foes::BONE) with selective outlines: ink on bottoms, outer sides and
  piece seams; a BONE[2] rim on top edges (so the tip shows on black); no ink down the
  blades' inner edges. Face BONE[4] stippled, bevel 3, rail 4, walls BONE[0], core black.
- Lights are found in the painted map: Channel (teal dashes flowing up, 0.8 idle, faster
  and TEAL_HOT while beaming, a charge band running up on lock), Heart, Beacon, Keel,
  Strobe, Window. Frame points: nozzles (flames hang below), tips (bolt muzzles), guns
  (antenna tips: pod muzzles).
- `Pose { form (= focus blend), locked, ... }`; `ShipArt::frame(form, bank)`,
  `form_index`, `draw_silhouette`, `TIP` (the beam's nose). Player gained `locked`
  (frames since the blend hit 1; cosmetic). main.rs: trails from the nacelle nozzles,
  `Particles::lock` (2 sparks each way off the crossbar + a Cross at the tip) on locked == 1.
- Beam is teal (TEAL / TEAL_HOT, now in palette.rs). Life icon redrawn. Afterimage alpha
  0.14/0.08/0.04. Atlas 1024x512. `blit_flip` removed (pods were its only user).
- Sheet (PG_SHEET / F9) is 440x344 with the transform row; CPU review sheet:
  `cargo test --release pilgrim_sheet -- --ignored` -> shots/pilgrim_frames.png.
- Every screenshot retaken (recipes: see the v0.6.4 `shot()` helper and the v0.7.0 boss
  strip commands; breather = stage:70 frame 200, warning = stage:150 frame 90).
  docs/screenshots/pilgrim-forms.png = boss:2 seed 3 bot, frames 509-516.

**Measured:** 214 tests; PG_SIM=50 100% cleared/no-miss, JAWSTHIC destroyed in all;
PG_PERF boss:2 60.2 fps, cpu 0.35 ms avg.

**Open questions for Jason:** should the wide shot's red bolts go teal to match the laser?
Is the bigger ship (31x44 vs 30x34) OK, or should it shrink? Sprint 7 (Score & shell) next.

## v0.7.0 JAWSTHIC (Sprint 6, the boss) - 2026-10-11

**Jason's calls:** the boss is JAWSTHIC, his concept ("name jawsthic? lol"), 100% drawn
in code from his reference (saved to docs/reference/jawsthic.jpg: a 3/4 view, bone-white
cracked-stone great white, gothic lancet windows, red/teal lights, segmented tail with
glitch smears, red diamond eye, red crosshair in the jaws). "These bosses are very
important: spend time and get it right." The Cathedral proposal is retired (render/
cathedral.rs deleted; its mock-up png stays in docs).

**Shipped**
- `render/carve.rs`: the boss painter. Parts = shapes (Poly/Disc/Ring/Line) with
  materials (Hull, Seam, Recess, Slot, Red, Teal, Tooth, Gum, Mouth, Vein, Pit, Frame,
  Pipe, Shade) in part coordinates; `carve(part, Pose{angle, flip})` maps each pixel back
  into the part, so any angle/facing is crisp; `Shape::turned(hinge, a)` for the jaw.
  Light is screen top-left; `Form::Barrel` (normals from the outline) or `Flat`; per-plate
  bevel (lip/drop at seams), bounce + bottom rim light, sparse grain in part coords,
  6-tone RAMP (26241F..DDD6C2, top 0.84 luma) with a narrow Bayer band (DARK 0.16,
  BRIGHT 0.96).
- `render/jawsthic.rs`: the model (body+jaw per opening, 5 tail plates, caudal, dorsal,
  pectoral, pectoral stump) and `JawsArt` (own 1024x1024 texture: body x7 jaw frames, tail
  turns every 0.05 rad to REACH [0.1..0.84], paddle +-0.2, both facings; carved in 66 ms).
  `draw()`: glitch bands (canvas rows shifted), surface clip (intro/fall), window charge
  glows, turret cores, eye pulse, crosshair, cracks (phase 3+), additive red hit flash,
  red/teal ghosts; pieces blown off in the death (`Boss::blown`). `draw_laser()`.
  Review: `cargo test --release jawsthic_sheet -- --ignored` (CPU sheet) and PG_MOCKUP=1
  (real renderer, 6 states) -> shots/jawsthic_sheet.png.
- `sim/boss.rs`: `Boss` (Act Intro/Fight/Break/Dying/Falling/Gone; facing; turn; hp with
  phase floors; fins [pectoral, tail fin]; jaw; swim; laser; guns per phase from
  tune::JAWS_GUNS with Muzzle Mouth/Window/Pectoral/Tail/Belly; `enraged` guns fire once
  both fins are gone). `tail()` is the swim kinematics shared with the art. BOSS_ID =
  u32::MAX owns its bullets. World: `boss`, `slow` (quarter-speed ticks), spawn at JAWS_AT
  (BossArrive), `boss_aftermath` (break -> cancel_owner into shards; down -> cancel all +
  slow; gone -> stage over + StageEnd), Target::Boss(p), bombs, laser + crash hits,
  `boss_at(seed, tuning, phase)`, `skip_section` breaks a phase during the fight.
- Patterns: Shape::Curtain{n, spacing, gap, period, jitter}, Shape::Flower{n, petals,
  turn}; Pull bullets are swallowed within 4 px.
- Stage: SECTIONS has 8 (JAWSTHIC at 157.5 s), WARNING_SECTION = 6; STAGE_END is gone
  (the stage ends with the boss); `Stage.boss_called`.
- HUD: `JawsView` bar (segments by share, fills during the intro, phase name, timer red
  under 10 s); the tally line JAWSTHIC DESTROYED / LOST TO THE HOLE + PHASES BROKEN n/4
  CLEAN n.
- Effects: smears (tail fin + plates, more when glitching/hot), drips, wake on the intro
  fin pass, splash on the breach, roar shock/surge, break bursts, fin shatter, death
  pieces, final blast + `Fx::whiteout` (70 frames, 0.9 peak / 0.45 reduced). Effects keep
  ticking behind the stage-end panel now.
- Audio: `synth::boss_theme` (16 bars, 150 BPM, E Phrygian, seamless via wrap-around
  mixing + two-pass reverb), approach, roar, laser charge/beam, glitch x3, swallowed.
  Audio music track (BossBreach starts it, BossDown/BossFall fade it; boss practice phases
  start it in amain). VOL_MUSIC 0.32. Not reviewed by ear.
- Bot: aims at the boss's first target, crash boxes in the lookahead, laser capsule, and
  `sweep_risk` (big penalty inside the remaining sweep, gradient toward the warning line).
  `bot::phase_trial`, `stage_limit()`, StageRun.phase_time/phase_deaths/boss_killed.
  PG_SIM=boss:<n>; PG_KILL=<screen frame>.

**Tuning (tune.rs):** JAWS_HP 12000, phases (share, secs, dmg mult, turn period):
(0.25, 30, 0.65, 840) (0.30, 40, 1.0, -) (0.30, 40, 1.0, 600) (0.15, 35, 0.42, 420);
FIN_HP 900, FIN_BLOW 0.25; JAWS_Y 82, HOME (150 right, 120 left), sway 12/480, bob 5/300;
laser (45 warn, 60 beam, 55 deg, 9 px), every 330 from 150; death 235 frames, slowmo 48,
blasts [0,8,15,22,30,40,52,66,82,100,140].

**Measured:** PG_SIM=boss:200 all phases 100% survived without bombs, medians 24.8 / 23.1
/ 26.0 / 24.8 s (21.6-32.0). PG_SIM=200 stage: 100% cleared, 100% no-miss, boss destroyed
in every run. PG_PERF: cpu 0.35-0.49 ms avg, 2 ms worst; occasional stalls only on the
first runs after a rebuild (Defender scanning the exe).

**Known / next:** the bot finds every phase easy (grazes 1-29); human difficulty is for
Sprint 9 and Jason's play. The music needs his ear (Sprint 8 is the mix). He dropped
"Split-sword starship_ flight to laser.png" into docs/reference mid-sprint (a bone-white
ship whose two blades join into a sword that fires a teal laser): not committed, asked him
what it's for. Next: Sprint 7, Score & shell (the capture bonus uses Boss.clean).

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
