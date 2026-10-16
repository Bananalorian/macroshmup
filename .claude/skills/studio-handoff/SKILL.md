---
name: studio-handoff
description: Prepare session handoff.
user-invocable: true
---
# /studio-handoff

Read `.studio/STUDIO_CHARTER.md`, `.studio/OPERATING_MANUAL.md`, and `.studio/STATE.md`.

Update STATE based on actual evidence; identify Git status, active task, decisions, blockers, test results and next action. Write new dated handoff under .studio/memory/handoffs (named by UTC stamp, `YYYYMMDDTHHMMSSZ.md`, as `studio.py handoff` does). Recommend starting new session with /studio-resume.

Before ending, bring the active task's record (`.studio/production/tasks/`) up to date: status, checks run with their results, findings, unresolved issues. If `LEADERSHIP_HANDOFF.md` or `IMPLEMENTATION_REPORT.md` is about to be replaced, paste its contents, dated, into that task's Handoff log first. The session handoff links the task record rather than copying it.

After substantial changes, record an observable agent event and update relevant durable documentation. Respect director approval boundaries and never invent evidence.
