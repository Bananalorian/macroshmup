# Studio Operating Manual — v0.1 (amended for PILGRIM on 2026-10-16, TASK-0002)

## Session boot protocol (mandatory)
1. Read `CLAUDE.md` (Claude) or `AGENTS.md` (Codex), then `PROJECT_RULES.md`, this manual, `STUDIO_CHARTER.md`, and `STATE.md`.
2. Inspect Git branch, HEAD, working tree and relevant project files. A stale or missing state triggers recovery/audit, not guesses.
3. Read the active task's record (`.studio/production/tasks/`), its spec and the newest handoff (`.studio/memory/handoffs/`) when applicable; distinguish facts, proposals, assumptions and unknowns.
4. Greet the director unless they requested state-first or silent resume. Offer: new idea, brainstorm, prototype, implement, evaluate existing project, resume.
5. Present a concise Studio Brief: verified work, current objective, blockers, next proposed action. Await approval before a new implementation task.

## Roles and delegation
Receptionist: onboarding and routing. Producer: priority, scope, sprint/task state, coordination. Technical Director: architecture and delegated implementation. Technical Writer: state, design specs, decisions, changes, reports. QA Lead: tests and evidence validation. Implementer: approved bounded code changes. Claude Code may spawn subagents using project-local `.claude/agents` when supported; otherwise work sequentially and report accurately. Do not claim agents communicated unless messages/events were explicitly produced.

## Work protocol
INTAKE -> CLARIFY/DECIDE -> SPECIFY -> APPROVE -> PLAN -> IMPLEMENT -> TEST -> REVIEW -> DOCUMENT -> HANDOFF. Brainstorm and audit without modifying source; approval precedes implementation. Bounded tasks require objective, relevant files, constraints, acceptance criteria, verification and reporting. On failure record exact errors and blockers, do not mark done. Every task has a record (see Task records); each step that changes its status is written there when it happens.

## Project records (one job each)
| Record | Holds |
|---|---|
| `PROJECT_RULES.md` | The shared project rules: non-negotiables, build, definition of done, debug hooks. |
| `SPRINTS.md` | The plan and the design bible; the decisions made before the Studio (§8). |
| `.studio/decisions/` | Decisions since onboarding, and any change to an older one (DEC-NNNN). |
| `.studio/production/backlog.md` | The task index and ID registry, open questions, and the ledger of releases before the Studio. |
| `.studio/production/tasks/` | One permanent record per task (TASK-NNNN). |
| `.studio/STATE.md` | Where things stand now. An index, not proof. |
| `.studio/LEADERSHIP_HANDOFF.md`, `.studio/IMPLEMENTATION_REPORT.md` | Working copies for the current task only. |
| `.studio/memory/handoffs/` | Dated session handoffs (`/studio-handoff`, `studio.py handoff`). |
| `.studio/events.jsonl` | Append-only activity log. |
| `.studio/qa/README.md` | How verification is done and recorded. |
| `.studio/CHANGELOG.md` | Changes to the Studio's own documents. |
| `CHANGELOG.md` | Release notes for players. |
| `MEMORY.md` | Session notes per release: tuning, lessons, what's next. |
| Git tags, CI runs, GitHub Releases | Proof of what shipped. |

## Task records
- **Where:** `.studio/production/tasks/TASK-NNNN-slug.md`, copied from `TEMPLATE.md`. The next free ID is at the top of the backlog. IDs are never reused, not even for cancelled tasks.
- **When:** a record exists before approval is asked for (status `proposed`), or as soon as work starts when Jason gives a task directly. A future sprint may hold a backlog ID without a record until it's put up for approval.
- **Statuses:**
  - `proposed`: written down, not approved.
  - `approved`: Jason approved the objective and scope; the record quotes or links his words.
  - `implemented`: the changes exist (files and commits listed). It says nothing yet about whether they work.
  - `verified`: the checks were run and their results are in the record. A failing or missing check keeps it at `implemented`.
  - `accepted`: Jason accepted the result after review, quoted or linked. Only Jason accepts. A sprint that's tagged with CI green has shipped (recorded under verification); that isn't acceptance.
  - `blocked`, `rejected`, `superseded` (by TASK-NNNN), `cancelled`: each with its reason.
- **Contents (all required):** objective as approved; scope; constraints; acceptance criteria; decisions and rationale; affected files and commits; verification performed, with results and durable evidence, plus the checks not performed; review findings and their disposition; unresolved issues; handoff log; final status.
- **Append-only:** add dated entries. Fix a mistake with a dated correction. Never delete a record or rewrite an earlier status, result or finding.
- **Durable evidence:** commits, tags, CI run URLs, files committed to the repo (screenshots in `docs/screenshots/`), or the numbers quoted in the record. `./shots/` is gitignored and scratchpads are temporary, so they don't count on their own.
- **Linked both ways:** backlog row ↔ record; `STATE.md` names the active task; every event about a task carries `"task": "TASK-NNNN"`; decision records name their task.
- **Before the Studio:** releases v0.1.0 to v0.13.0 aren't backfilled as records. The backlog's ledger points to their tags, CI runs, `CHANGELOG.md` and `MEMORY.md`.

