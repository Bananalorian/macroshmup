# TASK-0007 — Automatic Studio record checks and initial Studio commit

| Field | Value |
|---|---|
| ID | TASK-0007 |
| Status | verified; awaiting Jason's acceptance |
| Owner | Codex |
| Requested by | Jason, current chat, 2026-10-16 |
| Version / release | Studio maintenance; no game release |
| Links | [backlog](../backlog.md) · [review](TASK-0008-toolkit-upgrade-review.md) |

## Status history
- 2026-10-16T13:50:19+00:00 — approved — Jason: "ok let's implement TASK-0007 so the task recordkeeping is automatically checked, then commit the studio files without including the toolkit clone"

- 2026-10-16T13:53:01+00:00 — implemented — checker, regression suite and CI integration added.
- 2026-10-16T13:53:01+00:00 — verified — 26 regression tests passed; real records returned zero errors (V1–V3).

## Objective (as approved)
Implement automatic task-record checks and commit the Studio setup, excluding the nested toolkit clone.

## Scope
- In: a dependency-free Python record checker and regression tests, existing CI test job, Studio procedure/evidence updates, `.gitignore`, and the initial Studio documentation commit.
- Out: gameplay implementation, upstream toolkit changes, pushing to GitHub, and the unrelated `macroshmup.code-workspace` file.

## Constraints
Preserve Claude's onboarding history, task IDs and decisions. Existing acceptance questions are not resolved by approval to commit. Exclude the nested clone and local cache files. Record actual test results and stage explicit paths.

## Acceptance criteria
1. Checker fails with actionable errors for missing record sections, duplicate IDs and events referencing nonexistent records.
2. It checks backlog/record status and ID consistency, evidence/review structure, event JSON/timestamps and local document file links.
3. Regression tests demonstrate valid data passes and broken fixtures fail; proposed future tasks may reserve IDs without records.
4. CI automatically runs the checker and regression suite before the existing game tests, gating dependent Pages/Release jobs.
5. Commit the Studio setup and reconciliation documents without the toolkit clone, workspace file, caches or gameplay changes.

## Decisions and rationale
- Keep the checker under `tools/` so it works without the ignored toolkit clone or third-party packages.
- Preserve one historical taskless event through an exact canonical-content SHA-256 exception. A subsequent event already links it to TASK-0001; new taskless task events fail. Explicit session/installation events may declare a non-task scope.
- Keep record content truth, human acceptance and archival-before-replacement as review responsibilities; a structural validator cannot certify them.
- Add checks to the existing CI test job, which deployment and release already depend on. No Git hook installation or branch-protection change is needed.

## Affected files and commits
Implemented in the commit containing this record. Commit scope includes `.studio/`, `.claude/`, `PROJECT_RULES.md`, both provider entries, Claude's `SPRINTS.md` and `MEMORY.md` reconciliation, checker/tests, CI and ignore rules. No push authorized by this request.

## Verification performed
| # | Check | Run on | Result | Evidence | Criteria |
|---|---|---|---|---|---|
| V1 | `python -B -m unittest discover -s tools -p "test_studio_records.py" -v` | Windows, Python 3.13.4, 2026-10-16 working tree | 26 passed, 0 failed, 0.960 seconds | Exact count and command here; committed regression suite | 1, 2, 3 |
| V2 | `python -B tools/check_studio_records.py` | Current project working tree, 2026-10-16 | 0 errors | Exact result here; rerun before commit | 1, 2 |
| V3 | Inspect `.github/workflows/build.yml` dependencies and new steps | Working tree | Regression suite and validator precede Cargo tests in `test`; `pages` and `release` depend on `test`, so failed record checks block those jobs | Committed workflow diff | 4 |
| V4 | Inspect staged paths, Git index modes, ignore rules and whitespace before commit | Initial Studio commit | Passed: 53 explicit files staged; no toolkit path, gitlink, workspace file, cache or gameplay source; `git diff --cached --check` passed | Commit file list; exact staging commands in session | 5 |

The first local test pass exposed a Windows default-encoding error in the temporary file-copy helper and a test fixture read. Corrected both to explicit UTF-8; the subsequent 26-test pass and real-record validation above use the corrected files.

Not performed: game builds/playtesting; remote CI (no push requested).

## Review findings and disposition
| # | Finding | Reviewer | Disposition |
|---|---|---|---|
| R1 | Windows default encoding corrupted punctuation while copying the initial checker/fixture. | Codex | Fixed with explicit UTF-8; V1 and V2 pass. |
| R2 | Record checks must not require records for untouched future sprint proposals. | Codex | Proposed backlog-only IDs allowed; active tasks and task events require permanent records; regression coverage included. |
| R3 | Historical taskless event must survive without exempting future malformed events. | Codex | Exact canonical SHA-256 exception only; changed-event regression test fails as expected. |
| R4 | Empty review dispositions, invalid scope types and duplicate required headings could slip through or crash validation. | Codex | Added validation and regression cases; V1 passes. |
| R5 | A structural checker cannot prove evidence truth, acceptance authenticity, or that old handoffs were archived. | Codex | Documented remaining manual review responsibilities. No claim of full semantic enforcement. |
| R6 | Toolkit clone and unrelated editor workspace must not enter the game commit. | Codex | Toolkit ignore rule and explicit staging allowlist; index inspection required before commit. |

## Unresolved issues
Jason's prior acceptance/design questions remain. Structural checks cannot validate the truth of recorded evidence or detect every historical rewrite.

## Handoff log
Current TASK-0002 implementation report and leadership handoff retained; this task is recorded here.

## Final status
Verified 2026-10-16T13:53:01+00:00. Included in the initial Studio commit with its existing onboarding documents. Jason's acceptance remains separate; no push or game release.
