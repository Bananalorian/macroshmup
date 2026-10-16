# Decision Records

One file per decision: `DEC-NNNN-short-slug.md`, numbered in order, never reused. Each records
ID, date, state, question, options, decision, rationale, approval (a quote or reference to
Jason's word), what it supersedes or is superseded by, and the documents, tasks, code and tests
it touches.

**States:** `proposed` (waiting on Jason) · `approved` (Jason's call, with evidence) ·
`superseded` (replaced; the record stays, with a link to what replaced it).

**Rules**
- Never erase or rewrite a decision. Supersede it: write a new record, set the old one's state
  to `superseded by DEC-NNNN` and leave its text as it was.
- Where a decision changes text in `SPRINTS.md` or `PROJECT_RULES.md`, keep the old wording
  visible there, marked *Superseded*, with a link to the record.
- Design decisions are Jason's. A record written by an AI says who decided and where; if that
  can't be shown, the state is `proposed`.

**Before the Studio (v0.1.0 to v0.13.0):** decisions live in `SPRINTS.md` §8 "Decisions",
the "(Jason's call, vX)" notes in `PROJECT_RULES.md` and `SPRINTS.md`, and the "Jason's calls"
notes per release in `MEMORY.md`. They stand as they are. A record is written here when one of
them is changed, superseded or found in conflict.

## Register

| ID | Decision | State | Date |
|---|---|---|---|
| [DEC-0001](DEC-0001-two-hues.md) | Only two hues, red and teal (the plan's pillar) | superseded by DEC-0002 | 2026-10-10 (v0.1.0) |
| [DEC-0002](DEC-0002-five-bullet-hues.md) | Enemy bullets in five hues, teal included | superseded by DEC-0003 | 2026-10-11 (v0.6.4) |
| [DEC-0003](DEC-0003-bullet-color-by-action.md) | Enemy bullets colored by what to do; never teal | approved | 2026-10-13 (v0.12.0) |
| [DEC-0004](DEC-0004-hitbox-always-shown.md) | The hitbox shows all the time by default | approved | 2026-10-13 (v0.10.2) |
| [DEC-0005](DEC-0005-kit-earned-by-style.md) | The ship's kit is earned by style; no item pickups | approved | 2026-10-13 (v0.13.0) |
| [DEC-0006](DEC-0006-shared-project-rules.md) | Shared project rules live in PROJECT_RULES.md | approved | 2026-10-16 |
| [DEC-0007](DEC-0007-task-records.md) | Permanent task records in `.studio/production/tasks/` | approved (Jason, 2026-10-16; proposed until then) | 2026-10-16 |
