# TASK-0001 — Shared project rules in PROJECT_RULES.md

| Field | Value |
|---|---|
| ID | TASK-0001 |
| Status | accepted (Jason, 2026-10-16) |
| Owner | Codex (implementation); Claude Code (verification) |
| Requested by | Jason, with Codex, 2026-10-16 (Codex's event in `.studio/events.jsonl`; Jason's onboarding message) |
| Version / release | none (instruction files only) |
| Links | [backlog](../backlog.md) · [DEC-0006](../../decisions/DEC-0006-shared-project-rules.md) · follows into [TASK-0002](TASK-0002-studio-onboarding.md) |

## Status history
- 2026-10-16 — approved — Jason (Codex's event: "User-approved instruction reconciliation"; Jason: "Codex and I reconciled the assistant instructions"). Recorded after the fact by Claude Code during onboarding; no record was opened before the work.
- 2026-10-16 12:50 UTC — implemented — Codex (event in `.studio/events.jsonl`).
- 2026-10-16 — verified — Claude Code (V1-V3 below).
- 2026-10-16T14:15:39Z — accepted — Jason: "I accept TASK-0001, TASK-0002, TASK-0007 and TASK-0008, and approve the task-record layout in DEC-0007." Recorded by Claude Code.

## Objective (as approved)
From Jason's message (2026-10-16):
- `PROJECT_RULES.md` holds the shared project rules.
- `CLAUDE.md` and `AGENTS.md` are short entry files pointing to it, with provider-specific Studio roles.
- The current bullet-color rules and the `PG_RUSH` / `PG_KIT` debug hooks are preserved.
- The maintenance is recorded in `.studio/events.jsonl`; the changes stay uncommitted; onboarding is left pending.

## Scope
- In: `PROJECT_RULES.md` (new), `CLAUDE.md`, `AGENTS.md`, one event.
- Out: onboarding, `SPRINTS.md`, source code, commits.

## Constraints
Preserve every existing rule; no gameplay changes; leave onboarding pending.

## Acceptance criteria
Not written down before the work. Reconstructed from the objective above:
1. `PROJECT_RULES.md` carries every rule from the v0.13.0 `CLAUDE.md`.
2. `CLAUDE.md` and `AGENTS.md` point to it and hold only entry instructions and roles.
3. Rule 4 (bullet colors by what to do, never teal) and the `PG_RUSH` / `PG_KIT` rows survive.
4. The event is logged; nothing is committed; STATE still says onboarding is required.

## Decisions and rationale
- One shared rules file instead of two drifting copies: the old `AGENTS.md` still carried the v0.6.4 color rule and lacked `PG_RUSH` / `PG_KIT` (diff in V2). See DEC-0006.
- "Claude writes the code" became provider-neutral wording that defers to the entry files and the Studio charter.

## Affected files and commits
- `PROJECT_RULES.md` (new), `CLAUDE.md`, `AGENTS.md` (rewritten), `.studio/events.jsonl` (one line).
- Uncommitted as of 2026-10-16 (on top of `cf49e4f`, v0.13.0).

## Verification performed

| # | Check | Run on | Result | Evidence | Criteria |
|---|---|---|---|---|---|
| V1 | `diff` of `git show HEAD:CLAUDE.md` against `PROJECT_RULES.md` | working tree, 2026-10-16 | The only differences: the new "Shared instructions and Studio startup" section, and "Claude writes the code" replaced by provider-neutral wording. Every other line is identical, rule 4 and the `PG_RUSH` / `PG_KIT` rows included. | this record | 1, 3 |
| V2 | `diff` of `HEAD:AGENTS.md` against `HEAD:CLAUDE.md` | v0.13.0 | The old `AGENTS.md` differed in rule 4 (the v0.6.4 hue list) and lacked the `PG_RUSH` / `PG_KIT` rows, so `PROJECT_RULES.md` was taken from the right copy. | this record | 1 |
| V3 | Read `CLAUDE.md`, `AGENTS.md`, `.studio/events.jsonl`, `.studio/STATE.md`, `git status` | working tree, 2026-10-16 | Both entry files are 3 short paragraphs pointing to `PROJECT_RULES.md`; the event is present; STATE read "ONBOARDING REQUIRED - NOT AUDITED"; the files were uncommitted. | this record | 2, 4 |

Not performed: nothing else applies (no code changed; `git diff HEAD -- src` is empty).

## Review findings and disposition

| # | Finding | Reviewer | Disposition |
|---|---|---|---|
| R1 | No task record or acceptance criteria existed before the work. | Claude Code, 2026-10-16 | Fixed: this record, criteria reconstructed and labelled as such. |
| R2 | `PROJECT_RULES.md`'s definition of done didn't mention the Studio records. | Claude Code, 2026-10-16 | Fixed in TASK-0002 (one bullet added). |

## Unresolved issues
- Commit: Jason's call (left uncommitted on purpose).
- 2026-10-16: resolved. Committed in `ea1072f` (TASK-0007); accepted by Jason. Nothing open.

## Handoff log
No leadership handoff or implementation report was written for this task.

## Final status
Verified 2026-10-16. Waiting on Jason's acceptance.

Accepted 2026-10-16T14:15:39Z by Jason (quoted in Status history). Closed.

## Commit authorization update — 2026-10-16 (TASK-0007)
Jason authorized committing the Studio files while excluding the toolkit clone. This record and its prior contents are included in that initial Studio commit. Earlier references to uncommitted work describe the state at the time; acceptance and design questions remain separate.
