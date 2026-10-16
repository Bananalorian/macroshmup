# DEC-0006 — Shared project rules live in PROJECT_RULES.md

| Field | Value |
|---|---|
| State | approved |
| Date | 2026-10-16 |
| Task | [TASK-0001](../production/tasks/TASK-0001-shared-project-rules.md) |

**Question:** `CLAUDE.md` and `AGENTS.md` were two full copies of the project rules, and they had
drifted. `AGENTS.md` still had the v0.6.4 bullet colors and lacked `PG_RUSH` / `PG_KIT`.

**Decision:** one file, `PROJECT_RULES.md`, holds the shared rules (design non-negotiables,
build, definition of done, debug hooks). `CLAUDE.md` and `AGENTS.md` are short entry files
that point to it and give each provider's Studio role. Shared rules are changed only in
`PROJECT_RULES.md`.

**Approval:** Jason with Codex, 2026-10-16 (Codex's event in `.studio/events.jsonl`; Jason's
onboarding message: "Codex and I reconciled the assistant instructions").

**Roles of the other documents** (the map is in `.studio/OPERATING_MANUAL.md` → Project records):
`SPRINTS.md` stays the plan and the design bible; Studio workflow lives in `.studio/`.
