//! The Slam: the Pilgrim closes into the sword and dashes into a target, hits it hard
//! and shatters the bullets around it; the game freezes a moment on each strike.
//! Then a reticle picks the next target the way you're steering, and the slam button
//! chains on to it, 3 links in all. Each link spends a segment of the rush gauge, and
//! kills fill it.
//!
//! The ship can't be hurt from the dash's start to a little after the chain's end,
//! and it bounces back off what it hits, so it's never left inside an enemy.

use super::geom::Rect;
use super::{Controls, Event, Target, World};
use crate::tune::*;

/// Where a chain is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Slam {
    /// Dashing from `from` into `target` (`to` is where it strikes), `t` frames in;
    /// link `link` of the chain.
    Dash { from: (f32, f32), to: (f32, f32), target: Target, t: u32, link: u32 },
    /// Struck: `t` frames into aiming the next link (never straight back into `last`
    /// while there's anything else).
    Aim { t: u32, last: Target, link: u32 },
}

/// The point on `r` nearest `(x, y)`.
fn nearest(r: &Rect, x: f32, y: f32) -> (f32, f32) {
    (x.clamp(r.x0, r.x1), y.clamp(r.y0, r.y1))
}

impl World {
    /// Aiming a chain's next link.
    #[cfg(test)]
    pub fn aiming(&self) -> bool {
        matches!(self.slam, Some(Slam::Aim { .. }))
    }

