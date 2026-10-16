---
name: studio-decisions
description: Manage decisions.
user-invocable: true
---
# /studio-decisions

Read `.studio/STUDIO_CHARTER.md`, `.studio/OPERATING_MANUAL.md`, and `.studio/STATE.md`.

Document proposed, approved and superseded decisions, rationale, versions, impact on backlog/spec/tests; never erase old rationale.

One file per decision, `.studio/decisions/DEC-NNNN-slug.md`, listed in that folder's README register. To supersede, write a new record and mark the old one `superseded by DEC-NNNN` without changing its text; where `SPRINTS.md` or `PROJECT_RULES.md` carried the old rule, keep its wording there, marked *Superseded*, with a link. Name the task the decision came from.

After substantial changes, record an observable agent event and update relevant durable documentation. Respect director approval boundaries and never invent evidence.
