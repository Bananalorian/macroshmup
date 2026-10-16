# DEC-0002 — Enemy bullets in five hues, teal included

| Field | Value |
|---|---|
| State | superseded by [DEC-0003](DEC-0003-bullet-color-by-action.md) (v0.12.0) |
| Date | 2026-10-11, shipped in v0.6.4 (`1bf2ed5`) |
| Supersedes | [DEC-0001](DEC-0001-two-hues.md), for enemy bullets |
| Recorded | 2026-10-16, during onboarding ([TASK-0002](../production/tasks/TASK-0002-studio-onboarding.md)) |

**Question:** should every enemy bullet be red?

**Decision:** Jason: "I don't want every bullet red; should be some teal and other colors"
(`MEMORY.md`, v0.6.4). Bullets came in red, teal, violet, amber and blue, assigned by pattern
(aimed singles and needles red, rings teal or blue, spirals violet, fans and sprinkles amber),
always with a white-hot core and a black outline.

**Approval:** Jason, 2026-10-11 (quote above). Rule 4 in `CLAUDE.md` and `AGENTS.md` said so
until v0.12.0.

**Superseded:** v0.12.0 dropped teal and tied each hue to what the player has to do (DEC-0003).
The old rule 4 wording survives in `git show v0.12.0~1:CLAUDE.md`. `AGENTS.md` kept it by
mistake through v0.13.0, until TASK-0001 replaced both files with `PROJECT_RULES.md`.