    /// What a slam from `(x, y)` would strike: something in reach and worth it (never
    /// popcorn), the nearest along `dir` if one is held (else just the nearest), not
    /// `not` unless it's all there is.
    pub fn slam_target(&self, (x, y): (f32, f32), dir: (f32, f32), not: Option<Target>) -> Option<(Target, Rect)> {
        let held = dir.0.hypot(dir.1) > 0.3;
        let (ux, uy) = if held { (dir.0 / dir.0.hypot(dir.1), dir.1 / dir.0.hypot(dir.1)) } else { (0.0, 0.0) };
        let score = |r: &Rect| {
            let (px, py) = nearest(r, x, y);
            let (dx, dy) = (px - x, py - y);
            let d = dx.hypot(dy);
            if d > SLAM_REACH {
                return None;
            }
            // along the stick it's near; off it, far; behind it, out
            let along = if held && d > 1.0 { (dx * ux + dy * uy) / d } else { 1.0 };
            (along > -0.2).then(|| d * (2.0 - along))
        };
        let worth = |t: &Target| !matches!(*t, Target::Foe(i, _) if self.foes.list[i].kind.fodder() || self.foes.list[i].kind.scenery());
        let pick = |skip: Option<Target>| {
            self.boxes
                .iter()
                .filter(|(t, _)| Some(*t) != skip && worth(t))
                .filter_map(|&(t, r)| score(&r).map(|s| (s, t, r)))
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, t, r)| (t, r))
        };
        pick(not).or_else(|| not.and_then(|_| pick(None)))
    }

    /// The slam this tick. Returns whether it's driving the ship (dashing).
    pub(super) fn step_slam(&mut self, c: &Controls) -> bool {
        let me = (self.player.x, self.player.y);
        match self.slam {
            None => {
                self.reticle = None;
                if c.slam && self.alive() && self.rush >= 1.0 {
                    if let Some((t, r)) = self.slam_target(me, c.mv, None) {
                        self.start_dash(t, &r, 1);
                        return true;
                    }
                }
                false
            }
            Some(Slam::Dash { from, to, target, t, link }) => {
                let t = t + 1;
                // it follows the target if it moves
                let to = self.boxes.iter().find(|b| b.0 == target).map_or(to, |b| nearest(&b.1, from.0, from.1));
                let k = t as f32 / SLAM_DASH as f32;
                let k = 1.0 - (1.0 - k) * (1.0 - k);
                self.player.place(from.0 + (to.0 - from.0) * k, from.1 + (to.1 - from.1) * k);
                if t < SLAM_DASH {
                    self.slam = Some(Slam::Dash { from, to, target, t, link });
                } else {
                    self.strike(target, from, to, link);
                }
                true
            }
            Some(Slam::Aim { t, last, link }) => {
                let t = t + 1;
                self.reticle = self.slam_target(me, c.mv, Some(last));
                match self.reticle {
                    Some((next, r)) if c.slam && self.rush >= 1.0 && link < SLAM_LINKS => {
                        self.start_dash(next, &r, link + 1);
                        return true;
                    }
                    _ if t >= SLAM_AIM || c.slam => self.end_chain(),
                    _ => self.slam = Some(Slam::Aim { t, last, link }),
                }
                false
            }
        }
    }

    fn start_dash(&mut self, target: Target, r: &Rect, link: u32) {
        let from = (self.player.x, self.player.y);
        self.rush -= 1.0;
        self.reticle = None;
        self.slam = Some(Slam::Dash { from, to: nearest(r, from.0, from.1), target, t: 0, link });
        self.invuln = self.invuln.max(SLAM_DASH + SLAM_AIM + SLAM_INVULN);
        self.invuln_blink = false;
        self.events.push(Event::SlamDash { x: from.0, y: from.1, link });
    }

    /// The sword strikes `target` at `to`: it hurts it hard, shatters the bullets round
    /// it, and bounces back the way it came.
    fn strike(&mut self, target: Target, from: (f32, f32), to: (f32, f32), link: u32) {
        let alive = self.boxes.iter().any(|b| b.0 == target);
        self.by = super::By::Slam;
        if alive {
            self.hurt(target, if matches!(target, Target::Boss(_)) { SLAM_BOSS_DMG } else { SLAM_DMG });
        }
        self.stylish(super::style::Move::Slam, STYLE_SLAM.0 + STYLE_SLAM.1 * (link - 1) as f32);
        let mut out = std::mem::take(&mut self.cancelled);
        let before = out.len();
        self.hazards.cancel_within(to.0, to.1, SLAM_CANCEL_R, &mut out);
        let shards = out.len() - before;
        self.cancel_shards |= shards > 0;
        self.cancelled = out;
        self.award_shards(shards);
        let (dx, dy) = (to.0 - from.0, to.1 - from.1);
        let d = dx.hypot(dy).max(1.0);
        let x = (to.0 - dx / d * SLAM_BOUNCE).clamp(8.0, FIELD_W as f32 - 8.0);
        let y = (to.1 - dy / d * SLAM_BOUNCE).clamp(8.0, FIELD_H as f32 - 8.0);
        self.player.place(x, y);
        self.events.push(Event::Slam { x: to.0, y: to.1, link });
        if link < SLAM_LINKS {
            self.slam = Some(Slam::Aim { t: 0, last: target, link });
        } else {
            self.end_chain();
        }
    }

    fn end_chain(&mut self) {
        self.slam = None;
        self.reticle = None;
        self.invuln = self.invuln.max(SLAM_INVULN);
    }

    /// Fill the rush gauge by `n` segments (it holds `SLAM_LINKS`).
    pub(super) fn fill_rush(&mut self, n: f32) {
        self.rush = (self.rush + n).min(SLAM_LINKS as f32);
    }
}

#[cfg(test)]
mod tests {
    use super::super::bullets::{Behavior, Kind, Spawn};
    use super::super::dummy::{Dummy, DummyKind};
    use super::*;
    use crate::sim::player::Tuning;

    fn press(mv: (f32, f32)) -> Controls {
        Controls { slam: true, mv, ..Default::default() }
    }

    fn world(targets: &[(DummyKind, f32, f32)]) -> World {
        let mut w = World::new(1, Tuning::default());
        w.dummies = targets.iter().map(|&(k, x, y)| Dummy::new(k, x, y)).collect();
        w.hazards.bullets.clear();
        w.tick(Controls::default());
        w
    }

