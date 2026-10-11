//! The bot: a dodger that plays through the same deterministic sim. Each frame it
//! runs the guns and bullets forward a few frames, then tries 9 directions (8 and
//! standing still) at full and focused speed and picks the safest path. It powers
//! the fairness checks: if the bot can't survive a pattern, nobody should have to.

use super::World;
use super::bullets::seg_dist;
use super::pattern::Hazards;
use super::player::Controls;
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
    /// Predicted bullets per future frame (reused, so the bot doesn't allocate per frame).
    ahead: Vec<Vec<Ghost>>,
    last: usize,
    /// Bomb when struck (the deathbomb) or when every path is about to be hit.
    pub use_bombs: bool,
}

const DIRS: [(f32, f32); 9] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (-1.0, 1.0), (-1.0, 0.0), (-1.0, -1.0), (0.0, -1.0), (1.0, -1.0)];

impl Bot {
    pub fn new() -> Bot {
        Bot { ahead: (0..BOT_LOOKAHEAD).map(|_| Vec::with_capacity(512)).collect(), last: 0, use_bombs: true }
    }

    pub fn decide(&mut self, w: &World) -> Controls {
        // struck: bomb inside the window, every time it can
        if self.use_bombs && w.can_bomb() && matches!(w.life, super::Life::Struck { .. }) {
            return Controls { bomb: true, ..Default::default() };
        }
        let p = &w.player;
        let reach = BOT_LOOKAHEAD as f32 * w.tuning.speed + 40.0;
        // run the hazards forward with the player standing still
        let mut h: Hazards = w.hazards.clone();
        let mut m = [(0.0, 0.0, false); 8];
        let n = w.mounts(&mut m);
        for frame in self.ahead.iter_mut() {
            h.step((p.x, p.y), &m[..n]);
            frame.clear();
            let b = &h.bullets;
            for i in 0..b.len() {
                if (b.x[i] - p.x).abs() < reach && (b.y[i] - p.y).abs() < reach && b.age[i] >= BLOOM_FRAMES {
                    frame.push(Ghost { x: b.x[i], y: b.y[i], dx: b.dx[i], dy: b.dy[i], h: b.kind[i].half_len(), r: b.kind[i].hit_r() });
                }
            }
        }

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
                let mut score = -(nx - BOT_HOME.0).hypot(ny - BOT_HOME.1) * BOT_HOME_PULL;
                let (mut x, mut y) = (p.x, p.y);
                let mut first_hit = usize::MAX;
                for (k, frame) in self.ahead.iter().enumerate() {
                    x = (x + vx).clamp(EDGE_X, FIELD_W as f32 - EDGE_X);
                    y = (y + vy).clamp(EDGE_TOP, FIELD_H as f32 - EDGE_BOTTOM);
                    let weight = 1.0 / (k + 1) as f32;
                    for g in frame {
                        let clear = seg_dist(x, y, g.x, g.y, g.dx, g.dy, g.h) - g.r - HITBOX_R;
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
        // no way out within a couple of frames: bomb
        let bomb = self.use_bombs && w.can_bomb() && best_hit <= 1;
        Controls { mv: DIRS[di], fire: true, focus: si == 1, bomb, ..Default::default() }
    }
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
        w.hazards.bullets.push(crate::sim::bullets::Spawn { x, y, vx: 0.0, vy: 0.0, kind: crate::sim::bullets::Kind::Orb, behavior: crate::sim::bullets::Behavior::Straight });
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
