# PILGRIM

A vertical bullet-hell shmup in Rust + macroquad: Stage 1 and its boss. Built sprint by sprint
from `SPRINTS.md`. The project setup mirrors https://github.com/Bananalorian/shinjuku.

## Who decides what
- Jason owns the creative direction: look, feel, names, story, the boss, what's in and out. Claude writes the code, offers options, and asks before changing a design decision. Anything marked *proposal* in `SPRINTS.md` waits for his approval.
- One sprint per go. Build it, verify it, ship it, report, then stop and wait for "continue" or "Sprint N".

## Build and run
```sh
cargo run --release                              # desktop
cargo test                                       # sim tests, headless (also run in CI)
rustup target add wasm32-unknown-unknown         # once
cargo build --release --target wasm32-unknown-unknown
python build_web.py                              # -> dist/index.html  (python3 on macOS/Linux)
```
The toolchain is pinned in `rust-toolchain.toml` and CI uses the same version. Don't bump Rust or macroquad without asking.

## Non-negotiables
1. **The ship has no inertia.** Diagonals are normalized. Input moves the ship on the tick it's read.
2. **A fixed 60 Hz sim.** Every gameplay timing is in frames. Gameplay RNG is seeded; cosmetic RNG is separate. `sim/` never draws, plays audio, or touches macroquad globals.
3. **Enemy bullets draw above everything except the HUD**, and their cores are the brightest pixels on screen. Check with F2 or `PG_GRAY=1`.
4. **Red means danger, teal means the Pilgrim.** Only the Pilgrim, the boss, enemy bullets and explosion fringes get color. Everything else uses the monochrome palette.
5. **Pixel-perfect:** a 640×360 canvas, a 270×360 playfield, integer scaling, nearest filtering, effects locked to the low-res grid.
6. **No camera shake while enemy bullets are live.** At most 3 full-screen flashes per second. Respect the Reduce flashing and Screen shake options.
7. **No asset files.** Art, font, sound and music are generated in code. The only exceptions are PNG sprite overrides or music files that Jason adds himself.
8. **Every tunable number lives in `tune.rs`.**
9. **Keep it cross-platform; assume Windows locally:** no `/tmp`, no bash-only build steps. Screenshots go to `./shots/` (gitignored).

## Design references
- [Boghog's bullet hell shmup 101](https://shmups.wiki/library/Boghog%27s_bullet_hell_shmup_101) is the design bible. `SPRINTS.md` §3 condenses it; follow it unless Jason says otherwise.
- Jason's reference art lives in `docs/reference/`: the PILGRIM sprite sheet (the ship, its animations, the palette) and the black hole image (background, mood, the RGB streaks). Match them. The palette tokens are in `SPRINTS.md` §2.
- Code to reuse from Shinjuku: `synth.rs`, `audio.rs`, `input.rs`, `pad.rs`, `storage.rs`, `font.rs`, `build_web.py`, `web/template.html`, `.github/`. The bomb is Shinjuku's grenade blast scaled up: `World::explode`, the post shader's `Shock` ring, and `synth::explosion` (the grenade uses the light variant; the bomb gets the heavy one).

## Definition of done (every sprint)
- Native and wasm builds are warning-free, `cargo test` is green, and every new rule has a test.
- **Every claim in the CHANGELOG is backed by a test, a screenshot (`PG_SHOTS`) or a bot run (`PG_AUTO`, `PG_SIM`).** Never report a fix you haven't seen working, and re-check earlier sprints' "Done when" items for regressions.
- Bump the version in `Cargo.toml` → add a CHANGELOG entry (same style as Shinjuku's) → update the README and `docs/screenshots/` → commit as `vX.Y.Z Title: why`, with a body explaining what changed and why → tag `vX.Y.Z` → push → confirm CI is green (Pages and Release).
- Update `MEMORY.md`: what shipped, tuning values that changed, known issues, what's next.
- Report to Jason: what changed, what to try, any decision needed. Then stop.

## Debug hooks (native)
| Hook | What it does |
|---|---|
| `PG_SCENE=title` / `stage:<seconds>` / `midboss` / `boss:<phase>` / `sandbox:<pattern>` / `stress` | jump straight there |
| `PG_SHOTS=60,300` | save those frames to `./shots/` and quit |
| `PG_AUTO=1` | the dodging bot plays |
| `PG_SIM=200` | 200 headless seeded bot runs with a balance report |
| `PG_SEED=<n>` | fixed seed |
| `PG_GRAY=1` | grayscale screenshots (value check) |
| `PG_WAV=1` | dump every synthesized sound to `./shots/sfx/` |

Debug keys (debug builds only): F1 hitboxes · F2 grayscale · F3 quarter speed · F4 frame step · F5 invincible ·
F6 next section · F7 next sandbox pattern · F8 performance overlay · F9 export sprite sheets.
