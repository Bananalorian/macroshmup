//! Enemy bullets: a struct-of-arrays pool with swap-remove, sized once and never
//! reallocated during play. Each bullet has a kind (its look and hitbox) and a
//! behavior (how it flies).

use crate::tune::{BULLET_CAP, BULLET_HIT, BULLET_SIZE};

/// A pulled bullet this close to what pulls it is swallowed.
const SWALLOW: f32 = 4.0;

/// Draw order goes by this order too: big and slow at the bottom, small and fast on top.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Small,
    Orb,
    Large,
    Rice,
    Needle,
    /// A four-point star, twinkling and turning as it drifts (v0.8.6): slow layers.
    Star,
    /// A crystal shard, pointed along its flight like the boss's diamond eye.
    Diamond,
}

impl Kind {
    pub const ALL: [Kind; 7] = [Kind::Small, Kind::Orb, Kind::Large, Kind::Rice, Kind::Needle, Kind::Star, Kind::Diamond];
    /// Bottom to top.
    pub const DRAW_ORDER: [Kind; 7] = [Kind::Large, Kind::Star, Kind::Orb, Kind::Diamond, Kind::Rice, Kind::Small, Kind::Needle];

    /// Sprite radius (across) in px.
    pub fn radius(self) -> f32 {
        BULLET_SIZE[self as usize].0
    }
    /// Half-length of the core along the flight direction (0 for round bullets).
    pub fn half_len(self) -> f32 {
        BULLET_SIZE[self as usize].1
    }
    /// The hitbox: a capsule this wide around the core line.
    pub fn hit_r(self) -> f32 {
        self.radius() * BULLET_HIT
    }
    /// How far the sprite reaches in any direction (for culling and the no-walls check).
    pub fn extent(self) -> f32 {
        self.radius() + self.half_len()
    }
}

/// A bullet's color: the hue of its rim. Every bullet keeps a white-hot core and a
/// black outline whatever its hue, so it reads by value first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Hue {
    #[default]
    Red,
    Teal,
    Violet,
    Amber,
    Blue,
}

impl Hue {
    pub const ALL: [Hue; 5] = [Hue::Red, Hue::Teal, Hue::Violet, Hue::Amber, Hue::Blue];
}

/// How a bullet flies after it's fired.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Behavior {
    Straight,
    /// Speeds up (or slows) by `accel` per frame until it reaches `max`.
    Accel { accel: f32, max: f32 },
    /// Slows to a stop over `slow` frames, waits `pause`, then flies at the player at `speed`.
    SlowAim { slow: u16, pause: u16, speed: f32 },
    /// Turns `turn` degrees per frame for its first `frames` frames.
    Curve { turn: f32, frames: u16 },
    /// At `at` frames old, bursts into a ring of `n` bullets of `kind` at `speed`.
    Split { at: u16, n: u8, kind: Kind, speed: f32 },
    /// Pulled toward `(x, y)` by `pull` px/frame², never faster than `max`; it's
    /// swallowed when it gets there.
    Pull { x: f32, y: f32, pull: f32, max: f32 },
}

/// A bullet waiting to be added (from a split), so the pool isn't changed mid-loop.
#[derive(Clone, Copy, Debug)]
pub struct Spawn {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub kind: Kind,
    pub behavior: Behavior,
    /// The enemy that fired it (0: nobody), so its bullets can go when it dies.
    pub owner: u32,
    pub hue: Hue,
}

#[derive(Clone)]
pub struct Bullets {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub vx: Vec<f32>,
    pub vy: Vec<f32>,
    /// The last direction of flight (unit), kept while a bullet is stopped.
    pub dx: Vec<f32>,
    pub dy: Vec<f32>,
    pub kind: Vec<Kind>,
    pub behavior: Vec<Behavior>,
    pub age: Vec<u16>,
    pub grazed: Vec<bool>,
    pub owner: Vec<u32>,
    pub hue: Vec<Hue>,
}

impl Bullets {
    pub fn new() -> Bullets {
        let n = BULLET_CAP;
        Bullets {
            x: Vec::with_capacity(n),
            y: Vec::with_capacity(n),
            vx: Vec::with_capacity(n),
            vy: Vec::with_capacity(n),
            dx: Vec::with_capacity(n),
            dy: Vec::with_capacity(n),
            kind: Vec::with_capacity(n),
            behavior: Vec::with_capacity(n),
            age: Vec::with_capacity(n),
            grazed: Vec::with_capacity(n),
            owner: Vec::with_capacity(n),
            hue: Vec::with_capacity(n),
        }
    }

    pub fn len(&self) -> usize {
        self.x.len()
    }

    pub fn clear(&mut self) {
        self.x.clear();
        self.y.clear();
        self.vx.clear();
        self.vy.clear();
        self.dx.clear();
        self.dy.clear();
        self.kind.clear();
        self.behavior.clear();
        self.age.clear();
        self.grazed.clear();
        self.owner.clear();
        self.hue.clear();
    }

