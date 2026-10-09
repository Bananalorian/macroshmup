# MEMORY

Session notes for Claude Code: what shipped, tuning that changed, known issues, what's next.
Newest first.

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
