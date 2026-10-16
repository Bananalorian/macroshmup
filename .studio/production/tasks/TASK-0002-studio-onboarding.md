# TASK-0002 — Studio onboarding and permanent task records

| Field | Value |
|---|---|
| ID | TASK-0002 |
| Status | accepted (Jason, 2026-10-16) |
| Owner | Claude Code, working alone and in sequence (no `studio-*` subagents were spawned) |
| Requested by | Jason, in chat, 2026-10-16 (quoted below) |
| Version / release | none (documents and Studio procedure only) |
| Links | [backlog](../backlog.md) · [DEC-0001](../../decisions/DEC-0001-two-hues.md) to [DEC-0007](../../decisions/DEC-0007-task-records.md) · follows [TASK-0001](TASK-0001-shared-project-rules.md) · proposes [TASK-0007](../backlog.md) |

## Status history
- 2026-10-16 — approved — Jason: "You have my approval to make the bounded documentation and Studio-procedure changes needed for this onboarding and recordkeeping setup."
- 2026-10-16 ~12:58 UTC — started — Claude Code. Not logged to `events.jsonl` when it happened; see A13.
- 2026-10-16 ~13:20 UTC — implemented — Claude Code (files below).
- 2026-10-16 ~13:25 UTC — verified — Claude Code: V1 to V12 run, results below. Criterion 7 (the brief) is delivered in chat with this status.
- 2026-10-16T14:15:39Z — accepted — Jason: "I accept TASK-0001, TASK-0002, TASK-0007 and TASK-0008, and approve the task-record layout in DEC-0007." Recorded by Claude Code. DEC-0007 is approved by the same words; DEC-0005 stays open for the game-design discussion.

## Objective (as approved)
Jason's message, 2026-10-16, verbatim:

> Please start `/studio` onboarding for `D:\MY_PROJECTS\MY_VIDEO_GAMES\macroshmup`. First reread the updated `CLAUDE.md`, `PROJECT_RULES.md`, and Studio startup documents.
>
> During onboarding, please check these gaps:
>
> 1. **Conflicting documentation:** Compare `SPRINTS.md`, `MEMORY.md`, shared rules, approved decisions, and actual code. In particular, `SPRINTS.md` still says "only two hues exist, red and teal," while the current bullet rule specifies red for aimed shots, blue for fixed shapes, violet for spinning/bending shots, amber for random scatter, and never teal. Reconcile clearly superseded guidance using evidence; ask me about genuinely unresolved design decisions. Preserve historical decisions and mark them superseded rather than erasing history.
> 2. **Current project position:** Establish what has actually shipped, what remains proposed, and the next objective requiring my approval. Populate Studio state and backlog from verified evidence. Don't treat old plans or empty templates as proof of completed work.
> 3. **Permanent task records:** Make sure every task has a unique ID and a durable record of its approved objective, scope, constraints, acceptance criteria, decisions and rationale, affected files/commits, verification actually performed and its results, review findings and disposition, unresolved issues, and final status. Clearly distinguish proposed, approved, implemented, verified, and accepted work.
> 4. **Preserved history:** Ensure completed task records, reviews, and verification evidence survive subsequent tasks. If `LEADERSHIP_HANDOFF.md` and `IMPLEMENTATION_REPORT.md` remain current-task summaries, link them to permanent task-specific records before replacing their contents. Keep Studio state, backlog, and events linked to those records.
> 5. **Workflow consistency:** Check that task, review, and handoff procedures explicitly require these records. Distinguish written instructions from checks actually enforced by tooling.
>
> You have my approval to make the bounded documentation and Studio-procedure changes needed for this onboarding and recordkeeping setup. Reuse the existing Studio structure; don't create a parallel `harness/` directory or duplicate sources of truth. Preserve existing work, don't invent historical approvals or test results, and don't start gameplay implementation.
>
> Finish with a concise onboarding brief: verified project status, conflicts resolved or still needing my decision, files changed, where permanent task evidence will live, checks performed, and the next proposed action.