    /// Add a bullet. Past the pool's capacity it's dropped rather than reallocating.
    pub fn push(&mut self, s: Spawn) {
        if self.len() >= BULLET_CAP {
            return;
        }
        let v = s.vx.hypot(s.vy);
        let (dx, dy) = if v > 1e-6 { (s.vx / v, s.vy / v) } else { (0.0, 1.0) };
        self.x.push(s.x);
        self.y.push(s.y);
        self.vx.push(s.vx);
        self.vy.push(s.vy);
        self.dx.push(dx);
        self.dy.push(dy);
        self.kind.push(s.kind);
        self.behavior.push(s.behavior);
        self.age.push(0);
        self.grazed.push(false);
        self.owner.push(s.owner);
        self.hue.push(s.hue);
    }

    pub fn swap_remove(&mut self, i: usize) {
        self.x.swap_remove(i);
        self.y.swap_remove(i);
        self.vx.swap_remove(i);
        self.vy.swap_remove(i);
        self.dx.swap_remove(i);
        self.dy.swap_remove(i);
        self.kind.swap_remove(i);
        self.behavior.swap_remove(i);
        self.age.swap_remove(i);
        self.grazed.swap_remove(i);
        self.owner.swap_remove(i);
        self.hue.swap_remove(i);
    }

    /// Advance bullet `i` one frame. Returns true if it's gone (it split, its children
    /// going into `out`, or what pulls it swallowed it).
    pub fn step(&mut self, i: usize, player: (f32, f32), out: &mut Vec<Spawn>) -> bool {
        let age = self.age[i];
        let (mut vx, mut vy) = (self.vx[i], self.vy[i]);
        match self.behavior[i] {
            Behavior::Straight => {}
            Behavior::Accel { accel, max } => {
                let v = vx.hypot(vy);
                let nv = if accel >= 0.0 { (v + accel).min(max) } else { (v + accel).max(max) };
                vx = self.dx[i] * nv;
                vy = self.dy[i] * nv;
            }
            Behavior::SlowAim { slow, pause, speed } => {
                if age < slow {
                    // ease to a stop
                    let k = (slow - age - 1) as f32 / (slow - age) as f32;
                    vx *= k;
                    vy *= k;
                } else if age == slow + pause {
                    let (ax, ay) = (player.0 - self.x[i], player.1 - self.y[i]);
                    let d = ax.hypot(ay).max(1e-3);
                    vx = ax / d * speed;
                    vy = ay / d * speed;
                }
            }
            Behavior::Curve { turn, frames } => {
                if age < frames {
                    let (s, c) = turn.to_radians().sin_cos();
                    (vx, vy) = (vx * c - vy * s, vx * s + vy * c);
                }
            }
            Behavior::Split { at, n, kind, speed } => {
                if age >= at {
                    let base = self.dy[i].atan2(self.dx[i]);
                    for k in 0..n {
                        let a = base + k as f32 * std::f32::consts::TAU / n as f32;
                        out.push(Spawn { x: self.x[i], y: self.y[i], vx: a.cos() * speed, vy: a.sin() * speed, kind, behavior: Behavior::Straight, owner: self.owner[i], hue: self.hue[i] });
                    }
                    return true;
                }
            }
            Behavior::Pull { x, y, pull, max } => {
                let (ax, ay) = (x - self.x[i], y - self.y[i]);
                let d = ax.hypot(ay).max(1.0);
                if d < SWALLOW {
                    return true;
                }
                vx += ax / d * pull;
                vy += ay / d * pull;
                let v = vx.hypot(vy);
                if v > max {
                    vx *= max / v;
                    vy *= max / v;
                }
            }
        }
        self.vx[i] = vx;
        self.vy[i] = vy;
        let v = vx.hypot(vy);
        if v > 1e-4 {
            self.dx[i] = vx / v;
            self.dy[i] = vy / v;
        }
        self.x[i] += vx;
        self.y[i] += vy;
        self.age[i] = age.saturating_add(1);
        false
    }

    /// Distance from `(px, py)` to bullet `i`'s core line.
    pub fn core_dist(&self, i: usize, px: f32, py: f32) -> f32 {
        seg_dist(px, py, self.x[i], self.y[i], self.dx[i], self.dy[i], self.kind[i].half_len())
    }
}

