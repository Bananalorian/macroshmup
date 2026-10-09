# PILGRIM

![The title screen](docs/screenshots/title.png)

> SOMEWHERE BEYOND THE EVENT HORIZON. A SMALL SHIP, A LONG WAY FROM HOME.

A vertical bullet-hell shmup: Stage 1 and its boss, built sprint by sprint.
You fly the TYPE-07 "PILGRIM" toward a black hole.

**[▶ Play in your browser](https://bananalorian.github.io/macroshmup/)** ·
**[Download for Windows, macOS, Linux](https://github.com/Bananalorian/macroshmup/releases/latest)** ·
[Changelog](CHANGELOG.md) · [Sprint plan](SPRINTS.md)

Everything is generated in code. There are no asset files: the sprites, the font,
and (soon) every sound and the music are built when the game starts. Written in
Rust on [macroquad](https://macroquad.rs).

| | |
|---|---|
| ![The empty playfield](docs/screenshots/stage.png) | ![On a phone](docs/screenshots/phone.png) |

## Status

**v0.1, First light:** the title and an empty playfield with a parallax starfield,
running natively and in the browser, on desktop and phone. The ship arrives in v0.2.
See [SPRINTS.md](SPRINTS.md) for the plan.

## Controls

| | Keyboard | Touch |
|---|---|---|
| Start | Z or Enter | tap |
| Back to the title | Esc | |
| Fullscreen | F11 or Alt+Enter | |

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
| `PG_SCENE=title` / `stage:<seconds>` | jump straight there |
| `PG_SHOTS=60,300` | save those frames to `./shots/` and quit |
| `PG_SEED=<n>` | fixed seed |
| `PG_GRAY=1` | grayscale screenshots (value check) |
| `PG_SIZE=1280x720` | window size |
| `PG_PERF=1` | start with the performance overlay |

F8 toggles the performance overlay in debug builds.

## Credits

Design and art direction by Jason Bartimus. Code by Claude Code. The setup, the font
and the web shell come from [Last Train to Shinjuku](https://github.com/Bananalorian/shinjuku).
Design rules from [Boghog's bullet hell shmup 101](https://shmups.wiki/library/Boghog%27s_bullet_hell_shmup_101).
