# PILGRIM

![JAWSTHIC, Stage 1's boss: its pectoral fin and tail fin spin opposing spirals over the black hole](docs/screenshots/boss.png)

> SOMEWHERE BEYOND THE EVENT HORIZON. A SMALL SHIP, A LONG WAY FROM HOME.

A vertical bullet-hell shmup: Stage 1 and its boss, built sprint by sprint.
You fly the TYPE-07 "PILGRIM" toward a black hole.

**[▶ Play in your browser](https://bananalorian.github.io/macroshmup/)** ·
**[Download for Windows, macOS, Linux](https://github.com/Bananalorian/macroshmup/releases/latest)** ·
[Changelog](CHANGELOG.md) · [Sprint plan](SPRINTS.md)

Everything is generated in code. There are no asset files: the ship, the sprites,
the font and every sound are built when the game starts. Written in Rust on
[macroquad](https://macroquad.rs).

| | |
|---|---|
| ![Gunships alternate sides, popcorn around them](docs/screenshots/gunships.png) | ![A heavy: rings of big orbs and needle bursts](docs/screenshots/heavies.png) |
| ![Turret buoys: teal rings and violet spirals among amber fans](docs/screenshots/buoys.png) | |
| ![The derelict rises out of the disk, its turrets firing](docs/screenshots/derelict.png) | ![The same in grayscale: bullet cores are the brightest pixels](docs/screenshots/derelict-gray.png) |
| ![The breather: the disk flares, debris drifts, a tiny figure falls](docs/screenshots/breather.png) | ![WARNING](docs/screenshots/warning.png) |
| ![The stage-clear tally: JAWSTHIC destroyed](docs/screenshots/stage-end.png) | ![The pattern sandbox](docs/screenshots/sandbox.png) |
| ![Phase 4, EVENT HORIZON: a flower and spiral arms bending toward the black hole](docs/screenshots/boss-phase4.png) | ![The same fight in grayscale: bullet cores are still the brightest pixels](docs/screenshots/boss-gray.png) |
| ![The derelict, cracked open, its seams burning and its iris open](docs/screenshots/stage.png) | |
| ![The bomb: the cancel ring sweeps the screen](docs/screenshots/bomb.png) | ![The Pilgrim shatters into its own pixels](docs/screenshots/death.png) |
| ![A boss-sized blast: the shock ring bends the black hole](docs/screenshots/blast.png) | ![On a phone](docs/screenshots/phone.png) |
| ![Layered rings and aimed fans](docs/screenshots/layered.png) | ![The same in grayscale: bullet cores are the brightest pixels](docs/screenshots/layered-gray.png) |
| ![The title: Stage 1 or the sandbox](docs/screenshots/title.png) | ![The pause menu: sandbox, options and tuning](docs/screenshots/tuning.png) |
| ![Game over](docs/screenshots/game-over.png) | |

![JAWSTHIC arrives: its fin cuts through the black hole's disk, it breaches, roars, and its bar fills](docs/screenshots/boss-intro.png)

![Its four phases: TEETH, FINS, GLITCH and EVENT HORIZON](docs/screenshots/boss-phases.png)

![Its death: every bullet cancels, blasts walk up from its tail, the screen whites out, then the tally](docs/screenshots/boss-death.png)

![Jason's JAWSTHIC beside the redraw at game size, traced from his sprite atlas](docs/screenshots/jawsthic-vs-art.png)

![JAWSTHIC's sheet: in play, at rest, charging, in each phase, glitching, facing either way](docs/screenshots/jawsthic-sheet.png)

![The derelict's death: the wreck burns, blasts ripple across it, then it goes up](docs/screenshots/derelict-death.png)

![The bomb beside Last Train to Shinjuku's grenade: the same blast, scaled up](docs/screenshots/bomb-vs-grenade.png)

![Every explosion size at 4 moments, and the black hole at the start and end of the stage](docs/screenshots/fx-sheet.png)

![The stage-clear tally: the lines counting up, the score, the rank slamming in, and the end](docs/screenshots/tally.png)

![JAWSTHIC's phase captured: an S slams in under the boss, its bonus counting up](docs/screenshots/boss-grade.png)

| | |
|---|---|
| ![The options: volumes, shake, flashing, the hitbox, the fire scheme](docs/screenshots/options.png) | ![The title's demo: the bot flies Stage 1](docs/screenshots/demo.png) |

![The Pilgrim before v0.8.4, Jason's sprites, and the redraw at game size: the flight form, then the laser form](docs/screenshots/pilgrim-vs-art.png)

![The Pilgrim's animation sheet: the flight form, banking, the five frames to the laser form, and the beam](docs/screenshots/pilgrim-sheet.png)

![The split sword in play: focus, and the blades close into one sword as the teal beam fires from its tip](docs/screenshots/pilgrim-forms.png)

![Stage 1's enemies in bone white, all built in code](docs/screenshots/enemy-sheet.png)

## Status

**v0.8, score & shell:** a complete arcade loop. Kills score, double point-blank;
cancelled bullets pay as shards; grazes add up. Each of the boss's phases ends on a
grade (S, A, B or C) that slams in with its bonus. The stage ends in a tally that
counts up to a rank, with a saved hi-score and two extends along the way. The title
has Boss Practice, options (volumes, the hitbox, a HOLD TO BEAM fire scheme) and a
demo the bot plays when it's left alone.

**v0.7.2, a bigger play area:** the playfield is square now, 1080 × 1080 on a
1080p screen, top to bottom, with narrower panels either side, and the game opens
full screen (F11 for a window).

**v0.7.1, the split sword:** the Pilgrim is redesigned from Jason's sheet: two
bone-white blades with teal channels, on a crossbar with an engine at each end.
When you focus, the blades close into one sword and the teal beam fires from its
tip. It flies exactly as before. Since v0.8.4 it's redrawn pixel by pixel from his
sprites ([before, his art, now](docs/screenshots/pilgrim-vs-art.png)), and it
animates by moving whole parts, so both blades keep their white edges when it banks.

**v0.7, the boss:** JAWSTHIC, a great white carved from moon rock with a gothic
cathedral for a body, from Jason's concept, redrawn in v0.8.5 from his sprite atlas:
cracked flagstones, a red diamond eye, two rows of fangs, lancet windows, a segmented
tail trailing glitch smears, the black hole stripping bits off its back and belly.
It fires stars and crystal shards among its orbs and rice (v0.8.6). Its fin cuts
through the black hole's disk before it breaches. It fights in four phases, each with a timer: TEETH, FINS, GLITCH and EVENT
HORIZON. Each phase's bar drains from both ends in to a gem that turns green, yellow,
orange, then red. The phases bring aimed fans,
breakable fin turrets, a glitching curtain, laser sweeps, and bullets bending into
the black hole. It dies in a chain of blasts in slow motion and a white-out, to its
own synthesized theme.

It ends Stage 1, OUTER DISK, from v0.6: two and a half minutes of bone-white lunar
gothic enemies, a breather where the disk flares, and the derelict mid-boss. Under it
are the bomb, deathbomb and lives from v0.5, the black hole and post effects from
v0.4, and the bullet engine and pattern sandbox from v0.3 (still on the title: 13
patterns plus stress scenes, target dummies, a dodging bot).
See [SPRINTS.md](SPRINTS.md) for the plan.

## Controls

| | Keyboard | Controller | Touch |
|---|---|---|---|
| Move | arrows or WASD | D-pad or left stick | drag anywhere |
| Fire | Z | A | automatic while you steer |
| Focus (slow + beam) | Shift | RB or RT | a second finger |
| Bomb (8 frames after a hit still saves you) | X | B | BOMB button |
| Next sandbox pattern (sandbox only) | `]` (`[` goes back) | LB | NEXT button |
| Pause, sandbox and tuning | Esc | Start | II button |
| Sound on/off | M | | |
| Fullscreen | F11 or Alt+Enter | | |

The pause menu holds the sandbox switches (pattern, autopilot, slow-mo,
invincible, hitboxes, grayscale, FPS, BOOM TEST), the options (reduce flashing,
screen shake) and the tuning panel: speed, focus ratio, blend frames, touch
sensitivity, and 8-way or analog stick. On a keyboard,
1 to 6 change speed, focus and blend while you fly. Tuning and options are saved;
the sandbox switches last for the session.

## Build

```sh
cargo run --release                              # desktop
cargo test                                       # sim tests, headless (also run in CI)
rustup target add wasm32-unknown-unknown         # once
cargo build --release --target wasm32-unknown-unknown
python build_web.py                              # -> dist/index.html  (python3 on macOS/Linux)
```

Rust is pinned to 1.91.1 in `rust-toolchain.toml`, the same version CI uses.

### Debug hooks (native)

| Hook | What it does |
|---|---|
| `PG_SCENE=title` / `options` / `stage:<seconds>` / `midboss` / `boss` / `boss:<phase>` / `sandbox:<pattern>` / `stress` / `shots:<n>` | jump straight there (only `stage:0` counts for the hi-score; `stage:85` starts Stage 1 at 1:25; `boss` starts at the boss's arrival, `boss:2` straight into its phase 2; `sandbox:3` or `sandbox:spiral`; `shots`: keep n player shots alive in the sandbox) |
| `PG_SHOTS=60,300` | save those frames to `./shots/` and quit |
| `PG_AUTO=1` | the dodging bot plays (and retries forever) |
| `PG_SIM=200` | 200 headless seeded bot runs of Stage 1: clears, deaths per section, section timing, the derelict's kill time, each boss phase's time (`PG_SIM=boss:200` plays each boss phase without bombs; `PG_SIM=patterns:200` runs every sandbox pattern) |
| `PG_SEED=<n>` | fixed seed |
| `PG_GRAY=1` | grayscale screenshots (value check) |
| `PG_SIZE=1280x720` | window size |
| `PG_PERF=1` | the performance overlay, a line every 30 s in the terminal, and a summary when the game closes |
| `PG_SHEET=1` | save the ship's animation sheet and the enemy sheet to `./shots/` and quit |
| `PG_FXSHEET=1` | save every explosion size at 4 moments and the black hole at the start and end of the stage |
| `PG_MOCKUP=1` | save the boss's sheet (in play, at rest, charging, each phase) to `./shots/` |
| `PG_BOOM=<0-3>` | set off a small, medium, large or boss blast 30 frames in |
| `PG_INVINCIBLE=1` | bullets can't hurt you |
| `PG_BOMB=<frame>` / `PG_DIE=<frame>` | bomb, or get hit, on that frame |
| `PG_KILL=<frame>` | break the boss's current phase on that screen frame |
| `PG_WAV=1` | dump every sound to `./shots/sfx/` |
| `PG_MENU=1` | open the pause menu |
| `PG_FRESH=1` | ignore the save file |

Debug builds: F1 hitboxes · F2 grayscale · F3 quarter speed · F4 frame step (Enter
resumes) · F5 invincible · F6 next stage section (the boss's next phase, during the
fight) · F7 next pattern · F8 performance overlay · F9 save the sprite sheets.

## Credits

Design and art direction by Jason Bartimus. Code by Claude Code. The setup, the font,
the synth, the controller code and the web shell come from
[Last Train to Shinjuku](https://github.com/Bananalorian/shinjuku).
Design rules from [Boghog's bullet hell shmup 101](https://shmups.wiki/library/Boghog%27s_bullet_hell_shmup_101).
