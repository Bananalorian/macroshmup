# Studio State

Project: PILGRIM (repo `macroshmup`)
Status: ONBOARDED (TASK-0002, accepted 2026-10-16). No task approved or in progress.
Last updated: 2026-10-16T14:15Z by Claude Code, recording Jason's acceptance of TASK-0001, 0002, 0007 and 0008 and his approval of DEC-0007.

This file is an index, not proof. Every claim below points to its evidence.

## Verified position
- **Shipped:** v0.13.0 "Augments & shield" (Sprint 12), commit `cf49e4f`, tag `v0.13.0`, 2026-10-13. CI green on the tag (tests, browser and three desktop builds, Release) and on `main` (Pages). The GitHub Release has 4 downloads. TASK-0002 V1, V2.
- **History:** 31 tagged releases, v0.1.0 to v0.13.0, each with a passing CI run: [ledger](production/backlog.md#releases-before-the-studio). Sprints 0 to 12 are done; Jason's acceptance wasn't recorded before the Studio.
- **Tests:** `cargo test` 284 passed, 0 failed, 2 ignored on Windows (TASK-0002 V3). Linux CI runs 282 + 2 ignored on the same code (TASK-0007, publication verification); the gap is the two XInput tests inside `#[cfg(windows)] mod native` in `src/pad.rs` (checked by Claude Code, 2026-10-16).
- **Studio:** records, the record checker and its CI gate are committed and published on `main` since `ea1072f` (CI evidence in TASK-0007). Toolkit v0.2.0 at `95739b5`, ignored by Git (TASK-0008). No gameplay source, assets, tuning or game version has changed since `cf49e4f`.
- **Git:** `macroshmup.code-workspace` stays untracked on purpose.

## Current objective
Jason's game-design discussion: after v0.13.0 he said he has "lots to discuss" (`MEMORY.md`,
v0.13.0). Sprint 13 (TASK-0003, a *proposal*) waits for his go. No gameplay work is authorized.

## Active tasks
- None. The next task takes TASK-0009.

## Completed (accepted by Jason, 2026-10-16)
- [TASK-0001](production/tasks/TASK-0001-shared-project-rules.md) Shared project rules in `PROJECT_RULES.md`
- [TASK-0002](production/tasks/TASK-0002-studio-onboarding.md) Studio onboarding and permanent task records
- [TASK-0007](production/tasks/TASK-0007-automatic-record-checks.md) Automatic record checks and initial Studio commit
- [TASK-0008](production/tasks/TASK-0008-toolkit-upgrade-review.md) Toolkit v0.2.0 upgrade and recordkeeping review

## Decisions
- [DEC-0007](decisions/DEC-0007-task-records.md) task-record layout: approved 2026-10-16.
- [DEC-0005](decisions/DEC-0005-kit-earned-by-style.md) the kit amending "full power from the start": wording left open for the game-design discussion.

## Blockers / unknowns
- None blocking. Open design questions Q-01 to Q-16 (Q-08 resolved) are in the [backlog](production/backlog.md#open-questions-for-jason).

## Next action
Hear Jason's discussion list. Sprint 13 only on his go.

## Record checks
Before committing Studio records, run `python -B tools/check_studio_records.py` (on Jason's PC,
the full Python313 path: manual → Automatic record checks). CI runs it before the game tests.

## Reference documents
- `PROJECT_RULES.md` (shared rules) · `SPRINTS.md` (plan and design bible) · `MEMORY.md` (session notes)
- `.studio/OPERATING_MANUAL.md` (workflow; task records) · `.studio/production/backlog.md` (index and IDs) · `.studio/decisions/README.md` (register)
