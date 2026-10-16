# Studio State

Project: PILGRIM (repo `macroshmup`)
Status: ONBOARDED 2026-10-16 (TASK-0002); waiting on Jason's acceptance
Last verified: 2026-10-16 by Claude Code (evidence in TASK-0002, "Verification performed")

This file is an index, not proof. Every claim below points to its evidence.

## Verified position
- **Shipped:** v0.13.0 "Augments & shield" (Sprint 12), commit `cf49e4f`, tag `v0.13.0`, 2026-10-13. CI green on the tag (tests, browser and three desktop builds, Release) and on `main` (Pages). The GitHub Release has 4 downloads. TASK-0002 V1, V2.
- **History:** 31 tagged releases, v0.1.0 to v0.13.0, each with a passing CI run: [ledger](production/backlog.md#releases-before-the-studio). Sprints 0 to 12 are done; Jason's acceptance wasn't recorded before the Studio.
- **Tests at HEAD:** `cargo test` 284 passed, 0 failed, 2 ignored, re-run 2026-10-16 (TASK-0002 V3).
- **Git:** Studio setup, onboarding and TASK-0007 are included in the local commit containing the TASK-0007 record, based on game commit `cf49e4f`. No push or game release; gameplay source unchanged. Toolkit clone is ignored; `macroshmup.code-workspace` remains outside the commit.

## Current objective
Close onboarding, then hear Jason's list. After v0.13.0 he said he has "lots to discuss"
(`MEMORY.md`, v0.13.0). Sprint 13 (TASK-0003, a *proposal*) waits for his go.

## Active tasks
- [TASK-0001](production/tasks/TASK-0001-shared-project-rules.md) Shared project rules: verified; awaiting acceptance.
- [TASK-0002](production/tasks/TASK-0002-studio-onboarding.md) Studio onboarding and task records: awaiting acceptance.

## Decisions waiting on Jason
- [DEC-0005](decisions/DEC-0005-kit-earned-by-style.md): confirm the wording that amends "full power from the start".
- [DEC-0007](decisions/DEC-0007-task-records.md): accept the task-record layout.
- Q-08 resolved by Jason: commit Studio files and exclude the toolkit clone (TASK-0007).

## Blockers / unknowns
- None blocking. Open design questions Q-01 to Q-16 are in the [backlog](production/backlog.md#open-questions-for-jason).

## Next action
Jason reviews the onboarding brief; then his discussion list; then Sprint 13 on his go.

## Reference documents
- `PROJECT_RULES.md` (shared rules) · `SPRINTS.md` (plan and design bible) · `MEMORY.md` (session notes)
- `.studio/OPERATING_MANUAL.md` (workflow; task records) · `.studio/production/backlog.md` (index and IDs) · `.studio/decisions/README.md` (register)

## Toolkit maintenance — TASK-0008
Jason authorized the toolkit update and recordkeeping review on 2026-10-16. [Record](production/tasks/TASK-0008-toolkit-upgrade-review.md). Verified: toolkit v0.2.0 at `95739b5`, 7 tests passed, recordkeeping audit passed. Installed onboarding procedures preserved with narrow clarifications; prior acceptance/design questions remain unchanged. The later TASK-0007 implements structural record checks; see the section below.

## Automatic record checks — TASK-0007
Jason authorized implementation and committing Studio files, excluding the toolkit clone. [Task record](production/tasks/TASK-0007-automatic-record-checks.md). Verified: 26 regression tests passed and the project record check returned zero errors. CI runs both before game tests and gates dependent deployments/releases. Initial Studio files are committed locally, excluding the toolkit clone and unrelated workspace file. Acceptance of prior tasks is separate.
