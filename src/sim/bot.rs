//! The bot: a dodger that plays through the same deterministic sim. Each frame it
//! runs the enemies, guns and bullets forward a few frames, then tries 9 directions
//! (8 and standing still) at full and focused speed and picks the safest path,
//! drifting under whatever it should be shooting. It powers the fairness checks: if
//! the bot can't survive a pattern, nobody should have to.

use super::boss::{Act, Boss, Laser};
use super::bullets::seg_dist;
use super::enemy::{EnemyKind, Foes};
use super::geom::Rect;
use super::pattern::Hazards;
use super::player::Controls;
use super::stage1::N;
use super::{Event, Life, World};
use crate::tune::*;

/// One bullet as predicted for a future frame: core center, direction, half-length, radius.
#[derive(Clone, Copy)]
struct Ghost {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    h: f32,
    r: f32,
}

pub struct Bot {
    /// Predicted bullets and enemy bodies per future frame (reused, so the bot doesn't
    /// allocate per frame).
    ahead: Vec<Vec<Ghost>>,
    bodies: Vec<Vec<Rect>>,
    events: Vec<Event>,
    last: usize,
    /// Bomb when struck (the deathbomb) or when every path is about to be hit.
    pub use_bombs: bool,
}

const DIRS: [(f32, f32); 9] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (-1.0, 1.0), (-1.0, 0.0), (-1.0, -1.0), (0.0, -1.0), (1.0, -1.0)];

impl Bot {
    pub fn new() -> Bot {
        Bot {
            ahead: (0..BOT_LOOKAHEAD).map(|_| Vec::with_capacity(512)).collect(),
            bodies: (0..BOT_LOOKAHEAD).map(|_| Vec::with_capacity(32)).collect(),
            events: Vec::with_capacity(16),
            last: 0,
            use_bombs: true,
        }
    }

    /// What to line up under: the boss (its weak points first), else the high-HP
    /// enemy, else the nearest one above.
    fn target(w: &World) -> Option<f32> {
        let p = &w.player;
        if let Some(b) = &w.boss {
            if let Some(part) = b.targets().next() {
                let r = b.rect(part);
                return Some((r.x0 + r.x1) * 0.5);
            }
        }
        w.foes
            .list
            .iter()
            .filter(|e| e.hittable() && e.y < p.y - 40.0)
            .max_by(|a, b| (a.kind.heavy(), a.y).partial_cmp(&(b.kind.heavy(), b.y)).unwrap())
            .map(|e| e.x)
    }

    /// A laser as a long capsule: from its jaws out past the screen.
    fn beam(l: &Laser) -> Ghost {
        let (s, c) = l.angle().to_radians().sin_cos();
        let half = 320.0;
        Ghost { x: l.x + c * half, y: l.y + s * half, dx: c, dy: s, h: half, r: JAWS_LASER.3 * 0.3 }
    }

    /// How bad `(x, y)` is under a laser: where it has still to sweep is death. While
    /// it's only a warning line, the way out is back across that line (running ahead
    /// of a beam ends at the wall), so the deeper into the sweep, the worse.
    fn sweep_risk(l: &Laser, x: f32, y: f32) -> f32 {
        let a = (y - l.y).atan2(x - l.x).to_degrees();
        let (now, to) = (l.angle(), l.to);
        let ahead = if to > now { a - now } else { now - a };
        let span = (to - now).abs();
        if !(-8.0..=span + 3.0).contains(&ahead) {
            return 0.0;
        }
        if l.firing() { 3000.0 } else { 3000.0 + 80.0 * (ahead + 8.0) }
    }

    /// The Slam: with a link in the gauge, into anything worth it (not popcorn) in
    /// reach; aiming, on to whatever the reticle's on.
    fn slam(w: &World) -> bool {
        if w.rush < 1.0 {
            return false;
        }
        match w.slam {
            Some(super::Slam::Aim { .. }) => w.reticle.is_some(),
            Some(_) => false,
            None => w.slam_target((w.player.x, w.player.y), (0.0, 0.0), None).is_some_and(|(t, _)| match t {
                super::Target::Foe(i, _) => !w.foes.list[i].kind.fodder(),
                _ => true,
            }),
        }
    }

