# DEC-0007 — Permanent task records in `.studio/production/tasks/`

| Field | Value |
|---|---|
| State | proposed. Jason approved the scope on 2026-10-16 (bounded documentation and Studio-procedure changes for recordkeeping); this layout is Claude Code's proposal and waits for his acceptance. |
| Date | 2026-10-16 |
| Task | [TASK-0002](../production/tasks/TASK-0002-studio-onboarding.md) |

**Question:** where does each task's approved objective, criteria, verification, review and
outcome live, so it survives later tasks?

**Options considered**
1. One file per task in the existing `.studio/production/`, indexed by `backlog.md`. *(chosen)*
2. Keep everything in `LEADERSHIP_HANDOFF.md` / `IMPLEMENTATION_REPORT.md`: those are rewritten
   per task, so history is lost.
3. A new top-level directory: Jason ruled out a parallel `harness/` and duplicate sources of
   truth.

**Decision (proposed)**
- `.studio/production/tasks/TASK-NNNN-slug.md` from `TEMPLATE.md`; IDs come from the backlog and
  are never reused.
- Statuses: proposed → approved → implemented → verified → accepted (plus blocked, rejected,
  superseded, cancelled). Only Jason approves or accepts.
- `LEADERSHIP_HANDOFF.md` and `IMPLEMENTATION_REPORT.md` stay as working copies for the current
  task. Each names its task and links its record, and is copied into the record's Handoff log
  before it's replaced.
- Reviews and verification go in the record. Evidence has to be durable (Git, CI, quoted
  numbers); `./shots/` and the scratchpad don't count.
- Releases before the Studio (v0.1.0 to v0.13.0) aren't backfilled as task records. The backlog
  keeps a ledger pointing to their tags, CI runs, `CHANGELOG.md` and `MEMORY.md`.

**Rationale:** reuses the Studio's own structure (`production/`, `qa/`, `decisions/`,
`memory/handoffs/`) and adds one folder of records, with no second source of truth. `CHANGELOG.md`
(for players) and `MEMORY.md` (session notes) keep their jobs.

**Affected:** `.studio/OPERATING_MANUAL.md`, `.studio/production/`, `.studio/qa/README.md`,
the handoff and report files, the `studio-*` skills and agents, and `PROJECT_RULES.md`'s
definition of done (one bullet).