/// Distance from a point to a segment centered on `(cx, cy)` running `h` px
/// each way along the unit direction `(dx, dy)`.
pub fn seg_dist(px: f32, py: f32, cx: f32, cy: f32, dx: f32, dy: f32, h: f32) -> f32 {
    let (rx, ry) = (px - cx, py - cy);
    let t = (rx * dx + ry * dy).clamp(-h, h);
    (rx - dx * t).hypot(ry - dy * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(kind: Kind, behavior: Behavior, vx: f32, vy: f32) -> Bullets {
        let mut b = Bullets::new();
        b.push(Spawn { x: 100.0, y: 100.0, vx, vy, kind, behavior, owner: 0, hue: Default::default() });
        b
    }

    fn run(b: &mut Bullets, frames: usize, player: (f32, f32)) -> Vec<Spawn> {
        let mut out = vec![];
        for _ in 0..frames {
            if b.len() > 0 && b.step(0, player, &mut out) {
                b.swap_remove(0);
            }
        }
        out
    }

    #[test]
    fn the_pool_never_grows_past_its_capacity() {
        let mut b = Bullets::new();
        let cap = b.x.capacity();
        for _ in 0..BULLET_CAP + 100 {
            b.push(Spawn { x: 0.0, y: 0.0, vx: 0.0, vy: 1.0, kind: Kind::Small, behavior: Behavior::Straight, owner: 0, hue: Default::default() });
        }
        assert_eq!(b.len(), BULLET_CAP);
        assert_eq!(b.x.capacity(), cap, "no reallocation");
        b.swap_remove(0);
        assert_eq!(b.len(), BULLET_CAP - 1);
        assert!(b.kind.len() == b.len() && b.age.len() == b.len() && b.grazed.len() == b.len() && b.owner.len() == b.len() && b.hue.len() == b.len());
    }

    #[test]
    fn accel_speeds_up_to_its_cap() {
        let mut b = one(Kind::Orb, Behavior::Accel { accel: 0.1, max: 3.0 }, 0.0, 0.5);
        run(&mut b, 100, (0.0, 0.0));
        assert!((b.vy[0] - 3.0).abs() < 1e-4 && b.vx[0] == 0.0);
    }

    #[test]
    fn slow_aim_stops_waits_then_flies_at_the_player() {
        let mut b = one(Kind::Orb, Behavior::SlowAim { slow: 20, pause: 10, speed: 3.0 }, 3.0, 0.0);
        run(&mut b, 20, (100.0, 300.0));
        assert!(b.vx[0].hypot(b.vy[0]) < 1e-4, "stopped");
        let still = (b.x[0], b.y[0]);
        run(&mut b, 10, (100.0, 300.0));
        assert_eq!((b.x[0], b.y[0]), still, "waited");
        run(&mut b, 1, (still.0, 300.0));
        assert!(b.vx[0].abs() < 1e-4 && (b.vy[0] - 3.0).abs() < 1e-4, "aimed straight at the player");
    }

    #[test]
    fn curves_turn_for_a_while_then_fly_straight() {
        let mut b = one(Kind::Rice, Behavior::Curve { turn: 1.0, frames: 90 }, 2.0, 0.0);
        run(&mut b, 200, (0.0, 0.0));
        let ang = b.vy[0].atan2(b.vx[0]).to_degrees();
        assert!((ang - 90.0).abs() < 0.1, "turned 90 degrees: {ang}");
        assert!((b.vx[0].hypot(b.vy[0]) - 2.0).abs() < 1e-3, "kept its speed");
    }

    #[test]
    fn splits_burst_into_a_ring() {
        let mut b = one(Kind::Large, Behavior::Split { at: 30, n: 8, kind: Kind::Small, speed: 2.0 }, 0.0, 1.0);
        let kids = run(&mut b, 31, (0.0, 0.0));
        assert_eq!(b.len(), 0);
        assert_eq!(kids.len(), 8);
        assert!(kids.iter().all(|k| k.kind == Kind::Small && (k.vx.hypot(k.vy) - 2.0).abs() < 1e-4 && (k.y - 130.0).abs() < 1e-3));
    }

    #[test]
    fn pull_drags_toward_its_point() {
        let mut b = one(Kind::Small, Behavior::Pull { x: 100.0, y: 0.0, pull: 0.1, max: 2.0 }, 1.0, 0.0);
        run(&mut b, 40, (0.0, 0.0));
        assert!(b.vy[0] < -1.0, "heading up toward the point");
        assert!(b.vx[0].hypot(b.vy[0]) <= 2.0 + 1e-4);
        // one falling straight in is swallowed when it gets there (one going round orbits)
        let mut b = one(Kind::Small, Behavior::Pull { x: 100.0, y: 0.0, pull: 0.1, max: 2.0 }, 0.0, -0.5);
        run(&mut b, 120, (0.0, 0.0));
        assert_eq!(b.len(), 0);
    }

    #[test]
    fn needles_hit_along_their_length() {
        let b = one(Kind::Needle, Behavior::Straight, 0.0, 3.0);
        // the core runs up and down from the center
        assert!(b.core_dist(0, 100.0, 104.0) < 0.01);
        assert!((b.core_dist(0, 100.0, 110.0) - (10.0 - Kind::Needle.half_len())).abs() < 1e-4);
        assert!((b.core_dist(0, 103.0, 100.0) - 3.0).abs() < 1e-4);
        let o = one(Kind::Orb, Behavior::Straight, 0.0, 3.0);
        assert!((o.core_dist(0, 100.0, 104.0) - 4.0).abs() < 1e-4, "orbs are round");
    }

    #[test]
    fn hitboxes_lean_in_the_players_favor() {
        for k in Kind::ALL {
            assert!(k.hit_r() < k.radius() * 0.6);
        }
    }
}