    pub fn decide(&mut self, w: &World) -> Controls {
        // struck: bomb inside the window, every time it can
        if self.use_bombs && w.can_bomb() && matches!(w.life, super::Life::Struck { .. }) {
            return Controls { bomb: true, ..Default::default() };
        }
        let p = &w.player;
        let reach = BOT_LOOKAHEAD as f32 * w.tuning.speed + 40.0;
        // run the enemies, the boss and their bullets forward with the player standing still
        let mut h: Hazards = w.hazards.clone();
        let mut foes: Foes = w.foes.clone();
        let mut boss: Option<Boss> = w.boss.clone();
        let laser = w.boss.as_ref().and_then(|b| b.laser);
        let fire_ok = w.stage.as_ref().is_none_or(|s| s.fire_ok());
        let mut m = [(0.0, 0.0, false); 8];
        let n = w.mounts(&mut m);
        for (frame, bodies) in self.ahead.iter_mut().zip(self.bodies.iter_mut()) {
            h.step((p.x, p.y), &m[..n]);
            foes.step(&mut h, (p.x, p.y), fire_ok, &mut self.events);
            if let Some(b) = &mut boss {
                b.step(&mut h, (p.x, p.y), &mut self.events);
            }
            self.events.clear();
            frame.clear();
            if let Some(l) = boss.as_ref().and_then(|b| b.laser).filter(|l| l.firing()) {
                frame.push(Self::beam(&l));
            }
            let b = &h.bullets;
            for i in 0..b.len() {
                if (b.x[i] - p.x).abs() < reach && (b.y[i] - p.y).abs() < reach && b.age[i] >= BLOOM_FRAMES {
                    frame.push(Ghost { x: b.x[i], y: b.y[i], dx: b.dx[i], dy: b.dy[i], h: b.kind[i].half_len(), r: b.kind[i].hit_r() });
                }
            }
            bodies.clear();
            bodies.extend(foes.list.iter().filter(|e| e.alive()).map(|e| e.body()).filter(|r| r.dist(p.x, p.y) < reach));
            if let Some(b) = boss.as_ref().filter(|b| matches!(b.act, Act::Fight | Act::Break)) {
                bodies.extend(b.crash_boxes().into_iter().filter(|r| r.dist(p.x, p.y) < reach));
            }
        }
        // drift under the target, at the usual height
        let target = Self::target(w);
        let home = (target.unwrap_or(BOT_HOME.0), BOT_HOME.1);
        let aim = if target.is_some() { BOT_AIM } else { 0.0 };

        let speeds = [w.tuning.speed, w.tuning.speed * w.tuning.focus_ratio];
        let mut best = (f32::MIN, 0usize);
        // the earliest frame each path gets hit (for the best one: is a hit unavoidable?)
        let mut best_hit = usize::MAX;
        for (si, &speed) in speeds.iter().enumerate() {
            for (di, &(dx, dy)) in DIRS.iter().enumerate() {
                if di == 0 && si == 1 {
                    continue; // standing still is standing still
                }
                let len = dx.hypot(dy).max(1.0);
                let (vx, vy) = (dx / len * speed, dy / len * speed);
                // a gentle pull home, judged on the next step so it doesn't overshoot
                let (nx, ny) = (p.x + vx, p.y + vy);
                let mut score = -(nx - home.0).hypot(ny - home.1) * BOT_HOME_PULL - (nx - home.0).abs() * aim;
                let (mut x, mut y) = (p.x, p.y);
                let mut first_hit = usize::MAX;
                for (k, (frame, bodies)) in self.ahead.iter().zip(&self.bodies).enumerate() {
                    x = (x + vx).clamp(EDGE_X, FIELD_W as f32 - EDGE_X);
                    y = (y + vy).clamp(EDGE_TOP, FIELD_H as f32 - EDGE_BOTTOM);
                    let weight = 1.0 / (k + 1) as f32;
                    let crash = bodies.iter().map(|r| r.dist(x, y) - HITBOX_R);
                    for clear in frame.iter().map(|g| seg_dist(x, y, g.x, g.y, g.dx, g.dy, g.h) - g.r - HITBOX_R).chain(crash) {
                        if clear < BOT_MARGIN {
                            score -= 10_000.0 * weight;
                            first_hit = first_hit.min(k);
                        } else if clear < BOT_DANGER {
                            let d = BOT_DANGER - clear;
                            score -= d * d * weight;
                        }
                    }
                }
                // and away from the walls, judged on where the path ends
                let wall = (x - 30.0).min(FIELD_W as f32 - 30.0 - x).min(FIELD_H as f32 - 30.0 - y).min(0.0);
                score -= wall * wall * 0.3;
                // out of the way of a laser's sweep
                if let Some(l) = &laser {
                    score -= Self::sweep_risk(l, x, y);
                }
                let id = si * DIRS.len() + di;
                if id == self.last {
                    score += BOT_STICKY; // don't dither
                }
                if si == 1 {
                    score -= BOT_STICKY; // full speed unless slow is safer
                }
                if score > best.0 {
                    best = (score, id);
                    best_hit = first_hit;
                }
            }
        }
        self.last = best.1;
        let (si, di) = (best.1 / DIRS.len(), best.1 % DIRS.len());
        // no way out within a couple of frames: bomb (not while nothing can hurt it,
        // mid-slam or just respawned)
        let bomb = self.use_bombs && w.can_bomb() && best_hit <= 1 && w.vulnerable();
        Controls { mv: DIRS[di], fire: true, focus: si == 1, bomb, slam: Self::slam(w), ..Default::default() }
    }
}

