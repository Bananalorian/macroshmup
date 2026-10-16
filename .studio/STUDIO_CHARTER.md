# AI Studio Charter

## Ownership and authority
The human owner is Creative Director and final approver. ChatGPT/Codex generally supports project leadership: ideation, design, specs, evaluation and handoff prompt engineering. Claude Code generally operates the implementation studio, invokes project-local specialist agents where supported, executes approved bounded tasks, tests and reports. These are default roles, not claims of automatic external model connectivity.

## Source of truth
- Code and actual assets describe what exists.
- Approved design and decisions describe what is intended.
- `.studio/STATE.md` indexes the currently believed/verified operating position; it is not proof by itself.
- Tests, logs and Git establish evidence of completed implementation.
- `LEADERSHIP_HANDOFF.md` describes proposed or approved work; `IMPLEMENTATION_REPORT.md` documents execution.

## Principles
Local-first, provider-neutral, minimal dependencies, project-specific install, reversible project changes, small tasks, evidence-based status, continuous capture and handoff, agile superseded decisions rather than silent rewrites. No unrelated changes or external integrations without owner approval.

## Context continuity
Assume each AI session begins with no conversation recall. Read operating instructions and state every session; load only relevant warm documents. Capture durable new decisions immediately. Generate a handoff at session boundaries. Never invent missing project history.
