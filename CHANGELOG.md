# Changelog

Every version is one build sprint. Newest first. Versions are tagged in git, and
each tag publishes a GitHub Release with downloads.

## [0.8.2] - 2026-10-12 - No ghosts

### Changed
- **No more afterimages.** Moving side to side at full speed used to leave teal
  ghost copies of the ship trailing beside it, glowing through the gap between the
  blades: the "lines" in your screenshot. They're gone, so the ship draws only
  itself and banks cleanly ([in play](docs/screenshots/stage.png),
  [its sheet](docs/screenshots/pilgrim-sheet.png)). The engine trails stay. The
  positions the ship kept for the afterimages went with them.

### Checked
- Every screenshot where the ship was moving is retaken, and none shows a ghost.
  The animation sheet's afterimage demo is gone.
- Native and web builds are warning-free, and all 235 tests pass, in both debug
  (as CI runs them) and release. One fewer test than before: the afterimages'
  position history went with them.

## [0.8.1] - 2026-10-12 - Full size, dark blades

### Changed
- **The Pilgrim is back to its v0.7.2 size**, about 31 × 44 px
  ([its sheet](docs/screenshots/pilgrim-sheet.png),
  [in play](docs/screenshots/pilgrim-forms.png)). Every part is exactly as it was
  then; only the lights below changed.
- **No lit lines down the blades in flight.** Their channels stay dark, with just
  the red hearts. Lit dashes down both blades fought the banking frames as the ship
  moved side to side, even holding still. The channels only light when the laser
  fires, flowing up the sword to its tip, the part you liked. The engines' and the
  capsule's small window lights stay.

### Checked
- **Tests prove:**
  - the ship is 29-33 px wide and 42-48 px tall again;
  - in flight, firing or not and mid-transformation, no channel light is lit and
    the window lights hold steady;
  - the channel lights and flows while the laser fires.
- Every screenshot is retaken. Native and web builds are warning-free, and all 236
  tests pass in both debug (as CI runs them) and release.

## [0.8.0] - 2026-10-12 - Score & shell

### Added
- **Scoring** that's simple at first and has depth to find:
  - **Kills** score by size, from 500 for a drone to 200,000 for the derelict. A
    kill within 48 px of the Pilgrim scores double: its number shows in teal, and a
    teal ring flashes.
  - **Shards:** when a heavy or the derelict dies, or a boss phase breaks, its
    bullets turn into shards worth 50 each. A bomb's cancels give nothing.
  - **Grazes** add 100 each.
  - **Weak points:** the derelict's gun pods score 15,000, JAWSTHIC's fins 25,000,
    and JAWSTHIC itself 300,000.
  - **Popups:** points float up off every kill. Big ones (20,000 and up) come twice
    the size, flashing.
- **Arcade grades**, as you asked:
  - **Boss phases** ([one captured](docs/screenshots/boss-grade.png)): each of
    JAWSTHIC's phases ends on a grade:
    - **S:** no death or bomb, and broken inside 70% of its time. Pays double the
      capture bonus.
    - **A:** no death or bomb (captured). Pays the capture bonus: 150,000 to
      300,000, depending on the phase.
    - **B:** broken, but you died or bombed. Pays a fifth.
    - **C:** timed out. Pays nothing.

    The letter slams in over the boss, huge and white. A streak of light crosses
    the field and a ring bursts out, then the letter settles under CAPTURED with
    its bonus counting up. An S glows teal.
  - **The stage's rank,** S, A, B, C or D, comes from a rating out of 100:
    - the kill rate, up to 30;
    - the boss's four grades, up to 40, and its kill, 10;
    - no misses, 15;
    - no bombs, 5.
- **Extends:** an extra ship at 800,000 and at 1,800,000, up to six ships, with
  EXTEND flashing and a jingle.
