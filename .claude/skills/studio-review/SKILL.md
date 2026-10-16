---
name: studio-review
description: Review implementation.
user-invocable: true
---
# /studio-review

Read `.studio/STUDIO_CHARTER.md`, `.studio/OPERATING_MANUAL.md`, and `.studio/STATE.md`.

Read approved leadership handoff and actual implementation report; inspect source diff and test results; assess each acceptance criterion; mark each met, not met or uncertain.

Read the task's record (`.studio/production/tasks/`) first; review against its acceptance criteria and its "Verification performed" table, re-running checks where feasible. Write every finding into the record's "Review findings and disposition" table (fixed, accepted as is, deferred to TASK-NNNN, or rejected, each with who and why) and add any re-run as a new verification row. Recommend accept or reject; the status becomes `accepted` only on Jason's word, quoted in the record. Log a `review` event with `"task"`.

After substantial changes, record an observable agent event and update relevant durable documentation. Respect director approval boundaries and never invent evidence.
