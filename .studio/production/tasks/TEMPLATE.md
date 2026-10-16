# TASK-NNNN — Title

Copy this file to `TASK-NNNN-short-slug.md` (the next free ID in `../backlog.md`) and fill it in.
The rules are in `.studio/OPERATING_MANUAL.md` → Task records. In short: append, don't rewrite;
a dated correction beats an edited history; "not checked" is an honest result.

| Field | Value |
|---|---|
| ID | TASK-NNNN |
| Status | proposed / approved / implemented / verified / accepted (or blocked, rejected, superseded, cancelled) |
| Owner | who does the work (Claude Code, Codex, Jason) |
| Requested by | who asked, and where (quote or link) |
| Version / release | the vX.Y.Z it ships in, if any |
| Links | backlog row, decisions (DEC-NNNN), related tasks |

## Status history
Newest last. Each line: date, new status, who, evidence (a quote, a commit, a check below).
Only Jason approves and accepts.

- YYYY-MM-DD — proposed — by … — …

## Objective (as approved)
What Jason approved, in his words where possible.

## Scope
- In:
- Out:

## Constraints
Project rules that bind this task (PROJECT_RULES.md non-negotiables, decisions, "don't touch").

## Acceptance criteria
Numbered, so verification and review can cite them.

1.

## Decisions and rationale
Choices made during the task: what, why, who decided, the DEC-NNNN record if it outlives the task.

## Affected files and commits
Files changed, commits, tags. "Uncommitted" until it's in Git.

## Verification performed
Only checks actually run. For each: date, commit or working tree, command or method, result
(numbers, pass/fail), and where the evidence lives (it has to outlast the session: a committed
file, a commit, a tag, a CI run URL, or the numbers quoted here; `./shots/` is gitignored and the
scratchpad is temporary). Name checks not performed under "Not performed".

| # | Check | Run on | Result | Evidence | Criteria |
|---|---|---|---|---|---|
| V1 | | | | | |

Not performed:

## Review findings and disposition
Each finding: who reviewed, what was found, and its disposition: fixed (how, re-verified as Vn),
accepted as is (by whom, why), deferred (to TASK-NNNN), or rejected (why).

| # | Finding | Reviewer | Disposition |
|---|---|---|---|
| R1 | | | |

## Unresolved issues
Anything open at the end, with the task or question it moved to.

## Handoff log
Before `LEADERSHIP_HANDOFF.md` or `IMPLEMENTATION_REPORT.md` is replaced, paste its contents
here, dated, so the history survives.

## Final status
Status, date, and what it rests on (Jason's acceptance, or why it stopped).