/// How a bot run of Stage 1 went.
#[derive(Clone, Debug, Default)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))] // PG_SIM and the tests are native
pub struct StageRun {
    /// Reached the end with a ship left.
    pub cleared: bool,
    pub deaths: u32,
    pub bombs: u32,
    /// Deaths in each section, and when each section began (frames).
    pub section_deaths: [u32; N],
    pub began: [Option<u32>; N],
    pub killed: u32,
    pub spawned: u32,
    pub grazes: u32,
    pub peak: usize,
    /// Bullets on screen, summed over every frame (for the average).
    pub bullet_frames: u64,
    pub frames: u64,
    /// Frames from the derelict's arrival to its death (none if it left).
    pub hull_kill: Option<u32>,
    /// The boss: how long each phase took (frames, none if it timed out or wasn't
    /// reached), deaths in each, and whether it was destroyed.
    pub phase_time: [Option<u32>; 4],
    pub phase_deaths: [u32; 4],
    pub boss_killed: bool,
    /// The final score, the extends it earned, each phase's grade and the stage's rank.
    pub score: u64,
    pub extends: u8,
    pub grades: [Option<super::score::Grade>; 4],
    pub rank: Option<super::score::Grade>,
    /// Frames at each style rank (D to SSS), and the best reached.
    pub style_frames: [u64; 7],
    pub style_peak: u8,
}

/// The bot plays Stage 1 from the top with 3 lives, with or without bombs, shooting
/// or not (a pacifist run shows what the stage throws at you if you ignore it).
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub fn stage_trial(seed: u64, bombs: bool, shoot: bool) -> StageRun {
    let mut w = World::stage(seed, Default::default());
    let mut bot = Bot::new();
    bot.use_bombs = bombs;
    let mut run = StageRun::default();
    let mut hull_at = None;
    let mut phase_at = 0u32;
    let mut deaths_at = 0;
    while w.frame < stage_limit() && w.life != Life::GameOver && !w.stage.as_ref().is_some_and(|s| s.over) {
        let mut c = bot.decide(&w);
        c.fire &= shoot;
        w.tick(c);
        run.bullet_frames += w.hazards.bullets.len() as u64;
        for e in &w.events {
            match *e {
                Event::Bomb { .. } => run.bombs += 1,
                // (the run's frame, not the stage clock, which jumps ahead when it dies)
                Event::Kill { kind: EnemyKind::Hull, .. } => run.hull_kill = hull_at.map(|t0| w.frame as u32 - t0),
                Event::BossPhase(_) => {
                    phase_at = w.frame as u32;
                    deaths_at = w.deaths;
                }
                Event::BossBreak { .. } | Event::BossDown { .. } => {
                    if let Some(b) = &w.boss {
                        let p = b.phase as usize - 1;
                        if b.outcomes[p] == Some(super::boss::Outcome::Broken) {
                            run.phase_time[p] = Some(w.frame as u32 - phase_at);
                        }
                        run.phase_deaths[p] = w.deaths - deaths_at;
                    }
                    run.boss_killed |= matches!(e, Event::BossDown { .. });
                }
                Event::BossFall => {
                    run.phase_deaths[3] = w.deaths - deaths_at;
                }
                _ => {}
            }
        }
        if hull_at.is_none() && w.foes.list.iter().any(|e| e.kind == EnemyKind::Hull) {
            hull_at = Some(w.frame as u32);
        }
        w.events.clear();
        run.peak = run.peak.max(w.hazards.bullets.len());
    }
    let s = w.stage.as_ref().unwrap();
    run.frames = w.frame;
    run.cleared = s.over && w.life != Life::GameOver;
    run.deaths = w.deaths;
    run.section_deaths = s.deaths;
    run.began = s.began;
    run.killed = s.killed;
    run.spawned = s.spawned;
    run.grazes = w.grazes;
    run.style_frames = w.style.frames;
    run.style_peak = w.style.peak;
    run.score = w.score.points;
    run.extends = w.score.extends;
    run.grades = w.score.grades;
    run.rank = w.score.tally.map(|t| t.rank);
    run
}

/// How long a whole stage can possibly run (frames): to the boss, every phase to its
/// timeout with the breaks between, its death, and a little more.
pub fn stage_limit() -> u64 {
    let phases: u32 = JAWS_PHASES.iter().map(|p| p.1 * TICK_HZ).sum();
    (super::stage1::frames(JAWS_AT) + JAWS_INTRO[3] + phases + 3 * JAWS_BREAK + JAWS_DEATH.max(JAWS_FALL) + 600) as u64
}