    /// The slam dashes into the nearest target in reach, invulnerable, and hits hard
    /// enough to break a small one; it spends a segment of the gauge.
    #[test]
    fn it_dashes_into_the_nearest_target_and_breaks_it() {
        let mut w = world(&[(DummyKind::Still, START_X, START_Y - 120.0), (DummyKind::Still, START_X + 100.0, START_Y - 140.0)]);
        w.rush = 1.0;
        w.tick(press((0.0, 0.0)));
        assert!(matches!(w.slam, Some(Slam::Dash { target: Target::Dummy(0), .. })), "{:?}", w.slam);
        assert_eq!(w.rush, 0.0);
        let mut broke = false;
        for _ in 0..SLAM_DASH + 1 {
            assert!(!w.vulnerable(), "invulnerable as it dashes");
            broke |= w.events.iter().any(|e| matches!(e, Event::Break { .. }));
            w.events.clear();
            w.tick(Controls::default());
        }
        broke |= w.events.iter().any(|e| matches!(e, Event::Break { .. }));
        assert!(broke, "it breaks");
        assert!(w.player.y > START_Y - 120.0 + 4.0, "it bounced back off it: {}", w.player.y);
    }

    /// Popcorn is never a slam's target: it dashes past to something worth it, or
    /// not at all.
    #[test]
    fn it_never_slams_popcorn() {
        use crate::sim::enemy::{Enemy, EnemyKind, Path};
        let mut w = world(&[]);
        let hover = |x: f32, y: f32| Path::Hover { from: (x, y), at: (x, y), enter: 1, stay: 9999, sway: 0.0 };
        w.foes.list.push(Enemy::new(1, EnemyKind::Drone, 0, hover(START_X, START_Y - 60.0), 0.0));
        w.tick(Controls::default());
        w.tick(Controls::default());
        w.rush = 3.0;
        assert_eq!(w.slam_target((w.player.x, w.player.y), (0.0, 0.0), None), None, "a drone alone: nothing to slam");
        w.foes.list.push(Enemy::new(2, EnemyKind::Gunship, 0, hover(START_X + 40.0, START_Y - 140.0), 0.0));
        w.tick(Controls::default());
        w.tick(Controls::default());
        let t = w.slam_target((w.player.x, w.player.y), (0.0, 0.0), None).map(|t| t.0);
        assert_eq!(t, Some(Target::Foe(1, 0)), "past the nearer drone to the gunship");
    }

    /// No gauge, no slam; nothing in reach, no slam.
    #[test]
    fn it_needs_the_gauge_and_something_in_reach() {
        let mut w = world(&[(DummyKind::Still, START_X, START_Y - 120.0)]);
        w.rush = 0.9;
        w.tick(press((0.0, 0.0)));
        assert_eq!(w.slam, None);
        let mut w = world(&[(DummyKind::Still, START_X, START_Y - SLAM_REACH - 40.0)]);
        w.rush = 3.0;
        w.tick(press((0.0, 0.0)));
        assert_eq!(w.slam, None);
        assert_eq!(w.rush, 3.0);
    }