- **The stage-clear tally**
  ([its moments](docs/screenshots/tally.png), [its end](docs/screenshots/stage-end.png)):
  1. The lines come in one by one, ticking and counting up: kills, the boss's four
     grades popping in, NO MISS +500,000, NO BOMB +300,000, and graze.
  2. The score counts up.
  3. The rank slams in.
  4. SOMEWHERE BEYOND THE EVENT HORIZON types itself in before the menu.

  Any button skips to the end. A new best shows NEW RECORD with a jingle.
- **The hi-score** is saved on the desktop and in the browser. Only Stage 1 played
  from the top counts, not Boss Practice, the sandbox or the demo.
- **The HUD after your sheet:**
  - **Left panel:** PILGRIM, the score (8 digits, with the leading zeros dimmed),
    the hi-score, lives and bombs, graze, kills and misses, and the controls.
  - **Right panel:** SOMEWHERE BEYOND THE EVENT HORIZON, the stage and section, the
    tuning, the palette strip, and the stamps.
  - **On a phone,** the top strip shows the score, lives, bombs and hi-score.
- **The title:**
  - Its menu: STAGE 1, BOSS PRACTICE (once you've reached the boss), PATTERN
    SANDBOX and OPTIONS.
  - The hi-score.
  - **A demo:** left alone for 20 seconds, the title hands over to the bot, which
    flies Stage 1 under DEMO and PRESS START
    ([the demo](docs/screenshots/demo.png)). Any button brings the title back.
- **Options** ([the screen](docs/screenshots/options.png)), all saved:
  - music and sound volume;
  - shake and reduce flash;
  - the hitbox: shown always, or only while focused;
  - the fire scheme: a focus button, or HOLD TO BEAM, where holding fire for a
    moment becomes the beam (CAVE style);
  - touch sensitivity and the stick.
- **Boss Practice** joins the title once you've reached the boss. It starts at its
  arrival.
- **Game over** shows your score and the hi-score, with NEW RECORD when you beat it.
- **Sounds:** the tally's tick, a slam for grades and the rank, the extend's jingle.
- **Debug hooks:** `PG_SCENE=options`. `PG_SIM=200` now reports the score, the
  extends, each phase's grades and the stage's ranks.

### Changed
- `PG_AUTO` waits for the tally to finish before it plays again.

### Fixed
- **My mistake with your save file.** While I built this, a test wrote to your real
  save file (`%APPDATA%\.pilgrim\save.txt`). The options and tuning you'd saved
  were reset to their defaults, and it saved a fake hi-score of 1,000, which I've put
  back to 0. Tests can't reach the save file any more: a test proves it, and the
  file's timestamp stays put through the whole suite. **If you'd changed the speed,
  focus, blend, stick, flashing, shake or mute, please set them again.**

### Checked
- **Tests prove:**
  - a kill scores its points, double within 48 px;
  - a heavy's cancelled bullets pay as shards;
  - grazes add up quietly, and the sandbox keeps no score;
  - each extend comes once, never past six ships, and the stage-end bonuses never
    earn one;
  - each boss phase is graded once, with the right bonus: S when broken clean and
    fast, A when clean, B after a bomb, C when it times out;
  - the stage's end pays the no-miss and no-bomb bonuses, and ranks the run from
    its rating;
  - only Stage 1 from the top sets the hi-score;
  - the hi-score, the options, the fire scheme and Boss Practice's unlock are saved
    and read back;
  - the title offers Boss Practice once it's unlocked;
  - the demo starts after 20 seconds, and any button, its end, or a minute ends it;
  - HOLD TO BEAM turns a held fire into the beam;
  - volumes step from silent to full;
  - the tally ticks, slams once and sings for a record;
  - every label has its glyphs and fits its panel.
- **The bot** (`PG_SIM=200`):
  - **Scores:** a median of 2,971,850. 80% of runs land between 2.75M and 3.27M.
  - **Extends:** both in 98% of runs.
  - **Grades:**
    - phase 1 always grades A;
    - phase 2 grades S in 92% of runs;
    - phases 3 and 4 split between S and A.
  - **Rank:** A in 84% of runs, S in 15%.
  - **Unchanged:** it still clears the stage 100% of the time with no misses.

  The bot dodges better than any human, so a strong human run should land around
  B or A.
- **A full Stage 1 run by the bot**, screenshotted in real time: JAWSTHIC's last
  grade crosses an extend, then the white-out, then the tally with NEW RECORD and
  rank A.
- Native and web builds are warning-free, and all 236 tests pass.
- **Not checked by ear or by hand:**
  - I couldn't listen to the new sounds. Please give them a listen.
  - The new screens' keyboard, controller and touch input is covered by tests of
    the menu logic, not tried on a device.

## [0.7.3] - 2026-10-12 - A fifth smaller

### Changed
- **The Pilgrim is 20% smaller** with all its detail kept
  ([its sheet](docs/screenshots/pilgrim-sheet.png),
  [in play](docs/screenshots/pilgrim-forms.png)). It's now about 25 × 36 px,
  down from 31 × 44. Every part is still there:
  - the two blades, each with its 3-pixel teal channel, red heart and antenna;
  - the crossbar, and the nacelles with their windows, beacons, nozzles and twin
    flames;
  - the capsule, with its window, keel light, strobes and struts.

  The transformation still takes 5 frames to a broad sword, and the hitbox,
  shots, speeds and beam haven't changed.
- **The channel lights hold still in flight.** The dashes down both blades used to
  flow, faster while you fired, and that fought the banking frames as you moved
  side to side. Now they only flow up the sword while the laser fires. The
  nacelles' and the capsule's windows no longer flicker either.
- The engine trails and the lock's sparks come from the smaller ship's nozzles and
  crossbar.

### Checked
- **Tests prove:**
  - the ship is 24-26 px wide and 34-37 px tall;
  - every frame keeps the hitbox's column and every kind of light at every bank;
  - the transformation still closes, converges and tucks in frame by frame;
  - in flight, firing or not and mid-transformation, the channel and window
    lights don't change from frame to frame;
  - the channel lights flow only while the laser fires.
- Every screenshot is retaken. Native and web builds are warning-free, and all
  216 tests pass.

## [0.7.2] - 2026-10-12 - Full screen

### Changed
- **A bigger play area** (your call: 360 wide). The playfield is square now,
  360 × 360 pixels instead of 270 × 360, and it still runs top to bottom. On a
  1080p screen that's 1080 × 1080, with the side panels narrowed from 555 to 420
  screen pixels each ([in play](docs/screenshots/boss.png)).
  - **Everything placed across the field spreads with it:**
    - the enemies' lanes and the popcorn's sweeps;
    - the black hole and the derelict over it;
    - your starting point;
    - the sandbox's guns and target dummies;
    - JAWSTHIC's home spots, its sway, and its fin's pass through the disk.
  - **Its phase 3 curtain** is 23 bullets wide instead of 17, so it still spans
    the field.
  - **Patterns themselves are unchanged:** the same bullets at the same speeds,
    with more room around them.
  - **The side panels** break their long lines in two (STAGE 1 / OUTER DISK, and
    the stamps at the bottom) so everything fits.
  - On a phone the screen is 360 wide too, so each pixel is drawn a little smaller
    there (3x instead of 4x on an iPhone-sized screen).
- **The game opens full screen** on the desktop. F11 (or Alt+Enter) switches to a
  window, and the game remembers your choice. Debug runs (any `PG_` setting) stay
  in their window. The browser version is unchanged: F11 still takes it full
  screen.
- **The laser form keeps its size.** In v0.7.1 the blades closed into a narrow
  sword and the whole ship shrank by a third when you focused. Now they close into
  one broad sword, nearly as wide as the two blades were
  ([in play](docs/screenshots/pilgrim-forms.png)). It has a sharper point, and a
  bevel down each edge. The nacelles tuck in beside it.

### Checked
- **The bot, with all that room:**
  - **The whole stage** (`PG_SIM=200`, the bot with lives and bombs): 100%
    cleared, 100% with no misses, 0.03 bombs a run.
    - It kills 79% of the enemies, down from 86%, since they're spread wider.
    - The derelict is destroyed in 94% of runs, down from 100%; in the rest it
      leaves on its timer.
    - JAWSTHIC is destroyed in every run.
  - **Each boss phase without bombs** (`PG_SIM=boss:200`): every phase is survived
    in 99-100% of runs.
    - Each breaks in a median 24.9 / 29.5 / 29.0 / 25.0 s, inside the 20-35 s
      target. FINS and GLITCH take longer than before (23.1 and 26.0 s): the bot
      has further to go to line up under the fins.
    - Phases 2 and 4 time out in 2% of runs, and the GLITCH laser catches the bot
      once in 100.
  - **Every sandbox pattern** (`PG_SIM=patterns:50`): 100% survived.
- **Tests prove:**
  - the playfield is 360 × 360 in every layout, and 1080 × 1080 on a 1080p screen
    with no letterbox;
  - every line of the side panels fits them, and the tuning hint clears the
    controls;
  - the touch buttons sit outside the wider field;
  - no wave spawns closer than 20 px to a side;
  - the sandbox's patterns still leave a 12 px gap in every band, and the boss's
    fairness and 20-35 s checks still pass in every phase;
  - the game opens full screen until you choose a window, an old save included.
- **Not checked on your screen:** I didn't take over your display to try full
  screen while you were working. Please check that it opens full screen and that
  F11 brings back a window.
- Every screenshot is retaken with the wider field. Native and web builds are
  warning-free, and all 215 tests pass.

## [0.7.1] - 2026-10-12 - Split sword

### Changed
- **The Pilgrim is redesigned from your split-sword sheet**
  ([your reference](docs/reference/Split-sword%20starship_%20flight%20to%20laser.png)),
  drawn in code in the bone white of Stage 1 ([its sheet](docs/screenshots/pilgrim-sheet.png)).
  It flies and fights exactly as before; only the look has changed.
  - **Flight form:** two stone blades side by side, each with a teal-lit channel and
    a red heart. An antenna stands on each blade's bevelled top. Under the blades, a
    crossbar carries an engine nacelle at each end, and a capsule hangs below on
    struts. All three engines burn teal.
  - **Laser form:** when you focus, the blades close into one sword over the
    5-frame focus blend ([in play](docs/screenshots/pilgrim-forms.png)):
    - they slide together and slim down;
    - the antennas fold away and the nacelles tuck in;
    - the two channels merge into one core.

    As they lock, a glint runs down the seam, a sparkle flares at the tip and sparks
    flick off the crossbar. The beam fires from the sword's tip.
  - **The hitbox hasn't moved:** the ship is drawn around it. In the flight form it
    sits in the gap between the blades, level with their red hearts. In the laser
    form the single heart sits on it, under the focus dot.
  - The ship is about 31 × 44 px, taller than the old one (30 × 34), because the
    sword reaches higher above the hitbox.
- **The focus beam is teal** to match your laser form: a translucent teal column
  with a hot core and bright dashes running up it. It's as wide as before, which is
  the width it hits.
- The wide shot's red bolts now leave from the blade tips, and its angled pod
  streams from the antennas. The shots themselves haven't changed.
- The old wing pods trailed a little behind the hull. They're gone, because the
  blades are part of one rigid ship.
- It still rolls as it banks: the dipping side foreshortens and darkens. Only plain
  stone drops out of the roll, so every light, window and channel stays.
- It shatters into its new pixels when it dies. The HUD's life icons are the split
  sword in miniature.
- The engine trails stream from the two nacelles and the capsule.
- The afterimages at full speed are fainter: at the old strength the broader
  silhouette piled up into a wall of teal.

### Added
- The ship's animation sheet (`PG_SHEET=1` or F9) now shows the new ship:
  - idle, thrust, fire, hit flash and bank;
  - the five frames from flight form to laser form;
  - the laser form with its beam.
- A review sheet of every frame, each form at each roll, 6x:
  `cargo test --release pilgrim_sheet -- --ignored` writes
  `shots/pilgrim_frames.png`.

### Checked
- **It plays the same.** The only change to the simulation is a counter for the
  lock's glint, which only the drawing reads. With `PG_SIM=50`, the bot clears the
  stage in 100% of runs with no misses, and destroys JAWSTHIC in every one, as in
  v0.7.0.
- **Tests prove:**
  - in the flight form, the blades are split from tip to foot, with the hitbox in
    the gap between their hearts;
  - in the laser form, they're one blade, with its heart on the hitbox and a
    one-pixel point straight above it;
  - frame by frame, the blades only close, slim down and lose their antennas, the
    channels converge into one, and the nacelles tuck in;
  - the blades lock on the frame the focus blend finishes, and part on the frame
    you let go;
  - rolling keeps the hitbox's column and every kind of light, and only the
    dipping side narrows;
  - every frame stays inside the ship's 85% value budget, so bullet cores are still
    the brightest pixels;
  - it shatters into every pixel of its flight form;
  - the lock throws two teal sparks each way and a sparkle at the tip, all gone
    within 10 frames;
  - nothing in the sprite atlas overlaps. The atlas is now 1024 × 512, to make
    room for the ship's 25 frames.
- **Screenshots:** every screenshot of the game is retaken with the new ship, and
  [the transformation in play](docs/screenshots/pilgrim-forms.png) is new. Two keep
  the old ship: v0.5's comparison with Shinjuku's grenade, and the retired Cathedral
  mock-up.
- Native and web builds are warning-free, and all 214 tests pass.

## [0.7.0] - 2026-10-11 - JAWSTHIC

### Added
- **JAWSTHIC, Stage 1's boss**, your concept, drawn in code from
  [your reference](docs/reference/jawsthic.jpg): a great white carved from moon rock
  with a gothic cathedral for a body. Lancet windows down its flank glow red and teal.
  It has a dorsal fin with an arched window, a pectoral fin on a piped joint, and a
  segmented stone tail ending in a crescent fin. A red diamond eye sits under a hard
  brow, and its jaws drop open on a red crosshair. Glitch smears stream off its tail
  and drip off its belly. It swims side-on across the top of the screen with its tail
  beating, sways and bobs, and now and then glitches round to face the other way.
  ([The boss sheet](docs/screenshots/jawsthic-sheet.png) shows it at rest, charging,
  in each phase, and facing either way.)
- **Its arrival** ([strip](docs/screenshots/boss-intro.png)). After the WARNING, its
  dorsal fin cuts through the black hole's disk like a shark's fin through water,
  throwing up a wake, with the tip of its tail breaking the surface behind. It dives,
  breaches out of the disk, and roars. Its health bar fills and its music starts. It
  can't be hurt until it's in position.
- **Four phases**, each with its share of a segmented health bar and a timer
  ([strip](docs/screenshots/boss-phases.png)):
  1. **TEETH**, 30 s: aimed 5-way fans from its jaws (they open before each volley)
     over slow, static rings from its windows, one window after another.
  2. **FINS**, 40 s: its pectoral fin and its tail fin are turrets with red cores.
     They spin opposing spirals while its jaws fire aimed needle bursts.
     - Breaking a fin silences its spiral and tears a quarter out of the phase's
       health.
     - With both fins gone, its windows open fire.
  3. **GLITCH**, 40 s: a curtain of rice like a corrupted screen, each row slipping
     sideways, with a lane drifting through it, and random sprinkles off its tail.
     - Then a laser sweep from its jaws: a red warning line for 45 frames, with dots
       showing which way it will turn, then the beam for 60 frames.
     - The beam sweeps through where you were, toward your nearer wall. The way out
       is back across the warning line, into the open.
  4. **EVENT HORIZON**, from 15% health, 35 s: a turning flower from its jaws and
     spiral arms off its tail, all bending toward the black hole, plus aimed needle
     bursts. The hole swallows any bullet that reaches it, its disk flaring as it
     feeds. The boss's stone cracks open and burns.

  A phase ends when its share of health is gone or its time runs out. Its bullets
  cancel into shards, it shakes the break off for two seconds, and the next phase
  begins. If phase 4 runs out, the black hole swallows the boss and you don't get
  the kill.
- **Its death** ([six moments](docs/screenshots/boss-death.png)):
  1. Every bullet on the screen cancels into shards, and the first moments run at
     quarter speed under an RGB surge.
  2. Eleven chained blasts walk up from its tail fin, each blowing off a plate or a
     fin in its own pixels.
  3. Its head goes up in a boss-sized blast and the screen whites out.
  4. The stage-clear tally shows JAWSTHIC DESTROYED (or LOST TO THE HOLE), the
     phases broken, and how many you got through without a death or a bomb. That
     count is ready for Sprint 7's capture bonus.
- **Its music and sounds**, synthesized in code like everything else:
  - **The theme:** a seamless 16-bar loop at 150 beats a minute in E Phrygian. The
    bass throbs on E and the F a semitone above (the shark's two notes) over drums,
    with a cathedral organ under it and a dark arpeggio in the second half. It
    starts as the boss breaches and fades as it dies.
  - **The sounds:** its approach (E and F throbbing faster as the fin cuts the
    disk), a roar, the laser's charge and beam, glitch stutters, and the hole
    swallowing it.
- **A painter for big bosses** (`render/carve.rs`):
  - Parts are drawn at any angle and either facing. Each pixel looks back into the
    part's own shapes, so edges stay crisp as the tail swings, and the stone's grain
    stays on the stone.
  - The light stays at the top left of the screen whatever the angle. The body is
    lit as a rounded barrel, each plate gets its own bevel, and a rim of light
    catches the underside so a dark belly still stands off the black.
  - Tones step through a short bone ramp, dithered only where two meet.
  - Every frame is carved once at startup, in 66 ms: the jaw's 7 openings, every
    turn of the tail and fins, both facings, and the broken fins.
- **Boss practice and tools:**
  - `PG_SCENE=boss` starts at its arrival; `boss:1` to `boss:4` go straight into
    that phase.
  - `PG_SIM=boss:<n>` plays each phase with the bot.
  - `PG_KILL=<frame>` breaks the current phase on that frame; F6 does the same in
    debug builds.
  - `PG_MOCKUP=1` saves the boss sheet.
- Two new patterns: a **curtain** (a falling row with a drifting lane, each row
  slipping sideways) and a **flower** (a ring whose speed swells into petals).

### Changed
- **The stage ends with the boss** instead of at 2:40. Its section, JAWSTHIC,
  follows the WARNING.
- The bot now plays against the boss:
  - it lines up under the weak points;
  - it keeps clear of the body and the tail;
  - when a laser warns, it steps back across the warning line instead of running
    ahead of the beam.
- A bullet pulled into a point is swallowed when it gets there (the sandbox's PULL
  too).
- The Cathedral proposal is retired. Its mock-up stays in the docs.

### Checked
- **Fairness in every phase**, by the same tests as the sandbox's patterns:
  - the same seed gives the same fight;
  - no bullet appears within 24 px of you, even parked right under each gun;
  - every 4 px band of the bottom half keeps a 12 px gap through each whole phase,
    on two seeds;
  - the beam and the boss's body hurt like bullets; the warning line doesn't.
- **The bot without bombs** (`PG_SIM=boss:200`):
  - it survives every phase on all 200 seeds;
  - each phase breaks in a median 24.8 / 23.1 / 26.0 / 24.8 s, between 21.6 and
    32.0 s in every run, with no timeouts;
  - CI checks the 80% and 20-35 s bars on 6 seeds a phase.
- **The whole stage** (`PG_SIM=200`, the bot with 3 lives and bombs): 100% cleared,
  100% with no misses, 0.05 bombs a run. JAWSTHIC is destroyed in every run, and
  every section starts on time.
- **Tests also prove:**
  - each phase breaks at its share of health, or times out;
  - when phase 4 times out, the hole takes the boss;
  - it can't be hurt before the fight;
  - breaking a fin silences its spiral, costs the phase a quarter of its health,
    and opens the windows, and shots from below can reach both fins;
  - the laser warns for 45 frames, then sweeps its 55 degrees;
  - the death sends 11 blasts, cancels every bullet into shards, runs at quarter
    speed and ends the stage;
  - the boss stays on the screen and turns round;
  - the white-out swells once and fades, never strobing;
  - the theme loops without a click, and no sound clips;
  - every frame fits the boss's texture;
  - its stone is bone white and never as bright as a bullet's core
    ([the grayscale check](docs/screenshots/boss-gray.png)).
- **Performance** (`PG_PERF`, 25 s of each phase and of the death): the game's own
  work is 0.35-0.49 ms a frame on average and 2.0 ms at worst, holding 60 fps.
  - A few stalls showed up on the first runs after a rebuild, while that work stayed
    at 1-2 ms. That points outside the game, probably Windows scanning the new exe;
    repeat runs had none.
- Native and web builds are warning-free, and all 209 tests pass.
- **Not checked by ear.** I checked the music and the new sounds as waveforms
  (structure, levels, the loop's seam) but couldn't listen to them. Please give
  them a listen.

## [0.6.5] - 2026-10-11 - XInput

### Fixed
- **Controller lag on Windows** (to be confirmed by Jason). His Xbox controller
  lagged in this game over Bluetooth but not in other games, and not on the
  keyboard. The desktop build read it through gilrs's Windows.Gaming.Input path:
  a thread re-scanning every controller every 8 ms. Windows builds now read
  Xbox-style controllers through XInput, like most PC games: on the spot, every
  frame, from the driver's latest state (about 10 microseconds a read). Empty slots
  are checked once a second, since asking an empty slot is slow. Linux and macOS
  still use gilrs. On Windows, PlayStation controllers now need Steam Input or
  DS4Windows, as in most PC games.

### Added
- **A mock-up of the Cathedral**, the boss proposed for Sprint 6
  ([the mock-up](docs/screenshots/cathedral-mockup.png), or `PG_MOCKUP=1`): in play
  at the event horizon, at rest, charging (its stained glass and core blaze red),
  and in phase 2 with its wing turrets. Drawn by the game's own painter in the
  bone white of Stage 1. It isn't in the game until Jason approves it.

### Checked
- Tests prove XInput's buttons, triggers and sticks map to the standard layout (stick
  up is up the screen), and that reading with or without a controller never fails.
  The guard against rumble still passes.
- Not checked on the hardware: the controller was asleep when this was built.

## [0.6.4] - 2026-10-11 - Colors

### Changed
- **Enemy bullets come in five colors:** red, teal, violet, amber and blue. Each
  pattern has its own, so layered fire reads as separate families: aimed singles
  and needle bursts red, rings teal or blue, spirals violet, fans and random
  sprinkles amber. Every bullet still has a white-hot core and a black outline,
  so the cores stay the brightest pixels on screen. The spawn bloom and the trails
  of curving bullets take the bullet's color. The sandbox patterns are colored
  too, and the stress scenes mix all five.
- The rule in CLAUDE.md and the art bible now says so: teal is still the Pilgrim's,
  red still marks danger (telegraphs, enemy lights), and enemy bullets come in
  several hues.

### Added
- `PG_PERF=1` prints a line every 30 s (frame times, how much of each frame was
  the game's own work, particles, bullets, enemies) and a summary when the game
  closes. `PG_AUTO=1` now plays on forever, retrying two seconds after a game over
  or the stage's end.

### Checked
- **A 10-minute soak on Jason's PC** (the bot playing the stage over and over,
  sound on, his controller connected): memory flat at 151 MB, handles flat at 473,
  60 fps on average throughout, the game's own work 0.3 ms a frame from the first
  minute to the last. No slowdown builds up. A few hitches (one of 1.6 s) came
  from outside the game: its own work in those frames was under 9 ms.
- Tests prove every bullet sprite, in every color, has a white core, a rim in its
  color and a black outline, and that no rim is as bright as a core.
- `PG_SIM=200`: unchanged, 100% cleared, every section on time.

## [0.6.3] - 2026-10-11 - Bone white

### Changed
- **Every Stage 1 enemy, redrawn in bone white** after your lunar gothic reference
  sheets, for far more contrast against the black. Each sprite is built from simple
  shapes and painted by one shader, so they all match: chunky bevels lit from the
  top left, a pitted moon-rock speckle, craters, black slots and red lights.
  - The popcorn drone is the CLERK, the gunship the BAILIFF, the turret buoy the
    INSPECTOR (its eye turns), and the heavy the NOTARY.
  - **The derelict** is your mid-boss sheet: a gothic spire with an arched window
    and a cross, a great iris eye with a red core, cratered shoulders, and two gun
    pods hanging beneath them. The weak points are now the two pods and a nose gun
    at the tip of the spire. When it cracks, its seams burn and the iris opens on
    a swelling red core.
  - Hits flush enemies red (a white flash wouldn't show on bone): strongly on
    small ones, softly on the derelict.
  - The lights in your sheets that were cyan and green are red here: teal still
    means you. Say if you want them back.
- The art bible's value budget for enemies is now bone white, up to cream; bullet
  cores are still the brightest pixels on screen.

### Added
- **Two new fodder enemies:**
  - **the courier**, a fast three-spired dart that streaks down in a line, some
    firing one aimed needle;
  - **the usher**, a horseshoe that flies in, stops, fires an aimed shot from each
    prong twice, and leaves.
  They come through the gunship, buoy and heavy sections, often in pairs.

### Removed
- **Rumble, entirely.** Over time it was lagging your Bluetooth controller. The
  game no longer sends a controller anything: no rumble, no phone buzz, and the
  controller library runs without its force-feedback thread. The RUMBLE row is
  gone from the pause menu. It can come back later as its own feature.

### Checked
- Tests prove: every enemy is bone white yet dimmer than a bullet core; each
  sprite matches its hurtbox; the derelict's weak points all hang below it; and
  nothing in the code can send a controller a motor command.
- `PG_SIM=200`: the bot still clears the stage on every seed, with every section
  on the timeline; the derelict goes down in a median 21 s.
- [The enemy sheet](docs/screenshots/enemy-sheet.png), [the derelict's death](docs/screenshots/derelict-death.png),
  and [the grayscale check](docs/screenshots/derelict-gray.png).

## [0.6.2] - 2026-10-11 - Quiet radio

### Fixed
- **Input lag with a Bluetooth controller.** Every rumble command is a radio packet
  to the controller, and over Bluetooth each one took 8 ms on average and up to
  62 ms. On Windows the controller library re-sent the motor level 20 times a
  second the whole time the game ran, even at rest, and the game re-sent its fire
  hum 15 times a second on top. Now the motors are only told when the strength
  changes (with a top-up before a long rumble runs out, and an "off" when it ends),
  and on Windows the game drives them itself from a thread of its own, so a slow
  write never holds up a frame.

### Checked
- 15 s of bot play through the buoys on the desktop: 52 rumble commands, where the
  old code sent about 20 a second from the library alone plus 15 a second for the
  hum; every frame on time (865 frames, none over 20 ms).
- In the browser: 7 rumble commands in 3 s of firing, down from about 56.
- Tests prove a held hum tells the controller once, not every few frames; a
  weaker rumble never cuts off a stronger one; the motors go off when it's over.
- `PG_PERF=1` now reports on any exit: frames over 20 ms, frames that didn't run
  exactly one tick, and rumble commands sent.

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
