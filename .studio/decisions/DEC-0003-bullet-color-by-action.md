# DEC-0003 — Enemy bullets colored by what to do; never teal

| Field | Value |
|---|---|
| State | approved (current) |
| Date | 2026-10-13, shipped in v0.12.0 (`1356b53`) |
| Supersedes | [DEC-0002](DEC-0002-five-bullet-hues.md); [DEC-0001](DEC-0001-two-hues.md) for enemy bullets |
| Recorded | 2026-10-16, during onboarding ([TASK-0002](../production/tasks/TASK-0002-studio-onboarding.md)) |

**Question:** Jason: "i need bullets with clear distinctions" (`MEMORY.md`, v0.12.0).

**Options:** three schemes were offered; he picked "Color by what to do" (`MEMORY.md`, v0.12.0;
`SPRINTS.md` Sprint 11, "Jason's pick of three schemes").

**Decision:** red is aimed at you; blue is a fixed shape (rings, curtains, fans thrown straight);
violet spins or bends; amber is a random scatter. Never teal: teal is the Pilgrim's. Every bullet
keeps its white-hot core and black outline.

**Approval:** Jason, 2026-10-13 (the pick above).

**Where it lives:** `PROJECT_RULES.md` non-negotiable 4; `SPRINTS.md` §2 Color rules.

**Code and tests:** `src/sim/bullets.rs` `Hue` has four values (Red, Violet, Amber, Blue);
`src/sim/pattern.rs` `Gun::hue()` derives the hue from the gun's shape and behavior; the test
`sim::pattern::tests::every_bullets_color_says_what_to_do` checks the boss's and heavies' guns and
that there are four hues with no teal (passed on 2026-10-16, TASK-0002 V3). The pattern sandbox's
stress filler picks random hues; it's a test scene, not gameplay.

**Conflict resolved (2026-10-16):** `SPRINTS.md` §1 still said "only two hues exist, red and
teal" and §3 "in the red family". Both are now marked *Superseded* with links here; the old
wording is kept.