/// How a bot run of one boss phase went.
#[derive(Clone, Debug, Default)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct PhaseRun {
    pub deaths: u32,
    /// Frames to break it (none if it timed out).
    pub kill: Option<u32>,
    pub grazes: u32,
    pub peak: usize,
}

/// The bot fights boss phase `phase` (1-4) from its start, with endless lives and
/// with or without bombs, until it ends.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub fn phase_trial(seed: u64, phase: u8, bombs: bool) -> PhaseRun {
    let mut w = World::boss_at(seed, Default::default(), phase);
    w.lives = u8::MAX;
    let mut bot = Bot::new();
    bot.use_bombs = bombs;
    let mut run = PhaseRun::default();
    let limit = JAWS_PHASES[phase as usize - 1].1 * TICK_HZ + 10;
    for f in 0..limit {
        let c = bot.decide(&w);
        w.tick(c);
        run.peak = run.peak.max(w.hazards.bullets.len());
        let ended = w.events.iter().any(|e| matches!(e, Event::BossBreak { .. } | Event::BossDown { .. } | Event::BossFall));
        w.events.clear();
        if ended {
            let b = w.boss.as_ref().unwrap();
            if b.outcomes[phase as usize - 1] == Some(super::boss::Outcome::Broken) {
                run.kill = Some(f + 1);
            }
            break;
        }
    }
    run.deaths = w.deaths;
    run.grazes = w.grazes;
    run
}

/// Run the bot on preset `preset` for `frames` frames, with or without bombs (and
/// endless lives); returns (deaths, grazes, peak bullets).
#[cfg_attr(target_arch = "wasm32", allow(dead_code))] // PG_SIM and the tests are native
pub fn trial(preset: usize, seed: u64, frames: u32, bombs: bool) -> (u32, u32, usize) {
    let mut w = World::new(seed, Default::default());
    w.set_pattern(preset);
    w.lives = u8::MAX;
    let mut bot = Bot::new();
    bot.use_bombs = bombs;
    let mut peak = 0;
    for _ in 0..frames {
        let c = bot.decide(&w);
        w.tick(c);
        w.events.clear();
        peak = peak.max(w.hazards.bullets.len());
    }
    (w.deaths, w.grazes, peak)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bot_steps_out_of_a_bullets_way() {
        // one big orb coming straight down at the ship
        let mut w = World::new(1, Default::default());
        w.dummies.clear();
        let (px, py) = (w.player.x, w.player.y);
        w.hazards.bullets.push(crate::sim::bullets::Spawn {
            x: px,
            y: py - 60.0,
            vx: 0.0,
            vy: 3.0,
            kind: crate::sim::bullets::Kind::Large,
            behavior: crate::sim::bullets::Behavior::Straight,
            owner: 0,
            hue: Default::default(),
        });
        w.hazards.bullets.age[0] = 10;
        let mut bot = Bot::new();
        let mut aside: f32 = 0.0;
        for _ in 0..60 {
            let c = bot.decide(&w);
            w.tick(c);
            aside = aside.max((w.player.x - px).abs());
        }
        assert_eq!(w.deaths, 0);
        assert!(aside > 6.0, "it moved aside ({aside} px)");
    }

    #[test]
    fn the_bot_deathbombs() {
        let mut w = World::new(1, Default::default());
        w.dummies.clear();
        let (x, y) = (w.player.x, w.player.y);
        w.hazards.bullets.push(crate::sim::bullets::Spawn { x, y, vx: 0.0, vy: 0.0, kind: crate::sim::bullets::Kind::Orb, behavior: crate::sim::bullets::Behavior::Straight, owner: 0, hue: Default::default() });
        w.hazards.bullets.age[0] = 30;
        w.tick(Controls::default());
        assert!(matches!(w.life, crate::sim::Life::Struck { .. }));
        let mut bot = Bot::new();
        let c = bot.decide(&w);
        assert!(c.bomb, "it bombs inside the window");
        w.tick(c);
        assert!(w.alive() && w.deaths == 0);
    }

    #[test]
    fn standing_still_with_nothing_around_stays_home() {
        let mut w = World::new(1, Default::default());
        let mut bot = Bot::new();
        for _ in 0..200 {
            let c = bot.decide(&w);
            w.tick(c);
        }
        assert!((w.player.x - BOT_HOME.0).abs() < 8.0 && (w.player.y - BOT_HOME.1).abs() < 8.0, "{} {}", w.player.x, w.player.y);
    }
}
