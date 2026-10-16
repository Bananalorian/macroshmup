# DEC-0004 — The hitbox shows all the time by default

| Field | Value |
|---|---|
| State | approved (current) |
| Date | 2026-10-13, shipped in v0.10.2 (`fc7fe07`) |
| Supersedes | the plan's "The hitbox ... shows while focused" (`SPRINTS.md` §3, from v0.1.0) |
| Recorded | 2026-10-16, during onboarding ([TASK-0002](../production/tasks/TASK-0002-studio-onboarding.md)) |

**Question:** the ship is big and its hitbox small; how does the player see where they can be hit?

**Options:** shrink the ship, or always show the hitbox (`MEMORY.md`, v0.10.2: "hitbox small vs
big ship, he chose ALWAYS SHOW IT (asked with options)").

**Decision:** the hitbox shows all the time by default. Options → SHOW HITBOX can switch it back
to focus only. Saves from before v0.10.2 start with it on.

**Approval:** Jason, 2026-10-13 (above). v0.12.0 kept the ship's size because of it
(`SPRINTS.md` Sprint 11).

**Code and tests:** `src/storage.rs` default `hitbox_always: true`, saved as `hitbox_shown`;
`menu::tests::the_options_change_and_are_saved` checks that an old `hitbox=focus` save still
starts with it always on (passed on 2026-10-16, TASK-0002 V3).

**Conflict resolved (2026-10-16):** `SPRINTS.md` §3 is marked *Superseded* with a link here; the
old wording is kept.
