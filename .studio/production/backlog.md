# Production Backlog

**Next free ID: TASK-0009.** IDs are never reused. Statuses and record rules:
`.studio/OPERATING_MANUAL.md` → Task records. Each task's evidence lives in its record in
[`tasks/`](tasks/); this file is the index. Populated on 2026-10-16 during onboarding
([TASK-0002](tasks/TASK-0002-studio-onboarding.md)) from Git, CI, `SPRINTS.md`, `MEMORY.md` and
`CHANGELOG.md`.

## Tasks

| ID | Task | Status | Owner | Record | Source |
|---|---|---|---|---|---|
| TASK-0001 | Shared project rules in `PROJECT_RULES.md` | verified; awaiting Jason's acceptance | Codex (verified by Claude Code) | [record](tasks/TASK-0001-shared-project-rules.md) | Jason with Codex, 2026-10-16 |
| TASK-0002 | Studio onboarding and permanent task records | verified; awaiting Jason's acceptance | Claude Code | [record](tasks/TASK-0002-studio-onboarding.md) | Jason, 2026-10-16 |
| TASK-0003 | Sprint 13: Arcade shell (v0.14.0): CRT shaders, a Zero Ranger-like title, the WARNING lottery, a pause button | proposed (`SPRINTS.md` marks it *proposal*) | — | not yet | `SPRINTS.md` Sprint 13; UX plan, 2026-10-12 |
| TASK-0004 | Sprint 14: Music & mix (v0.15.0) | proposed (in the plan; waits for Jason's go) | — | not yet | `SPRINTS.md` Sprint 14 |
| TASK-0005 | Sprint 15: Tuning (v0.16.0) | proposed (in the plan; waits for Jason's go) | — | not yet | `SPRINTS.md` Sprint 15 |
| TASK-0006 | Sprint 16: 1.0 (v1.0.0) | proposed (in the plan; waits for Jason's go) | — | not yet | `SPRINTS.md` Sprint 16 |
| TASK-0007 | Automatic task-record checks and initial Studio commit | verified; awaiting acceptance | Codex | [record](tasks/TASK-0007-automatic-record-checks.md) | Jason, 2026-10-16; TASK-0002 A11 |
| TASK-0008 | Toolkit v0.2.0 upgrade and task-recordkeeping review | verified; awaiting acceptance | Codex | [record](tasks/TASK-0008-toolkit-upgrade-review.md) | Jason, 2026-10-16 |

**Before anything new:** Jason said after v0.13.0 that he has "lots to discuss" (`MEMORY.md`,
v0.13.0). His list comes first; Sprint 13 waits for his go.

## Open questions for Jason
Not tasks yet; each becomes one if he decides to act on it.

| # | Question | Since | Source |
|---|---|---|---|
| Q-01 | The rings at the start of a run "not from the player": he'll show us which. | v0.10.2 | `MEMORY.md` v0.10.2 to v0.13.0 |
| Q-02 | The stage tally's rank thresholds (S to D, `RANKS` in `tune.rs`): the bot has ranked S on every run since v0.10.0. Retune now or in Sprint 15? | v0.9.0 | `MEMORY.md` v0.9.0 to v0.13.0 |
| Q-03 | Boss phases are about 16% shorter with the kit. Retune or leave? | v0.13.0 | `MEMORY.md` v0.13.0 |
| Q-04 | The kit's words compete with point popups mid slam chain. | v0.13.0 | `MEMORY.md` v0.13.0 |
| Q-05 | The ship's size: kept, because the hitbox now always shows. Open only if he wants more. | v0.12.0 | `SPRINTS.md` Sprint 11 |
| Q-06 | Slow motion is still in the bomb's dip (`BOMB_DIP` 24 frames) and the boss's death (`JAWS_SLOWMO` 48). His "the slowmo makes the game feel like it is lagging" (v0.12.0) was about the Slam, and the bomb is to stay as he likes it (v0.10.1). No change proposed; recorded so it stays a conscious call. | v0.12.0 | `MEMORY.md` v0.12.0, v0.10.1; `src/tune.rs` |
| Q-07 | PNG overrides from `art/` are described in `SPRINTS.md` §2, but nothing loads them yet. Still wanted? | v0.1.0 plan | found 2026-10-16 (TASK-0002) |
| Q-08 | Resolved 2026-10-16: Jason authorized committing Studio files and excluding the toolkit clone; TASK-0007 adds the ignore rule and initial commit. | 2026-10-16 | TASK-0007 (request quoted in record) |

**Older, last recorded as open**, with no later answer found in `MEMORY.md` or `CHANGELOG.md`.
They may already be settled.

| # | Question | Last recorded |
|---|---|---|
| Q-09 | The wide shot's bolts: stay red, or go teal to match the laser? | v0.8.0 |
| Q-10 | The focus ratio (default 0.55; presets 0.45-0.75 in the pause menu). | v0.3.0 |
| Q-11 | Full screen by default wasn't tried on Jason's display. | v0.7.2 |
| Q-12 | The Windows XInput controller path wasn't confirmed on his hardware. | v0.6.5 |
| Q-13 | The music and sounds haven't been reviewed by ear (Sprint 14 covers it). | v0.8.0 |
| Q-14 | A real phone's frame rate is unmeasured (Sprint 15 covers it). | v0.4.0 |
| Q-15 | The black hole's grooves read a little ring-like at full size. | v0.4.0 |
| Q-16 | The pause menu's PATTERN row does nothing in the stage (not re-checked). | v0.6.0 |

## Releases before the Studio
Shipped means a version tag whose CI run passed: tests, the browser and three desktop builds,
and the GitHub Release. All 31 tags were checked with `gh run list` on 2026-10-16. Two needed a
second run: v0.8.0 (the first tag run failed on an overflow in the bot's endless lives, then it was
fixed and the tag moved) and v0.8.5 (one run cancelled). Jason's acceptance wasn't recorded
before the Studio. Each release's claims (tests, `PG_SIM`, screenshots) are in `CHANGELOG.md` and
`MEMORY.md` under its version; they weren't re-run during onboarding, except the full test suite
at v0.13.0.

| Version | Date | What | Commit | Jason's direction on record (`MEMORY.md`) |
|---|---|---|---|---|
| v0.1.0 | 2026-10-10 | Sprint 0, First light | `8bcae2a` | none recorded |
| v0.2.0 | 2026-10-10 | Sprint 1, The Pilgrim | `7129107` | none recorded |
| v0.3.0 | 2026-10-10 | Sprint 2, Danmaku engine | `f89b382` | his feedback on v0.2; no go quoted |
| v0.4.0 | 2026-10-11 | Sprint 3, Light & particles | `d64401e` | his feedback on v0.3; no go quoted |
| v0.5.0 | 2026-10-11 | Sprint 4, The special weapon | `f667203` | his feedback on v0.4; no go quoted |
| v0.6.0 | 2026-10-11 | Sprint 5, Stage 1 | `3eb3140` | his calls before the sprint (mid-boss, figure, stage name) |
| v0.6.1 | 2026-10-11 | Hotfix, The rumble stops | `c8df021` | his report |
| v0.6.2 | 2026-10-11 | Hotfix, Quiet radio | `4d5f8ae` | his report |
| v0.6.3 | 2026-10-11 | Bone white | `283f1c6` | his calls |
| v0.6.4 | 2026-10-11 | Colors | `1bf2ed5` | his call ([DEC-0002](../decisions/DEC-0002-five-bullet-hues.md)) |
| v0.6.5 | 2026-10-11 | XInput, the Cathedral mock-up | `fba270c` | his report and request |
| v0.7.0 | 2026-10-11 | Sprint 6, JAWSTHIC | `3fff0f2` | his concept and name |
| v0.7.1 | 2026-10-12 | Split sword | `7413230` | his call |
| v0.7.2 | 2026-10-12 | Full screen | `67770c4` | his feedback and pick |
| v0.7.3 | 2026-10-12 | A fifth smaller | `f2e0c82` | his request |
| v0.8.0 | 2026-10-12 | Sprint 7, Score & shell | `e53923b` | his calls |
| v0.8.1 | 2026-10-12 | Full size, dark blades | `0a92f2f` | his request |
| v0.8.2 | 2026-10-12 | No ghosts | `fd310a6` | his screenshot (a misread, undone in v0.8.3) |
| v0.8.3 | 2026-10-12 | Clean sides | `ad169df` | his correction |
| v0.8.4 | 2026-10-12 | Your sprites | `4ef0f96` | his art; he OK'd the redraw |
| v0.8.5 | 2026-10-12 | The new JAWSTHIC | `0855885` | his request |
| v0.8.6 | 2026-10-12 | Stars and particles | `39cc2a3` | his list |
| v0.8.7 | 2026-10-12 | One tail | `c02db11` | his request (reverted in v0.8.8) |
| v0.8.8 | 2026-10-12 | Back on track | `f3fdf5b` | his correction |
| v0.9.0 | 2026-10-12 | Sprint 8, Feel | `b15175d` | "go" |
| v0.10.0 | 2026-10-13 | Sprint 9, Boss attacks & the Slam | `47b5fa6` | "add this to sprint 9 and continue" |
| v0.10.1 | 2026-10-13 | The bomb as it was | `3acae3f` | his report |
| v0.10.2 | 2026-10-13 | Arcade feel | `fc7fe07` | his list and picks ([DEC-0004](../decisions/DEC-0004-hitbox-always-shown.md)) |
| v0.11.0 | 2026-10-13 | Sprint 10, Style | `6097dd7` | "lets continue to sprint 10" |
| v0.12.0 | 2026-10-13 | Sprint 11, The Swarm | `1356b53` | "sprint 11 go" and picks ([DEC-0003](../decisions/DEC-0003-bullet-color-by-action.md)) |
| v0.13.0 | 2026-10-13 | Sprint 12, Augments & shield | `cf49e4f` | "lets get sprint 12 done" and picks ([DEC-0005](../decisions/DEC-0005-kit-earned-by-style.md)) |

Not a release: `7a7719c` (2026-10-12) wrote the UX plan, Sprints 8-13, into `SPRINTS.md`.
