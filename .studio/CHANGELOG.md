# Studio Project Changelog

Project changes to be recorded here after they occur. Do not fabricate historical progress.
This log covers the Studio's own documents and procedures; game releases are in `/CHANGELOG.md`.

## 2026-10-16 — Onboarding and task records (TASK-0002)
- Onboarded: STATE and backlog filled from Git, CI and the project's documents; a ledger of the 31 releases before the Studio.
- Permanent task records: `production/tasks/` (template, TASK-0001, TASK-0002); IDs come from the backlog.
- Decision records DEC-0001 to DEC-0007; `SPRINTS.md`'s superseded rules marked, old wording kept.
- Manual: a map of which record holds what, task-record rules, copying the handoff and report before replacing them, `task` required on events, and what's enforced versus only written.
- Skills (`task`, `review`, `handoff`, `plan`, `decisions`, `resume`, `status`) and four agents now require the records. `/studio-review` no longer "accepts"; only Jason does.
- `PROJECT_RULES.md`: the definition of done includes the task record, and notes that CI doesn't enforce warning-free builds.

## 2026-10-16 — Shared project rules (TASK-0001, by Codex)
- `PROJECT_RULES.md` holds the shared rules; `CLAUDE.md` and `AGENTS.md` became short entry files.

## 2026-10-16 — Studio installed
- Toolkit cloned into `.ai-studio-toolkit/` and installed with `bootstrap/install.ps1`; existing `CLAUDE.md` and `AGENTS.md` preserved.

## 2026-10-16 — Toolkit v0.2.0 and recordkeeping review (TASK-0008)
- Toolkit checkout fast-forwarded to `95739b5`; seven local tests passed. No installer run over onboarding files.
- Permanent task record added; IDs, sections, file links and event references reviewed.
- Manual clarifies current summaries versus dated history and existing authorization; original reconciliation event linked to TASK-0001 with a new event.
- Existing record contents and game files preserved. Automated record enforcement remains proposed TASK-0007.

## 2026-10-16 — Automatic record checks and initial commit (TASK-0007)
- Added a Python standard-library validator and 26 regression tests for task records, IDs, status/evidence/review structure, events and document file links.
- Existing CI test job runs the checks before game tests; dependent Pages/Release jobs are gated by that job.
- Documented enforced structure versus remaining human review responsibilities.
- Jason authorized the initial Studio commit; toolkit clone is ignored, editor workspace excluded, and historical onboarding records preserved.

## 2026-10-16 — Acceptance recorded
- Jason accepted TASK-0001, TASK-0002, TASK-0007 and TASK-0008 and approved DEC-0007 (14:15 UTC). Each record gained a dated `accepted` entry and a closing line; earlier entries are unchanged.
- The backlog's task table splits into Open (the proposed sprints) and Accepted (linked records); STATE lists no active task. DEC-0005 stays open for the game-design discussion.
