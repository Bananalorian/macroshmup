---
name: studio-task
description: Manage one task.
user-invocable: true
---
# /studio-task

Read `.studio/STUDIO_CHARTER.md`, `.studio/OPERATING_MANUAL.md`, and `.studio/STATE.md`.

Identify or draft task ID; establish objective, scope, constraints, expected files, acceptance criteria, tests and evidence. Confirm approval, then delegate appropriately; record events and state.

The task's permanent record is required (`.studio/OPERATING_MANUAL.md` → Task records). Take the next free ID from `.studio/production/backlog.md` and bump it; create `.studio/production/tasks/TASK-NNNN-slug.md` from `TEMPLATE.md` before asking for approval (status `proposed`), and add the backlog row. On approval, quote Jason's words in the status history. At each later step (implemented, verified, review, accepted or stopped), append to the record, then update the backlog row, `STATE.md` and `.studio/events.jsonl` (with `"task"`). Never mark a status without its evidence.

After substantial changes, record an observable agent event and update relevant durable documentation. Respect director approval boundaries and never invent evidence.
