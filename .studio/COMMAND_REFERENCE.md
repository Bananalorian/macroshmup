# AI Studio Commands

Claude project-local skills (available when installed in `.claude/skills/`; actual discoverability depends on the Claude Code version):

- `/studio` — Reception, greeting or skip, team, activity menu.
- `/studio-resume` — Load state, inspect Git and last handoff, present brief.
- `/studio-adopt` — Read-only existing-project audit and proposed backlog.
- `/studio-status` — Verify and summarize current state and blockers.
- `/studio-team` — List roles and agent capabilities.
- `/studio-brainstorm` — Guided ideation; record proposals, not automatically approved requirements.
- `/studio-plan` — Decompose approved objectives into sprints and tasks.
- `/studio-task` — Select/inspect bounded task, obtain approval, route implementation.
- `/studio-handoff` — Checkpoint, verify Git, create handoff for fresh session.
- `/studio-audit` — Compare claimed state to code, Git, specs, tests.
- `/studio-review` — Compare implementation report with acceptance criteria.
- `/studio-decisions` — Propose/approve/supersede decisions with impact analysis.
- `/studio-help` — Show this reference.

Local utility commands from cloned toolkit root: `python studio.py --project /path/to/project install`, `inspect`, `status`, `log`, `handoff`, `dashboard`.

Commands are **not** installed globally. Start Claude Code from the project root; type `/` to find project-local skills. If a skill isn't available, ask Claude to follow the corresponding `.claude/skills/studio-*/SKILL.md` file directly.
