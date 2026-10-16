# DEC-0005 — The ship's kit is earned by style; no item pickups

| Field | Value |
|---|---|
| State | approved (current) |
| Date | 2026-10-13, shipped in v0.13.0 (`cf49e4f`) |
| Amends | `SPRINTS.md` §8 Decision 3, "Power-ups: none; full power from the start", and §3 "No power-ups in v1; you start at full power" (both from v0.1.0) |
| Recorded | 2026-10-16, during onboarding ([TASK-0002](../production/tasks/TASK-0002-studio-onboarding.md)) |

**Question:** how does the player get the augments and shield? (`SPRINTS.md` Sprint 12: "start
with them, style, or pickups".)

**Options:** start with them (would keep "full power from the start"); earn them by style
milestones; item pickups.

**Decision:** earned by style. The first time a life reaches rank B the left augment comes
online, A the right one, S the shield. A life keeps what it earns and loses it all with the
ship. The augments add about a fifth to the ship's damage. There are still no item pickups.

**Approval:** Jason, 2026-10-13, picked "Style milestones (Recommended)" and "Engine pods
(Recommended)" (`MEMORY.md`, v0.13.0; `SPRINTS.md` Sprint 12, "Jason's calls").

**Effect on Decision 3:** "full power from the start" no longer holds, since the kit adds power
as you play. "No pickups" still holds. Power-up pickups stay under "Later (outside this plan)".

**Code and tests:** `src/sim/kit.rs`, `src/sim/kit_tests.rs`; `KIT_RANKS` in `src/tune.rs`.

**Conflict resolved (2026-10-16):** `SPRINTS.md` §3 and §8 Decision 3 now say they were amended
by v0.13.0, with a link here; the original wording is kept.