    /// After a strike the reticle picks the next target the way you steer; the slam
    /// chains on to it, 3 links at most, then the chain ends.
    #[test]
    fn it_chains_to_the_target_you_steer_toward() {
        let left = (DummyKind::Big, START_X - 90.0, START_Y - 160.0);
        let right = (DummyKind::Big, START_X + 90.0, START_Y - 160.0);
        let mut w = world(&[(DummyKind::Big, START_X, START_Y - 100.0), left, right]);
        w.rush = 3.0;
        w.tick(press((0.0, -1.0)));
        for _ in 0..SLAM_DASH {
            w.tick(Controls::default());
        }
        assert!(w.aiming(), "{:?}", w.slam);
        w.tick(Controls { mv: (1.0, 0.0), ..Default::default() });
        assert_eq!(w.reticle.map(|r| r.0), Some(Target::Dummy(2)), "steering right picks the right one");
        w.tick(Controls { mv: (-1.0, 0.0), ..Default::default() });
        assert_eq!(w.reticle.map(|r| r.0), Some(Target::Dummy(1)), "steering left, the left one");
        w.tick(press((-1.0, 0.0)));
        assert!(matches!(w.slam, Some(Slam::Dash { target: Target::Dummy(1), link: 2, .. })));
        for _ in 0..SLAM_DASH {
            w.tick(Controls::default());
        }
        w.tick(press((1.0, 0.0)));
        assert!(matches!(w.slam, Some(Slam::Dash { link: 3, .. })), "{:?}", w.slam);
        for _ in 0..SLAM_DASH {
            w.tick(Controls::default());
        }
        assert_eq!(w.slam, None, "3 links and the chain is done");
        assert!(w.events.iter().filter(|e| matches!(e, Event::Slam { .. })).count() >= 1);
        // left to itself, the aim times out
        let mut w = world(&[(DummyKind::Big, START_X, START_Y - 100.0), left]);
        w.rush = 3.0;
        w.tick(press((0.0, 0.0)));
        for _ in 0..SLAM_DASH + SLAM_AIM + 1 {
            w.tick(Controls::default());
        }
        assert_eq!(w.slam, None);
        assert_eq!(w.rush, 2.0, "one link spent");
    }

    /// A strike shatters the bullets round it into shards, and big targets only lose
    /// a chunk.
    #[test]
    fn a_strike_shatters_bullets_and_dents_big_targets() {
        let mut w = world(&[(DummyKind::Big, START_X, START_Y - 100.0)]);
        w.rush = 1.0;
        let rect = w.dummies[0].rect();
        let at = (START_X, rect.y1);
        w.tick(press((0.0, 0.0)));
        for _ in 0..SLAM_DASH - 1 {
            w.tick(Controls::default());
        }
        for x in [at.0 + 10.0, at.0 + 100.0] {
            w.hazards.bullets.push(Spawn { x, y: at.1 + 6.0, vx: 0.0, vy: 0.0, kind: Kind::Orb, behavior: Behavior::Straight, owner: 0, hue: Default::default() });
        }
        w.tick(Controls::default());
        assert!(w.events.iter().any(|e| matches!(e, Event::Slam { .. })));
        assert_eq!(w.hazards.bullets.len(), 1, "the near bullet shattered, the far one didn't");
        assert!(w.cancel_shards);
        let d = &w.dummies[0];
        assert!(d.hp < d.max_hp && d.hp > 0.0, "a chunk: {} of {}", d.hp, d.max_hp);
    }

    /// The bot slams too (so PG_SIM covers it): into the heavies, chaining on.
    #[test]
    fn the_bot_slams_and_chains() {
        let mut w = World::stage_at(3, Tuning::default(), 115.0);
        let mut bot = crate::sim::bot::Bot::new();
        let (mut slams, mut chained) = (0, false);
        for _ in 0..60 * 30 {
            let c = bot.decide(&w);
            w.tick(c);
            for e in &w.events {
                if let Event::Slam { link, .. } = e {
                    slams += 1;
                    chained |= *link >= 2;
                }
            }
            w.events.clear();
        }
        assert!(slams >= 3 && chained, "{slams} strikes, chained: {chained}");
        assert!(w.deaths == 0);
    }

    /// Kills fill the gauge, big ones more, up to its 3 segments.
    #[test]
    fn kills_fill_the_rush_gauge() {
        let mut w = world(&[]);
        w.rush = 0.0;
        w.fill_rush(RUSH_FILL[3]);
        assert!(w.rush > 0.0 && w.rush < 1.0);
        let mut by_hp = crate::sim::enemy::EnemyKind::ALL;
        by_hp.sort_by(|a, b| a.stats().0.total_cmp(&b.stats().0));
        assert!(by_hp.windows(2).all(|p| RUSH_FILL[p[1] as usize] >= RUSH_FILL[p[0] as usize]), "bigger kills fill more");
        w.fill_rush(10.0);
        assert_eq!(w.rush, SLAM_LINKS as f32);
    }
}