(The message also summarized TASK-0001; that part is quoted in TASK-0001's record.)

## Scope
- In: `.studio/` (state, backlog, task records, decisions, QA, manual, handoff and report files, Studio changelog, config), the `studio-*` skills and agents in `.claude/`, `PROJECT_RULES.md` (definition of done), `SPRINTS.md` (superseded markings only), `MEMORY.md` (one session note).
- Out: gameplay code, art, sound, tuning; `CHANGELOG.md` and the version (no release); `CLAUDE.md` and `AGENTS.md` (TASK-0001's); the toolkit clone `.ai-studio-toolkit/`; commits.

## Constraints
From the objective: reuse the Studio's structure, no `harness/` directory, no second source of truth; preserve existing work; mark superseded guidance and keep the old wording; never invent approvals or results; no gameplay work. From `PROJECT_RULES.md`: Jason decides design; *proposal* items wait for him. From the auto-memory: stage files by name, never `git add -A`.

## Acceptance criteria
From the five items in the objective:
1. Conflicts between `SPRINTS.md`, `MEMORY.md`, the shared rules, approved decisions and the code are found. Clearly superseded guidance is reconciled with evidence and marked superseded, with the old wording kept. Genuinely open design questions go to Jason.
2. What shipped, what's proposed, and the next objective needing his approval are established from evidence; `STATE.md` and the backlog are filled from it; old plans and templates aren't counted as proof.
3. Every task has a unique ID and a durable record with the listed fields, and the statuses separate proposed, approved, implemented, verified and accepted.
4. Records, reviews and verification survive later tasks. The handoff and report files link to the records and get copied into them before they're replaced. STATE, backlog and events link to the records.
5. The task, review and handoff procedures require the records, and the manual separates written rules from what tooling enforces.
6. Bounded: no `harness/`, no duplicate source of truth, no gameplay change, no invented approvals or results.
7. A concise onboarding brief is delivered.

## Decisions and rationale
- **Where records live:** one file per task in `.studio/production/tasks/`, indexed by the backlog ([DEC-0007](../../decisions/DEC-0007-task-records.md), proposed, inside Jason's approved scope). It reuses the Studio's `production/` folder; the handoff and report files stay working copies.
- **Releases before the Studio aren't backfilled as task records.** Writing 31 records after the fact would invent approvals and acceptance that were never written down. The backlog instead has a ledger pointing to each tag, its CI run, `CHANGELOG.md` and `MEMORY.md`, and says plainly what's missing (acceptance, some approvals).
- **Superseded guidance reconciled** where Jason's own later call is on record: DEC-0001 → DEC-0002 → DEC-0003 (bullet colors), DEC-0004 (hitbox always shown), DEC-0005 (the kit amends "full power from the start"). In `SPRINTS.md` the old wording stays, marked *Superseded* or *Amended*, with links.
- **Future sprints get IDs now** (TASK-0003 to TASK-0006) so the plan can be referenced, but get records only when put up for approval.
- **Pre-Studio decisions stay where they are** (`SPRINTS.md` §8, "Jason's call" notes); a DEC record is written only when one changes.
- **Nothing committed:** Codex left TASK-0001 uncommitted on purpose, and committing is Jason's call (Q-08).

## Affected files and commits
Uncommitted, on top of `cf49e4f` (v0.13.0), as of 2026-10-16:
- New: `.studio/production/tasks/TEMPLATE.md`, `TASK-0001-shared-project-rules.md`, `TASK-0002-studio-onboarding.md`; `.studio/decisions/DEC-0001` to `DEC-0007`.
- Rewritten from the empty templates: `.studio/STATE.md`, `.studio/production/backlog.md`, `.studio/decisions/README.md`, `.studio/qa/README.md`, `.studio/LEADERSHIP_HANDOFF.md`, `.studio/IMPLEMENTATION_REPORT.md`, `.studio/CHANGELOG.md`.
- Amended: `.studio/OPERATING_MANUAL.md` (records map, task records, handoff copying, events need `task`, what's enforced), `.studio/config.json`, `.studio/events.jsonl` (appended).
- `.claude/skills/`: `studio-task`, `studio-review`, `studio-handoff`, `studio-plan`, `studio-decisions`, `studio-resume`, `studio-status`. `.claude/agents/`: `studio-producer`, `studio-implementer`, `studio-qa-lead`, `studio-technical-writer`.
- `PROJECT_RULES.md`: definition of done (the warnings note, one bullet for the task record).
- `SPRINTS.md`: §1 Color, §2 the PNG-override note, §3 hitbox, power-ups and bullets, §5 debug-hooks pointer, §8 Decision 3, the UX plan's status line.
- `MEMORY.md`: one session note at the top.

## Verification performed

| # | Check | Run on | Result | Evidence | Criteria |
|---|---|---|---|---|---|
| V1 | Git position: `git status -sb`, `git log`, `git rev-parse <tag>^{commit}` for every tag, `git diff --stat HEAD -- src tools web .github Cargo.toml Cargo.lock build_web.py` | 2026-10-16 | On `main`, HEAD `cf49e4f` = `origin/main` (nothing ahead or behind). 31 tags, v0.1.0 to v0.13.0, each resolves to a commit. No source, tool, web or CI file differs from HEAD. | backlog ledger (commit column) | 2 |
| V2 | CI: `gh run list --limit 200`, `gh run view` (v0.13.0 tag and main runs), `gh release view v0.13.0` | 2026-10-16 | Every tag has a successful run. v0.8.0 also has a failed run and v0.8.5 a cancelled one, each followed by a pass. v0.13.0 tag run: tests, browser, 3 desktop builds and GitHub Release all passed (Pages skipped, as designed for tags). v0.13.0 main run: Pages passed. Release v0.13.0 has 4 assets (browser html; Linux, macOS, Windows zips). | GitHub Actions run history | 2 |
| V3 | `cargo test` (test profile; rustc and cargo 1.91.1, the pinned toolchain) | working tree, source identical to `cf49e4f` | Recompiled `pilgrim v0.13.0`; no compiler warnings printed. 284 passed, 0 failed, 2 ignored (`jawsthic_sheet`, `pilgrim_sheet`: review-sheet generators run by hand). Includes `every_bullets_color_says_what_to_do`, `the_options_change_and_are_saved`, `the_sim_never_touches_macroquad`, `tests_never_touch_the_save_file`. | the numbers here (the raw log was in the session scratchpad) | 1, 2 |
| V4 | Code against the documents: `src/sim/bullets.rs` `Hue`, `src/sim/pattern.rs` `Gun::hue()` and the stress filler, `src/storage.rs` defaults, a search of `src/` for any PNG loading, `src/tune.rs` `RANKS` / `BOMB_DIP` / `JAWS_SLOWMO` | `cf49e4f` | Four hues, no teal; hue derived from shape and behavior. Random hues only in the sandbox stress filler (`pattern.rs` around line 216). `hitbox_always: true` by default. No PNG loader (only exports to `./shots/`). The constants exist: `RANKS` [90, 75, 55, 35], `BOMB_DIP` 24, `JAWS_SLOWMO` 48. | DEC-0003, DEC-0004; backlog Q-02, Q-06, Q-07 | 1 |
| V5 | Where the conflicting text came from: `git log -S` on `SPRINTS.md`; `git show v0.12.0~1:CLAUDE.md`, `v0.12.0~1:AGENTS.md`, `v0.12.0:CLAUDE.md` | 2026-10-16 | "only two hues exist", "none; full power from the start" and "shows while focused" all entered in `8bcae2a` (v0.1.0). Before v0.12.0 rule 4 listed five hues in both files; v0.12.0 changed only `CLAUDE.md`. | DEC-0001, DEC-0002 | 1 |
| V6 | Documents read in full: `SPRINTS.md` (777 lines), `MEMORY.md` (987 lines), the `CHANGELOG.md` headings, `README.md` (searched for stale rules), `PROJECT_RULES.md`, `CLAUDE.md`, `AGENTS.md`, every `.studio/` file, skill and agent | 2026-10-16 | Conflicts found: A1 to A12 below. `README.md` is current to v0.13.0 and has none. | this record | 1, 2, 5 |
| V7 | TASK-0001 checked (its V1 to V3) | 2026-10-16 | `PROJECT_RULES.md` preserves every rule; details in that record. | [TASK-0001](TASK-0001-shared-project-rules.md) | 1 |
| V8 | What tooling enforces: `.github/workflows/build.yml`; `.ai-studio-toolkit/src/ai_studio/core.py`, `cli.py`, `dashboard.py` | 2026-10-16 | CI: `cargo test`, wasm and native release builds, Pages from `main`, a Release on tags; no `-D warnings`. Studio CLI: `log` appends an event, `handoff` writes `.studio/memory/handoffs/<stamp>.md`, the dashboard reads STATE and events; nothing validates records. | manual → What's enforced | 5 |
| V9 | Records check, a one-off script run with Python 3.13 (`py -I`): relative links and anchors across `.studio/`, `.claude/` and the five root documents; record and decision headings match their IDs; every mentioned TASK ID has a backlog row below the next free ID; backlog rows link their records; every record and the template have all required sections; every event parses and names a known task | working tree, 2026-10-16 13:20 UTC, after the events were appended | 45 files, 62 links, 2 records, 7 backlog IDs (next 0008), 7 decisions: no problems. (The first run flagged the "Next free ID: TASK-0008" mentions themselves; the script was fixed to skip that ID.) | the result here; the script lived in the session scratchpad and isn't kept. Keeping such a check is TASK-0007. | 3, 4 |
| V10 | V9's script on a scratch copy with an anchor and a DEC link deliberately broken | 2026-10-16 | Both planted errors reported, exit 1. (Links into `docs/`, which wasn't copied, were reported too, as expected.) | the result here | 3, 4 |
| V11 | `git diff -U0 SPRINTS.md` and `git diff --stat SPRINTS.md MEMORY.md`; `PROJECT_RULES.md`'s two edits re-read | 2026-10-16 | `SPRINTS.md`: exactly the 8 intended hunks (§1, §2, §3 ×3, §5, §8, the UX plan), each old rule kept in the text. `MEMORY.md`: 19 lines added at the top, nothing removed. | the diff | 1, 6 |
| V12 | No gameplay or source change: `git status --short`, `git diff --stat HEAD -- src tools web .github Cargo.toml Cargo.lock build_web.py` | 2026-10-16, after all edits | Empty source diff. Changed or new: `AGENTS.md`, `CLAUDE.md` (TASK-0001), `SPRINTS.md`, `MEMORY.md`, `PROJECT_RULES.md`, `.studio/`, `.claude/`; no `harness/` directory. | the commands' output | 6 |

Not performed: native release and wasm builds, `PG_SIM`, `PG_AUTO`, screenshots, play. No source changed, and CI built and tested `cf49e4f`. The per-release claims in `CHANGELOG.md` and `MEMORY.md` (test counts, `PG_SIM` numbers, screenshots) weren't re-run; the ledger says so.

## Review findings and disposition
These are audit findings from Claude Code's own pass. No independent review (Jason or Codex) has happened yet.

| # | Finding | Reviewer | Disposition |
|---|---|---|---|
| A1 | `SPRINTS.md` §1 still said "only two hues exist, red and teal". | Claude Code | Fixed: current wording, old wording kept as *Superseded*, DEC-0001 to DEC-0003. |
| A2 | `SPRINTS.md` §3 bullets "in the red family". | Claude Code | Fixed the same way (DEC-0003). |
| A3 | `SPRINTS.md` §3 "the hitbox ... shows while focused"; since v0.10.2 it always shows by default. | Claude Code | Fixed: DEC-0004. |
| A4 | `SPRINTS.md` §3 and §8 Decision 3 "full power from the start" against v0.13.0's kit, earned by style. | Claude Code | Fixed as *Amended* with DEC-0005, on the evidence of Jason's Sprint 12 picks. Jason to confirm the wording. |
| A5 | `SPRINTS.md` §5 debug-hooks table is an older copy (no `options`, `PG_RUSH`, `PG_KIT`), a second source beside `PROJECT_RULES.md`. | Claude Code | Fixed: a pointer to `PROJECT_RULES.md`; the table kept as the Sprint 0 plan. |
| A6 | The UX plan's heading still read *proposal* though Sprints 8 to 12 were approved and shipped. | Claude Code | Fixed: a status line; Sprint 13 stays *proposal*. |
| A7 | `SPRINTS.md` §2 says PNGs in `art/` replace coded art; no such loader exists. | Claude Code | Noted in `SPRINTS.md` as planned, not built; question Q-07. |
| A8 | `MEMORY.md` v0.12.0 says "CLAUDE.md rule 4 and SPRINTS §2 updated"; `AGENTS.md` and `SPRINTS.md` §1 and §3 were missed. | Claude Code | Fixed by TASK-0001 and A1/A2. |
| A9 | `/studio-review` said to "mark accepted", against "only Jason accepts". | Claude Code | Fixed: criteria are marked met, not met or uncertain, and acceptance is Jason's. |
| A10 | `/studio-handoff` writes to `.studio/memory/handoffs/`, which the manual didn't mention. | Claude Code | Fixed: the manual names it; the folder is created by the first handoff. |
| A11 | Nothing enforces task records, events or the definition-of-done steps; CI doesn't fail on warnings. | Claude Code | Documented in the manual. Enforcement deferred to TASK-0007 (proposed). |
| A12 | `SPRINTS.md` §5 "Modules" lists the Sprint 0 layout (e.g. `world.rs`, which doesn't exist); `src/` has grown. | Claude Code | Accepted as is: it's the plan's starting layout, low impact. Jason may want it marked. |
| A13 | This task's start wasn't logged as an event when it happened. | Claude Code | Accepted as is: the status history records the time; events from here on carry `"task"`. |

## Unresolved issues
- Jason's acceptance of this task, of DEC-0007's layout and of DEC-0005's wording.
- Committing: Q-08.
- Open design questions Q-01 to Q-16 in the backlog.
- 2026-10-16 update: this task accepted and DEC-0007 approved (Jason, 14:15 UTC); committing resolved by TASK-0007 (`ea1072f`). Still open, outside this task: DEC-0005's wording (left for the game-design discussion) and Q-01 to Q-16 (Q-08 now resolved).

## Handoff log
- 2026-10-16 — `LEADERSHIP_HANDOFF.md`: no handoff for this task (Jason briefed Claude Code directly; the brief is quoted above). The file was rewritten from the empty template to name its record rules. The template read:
  > # Leadership Handoff
  > Status: EMPTY — no approved implementation request.
  > (Objective, Approved scope, Constraints, Design references, Acceptance criteria: TBD. Approval and next action: Await director approval.)
- 2026-10-16 — `IMPLEMENTATION_REPORT.md`, contents before it was replaced with this task's summary:
  > # Implementation Report
  > Status: NO IMPLEMENTATION YET
  > (Tasks / commit / file changes, Tests actually performed, Acceptance verification, Unresolved issues and next action: TBD.)

## Final status
Open: waiting on Jason's acceptance.

Accepted 2026-10-16T14:15:39Z by Jason (quoted in Status history). Closed.

## Independent recordkeeping review — 2026-10-16T13:37:16+00:00
Codex reviewed the installed task records and procedures during [TASK-0008](TASK-0008-toolkit-upgrade-review.md). Required record sections, file links, event task references, review/verification procedures and archive-before-replacement rules checked. Two procedure clarifications and the original event association were added without rewriting this history. See TASK-0008 V1–V5 and R1–R7 for scope and limits; this is not a rerun of all game/release tests or Jason's acceptance.

## Commit authorization update — 2026-10-16 (TASK-0007)
Jason authorized committing the Studio files while excluding the toolkit clone. This record and its prior contents are included in that initial Studio commit. Earlier references to uncommitted work describe the state at the time; acceptance and design questions remain separate.