## Coordination and observability
When an agent starts, hands over, identifies a blocker, finishes, or reviews work, append a structured event to `.studio/events.jsonl`. Schema: `{"at":"UTC ISO8601", "agent":"producer", "type":"started|message|blocked|completed|review|handoff", "message":"concise summary", "task":"TASK-NNNN"}`. `task` is required for any event about a task. `agent` names whoever actually did the work (`claude`, `codex`, or a `studio-*` agent that was really spawned). These explicit events drive the local dashboard. Do not record chain-of-thought or secrets. If the CLI is available (optional, Python 3.10+): `python .ai-studio-toolkit/studio.py --project . log --agent claude --type started --message "..." --task TASK-NNNN`; otherwise append the JSON line by hand.

## Documents and changes
Maintain `.studio/STATE.md` after meaningful milestones, approved decisions and handoffs. Put tasks in `.studio/production/backlog.md` with IDs, owner, status and a link to the record; keep acceptance criteria and evidence in the record. Decisions must say `proposed|approved|superseded`, include rationale and linked impacts. Keep `CHANGELOG.md` with actual changes. Do not rewrite historic decisions; supersede them. Escalate conflicts and unknowns to the owner.

## Handoff contract
Codex/leadership writes `.studio/LEADERSHIP_HANDOFF.md`: task ID and record link, objective, approved scope, constraints, specs, acceptance criteria, permissions, expected evidence. Claude/implementer reads it and returns `.studio/IMPLEMENTATION_REPORT.md`: task ID and record link, Git diff summary, task status, tests actually run, failures, unresolved questions, next action. Both files are working copies of the current task. **Before either is replaced, paste its contents, dated, into that task's record (Handoff log).** The Producer validates, then updates the record's status, the backlog and `STATE.md`. Session handoffs go to `.studio/memory/handoffs/<UTC stamp>.md`. When switching providers, do not both write to same project files concurrently; use Git working tree checks.

## Recovery and context budget
Never rely on token count or remembered chat for correctness. Checkpoint after each task and before context saturation; 500k is a user preference only where supported. A new session loads hot state, recent handoff, and task spec, with deeper archives on demand. Missing/outdated docs require recovery audit. A document describing a completed task without code/test evidence is not proof.

## Installation / tool policy
Existing-project audit is read-only for source and assets; studio documentation may be created with approval. Optional MCPs are proposed, reviewed, then installed explicitly. No scripts from unknown sources or automatic external services. Dashboard is read-only by design.

## What's enforced, and what's only written (onboarding snapshot)
Checked on 2026-10-16 against `.github/workflows/build.yml`, the test suite and `.ai-studio-toolkit/src/`.

**Enforced by tooling**
- CI on every push to `main` and every `v*` tag: `cargo test` has to pass, and the wasm and native (Windows, macOS, Linux) release builds have to compile. Pages deploys from `main`; a tag publishes the GitHub Release.
- `cargo test` enforces the rules that have tests. Examples: bullet hues (`every_bullets_color_says_what_to_do`), `sim/` never touching macroquad (`the_sim_never_touches_macroquad`), tests never touching the save file (`tests_never_touch_the_save_file`), the hitbox default (`the_options_change_and_are_saved`).

**Written only (nothing fails if it's skipped)**
- Warning-free builds: CI doesn't build with `-D warnings`; read the build output.
- Every CHANGELOG claim backed by a test, screenshot or bot run.
- The version bump, CHANGELOG, README, screenshots and `MEMORY.md` updates.
- Task records existing and complete, statuses and approvals recorded, `STATE.md`, backlog and events kept current.
- Copying the handoff and report into the record before they're replaced.
- The Studio CLI validates nothing: it appends events and writes checkpoints, and the dashboard only reads.

A check that would enforce the record rules is proposed as TASK-0007 in the backlog.

## Maintenance clarification — 2026-10-16 (TASK-0008)
- Append-only applies to dated status history, decisions, results and findings. The current Status summary may be updated to match the latest dated entry; never erase earlier entries or corrections.
- Existing explicit task authorization counts. Record Jason's request and proceed within that scope without requesting the same approval again. Approval to perform work is separate from acceptance of the result.
- Toolkit code is now v0.2.0. The installed project procedures remain the PILGRIM-specific onboarding version with these clarifications; generic templates were not installed over them. Automated record enforcement is still proposed as TASK-0007.

## Automatic record checks — TASK-0007 (2026-10-16)
This section supersedes the snapshot above for structural record enforcement. Run `python -B tools/check_studio_records.py` from the repository root. It needs Python 3.10+ and no toolkit clone or packages. On Windows, invoke the real Python executable if `python` resolves to the Store alias.

Enforced: unique task IDs and record filenames; next-ID consistency; required nonempty sections; heading/metadata IDs; recorded approval and latest-status consistency; backlog links/statuses; records for non-proposed tasks; verification/review rows for verified/accepted tasks; valid event JSON/UTC timestamps and task references; existing Markdown destination files in Studio/provider documents. Reserved, untouched proposed backlog items may lack records. Session/installation events without a task must explicitly set `scope` to `session` or `installation`. One exact historical event is exempt by content fingerprint; its later task association remains logged.

The existing CI `test` job runs the checker and `python -B -m unittest discover -s tools -p "test_studio_records.py" -v` before Cargo tests. A failure blocks dependent Pages and Release jobs. Locally run the checker before committing records and the regression suite when changing the checker. CI enforcement begins when this commit is pushed; no local Git hooks or branch protection were installed.

Still reviewed manually: truth of evidence and approvals, genuinely complete scope/content, historical rewrites, archival before replacing handoffs, remote links and Markdown anchors, and gameplay-specific definition of done. A structural pass is not Jason's acceptance. Do not use this checker to label these semantic requirements automatically enforced.
