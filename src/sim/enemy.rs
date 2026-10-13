//! Enemies: popcorn drones, gunships, turret buoys, heavies and the derelict hull.
//! Each one follows a path (a spline, or fly in, hover and leave, or a slow drift),
//! carries guns from the pattern library and takes hits on a big hurtbox. Kill it
//! fast and the screen stays clean; leave it and it keeps firing until it goes.
//!
//! | Enemy    | Optimal kill                         | Average kill        | Slow kill (ignored)                     |
//! |----------|--------------------------------------|---------------------|-----------------------------------------|
//! | Drone    | as it enters, before its one shot    | mid-path            | it fires once and flies off             |
//! | Courier  | in the gap before it reaches you     | as it passes        | its needle, then it's gone              |
//! | Usher    | as it slows to stop                  | after one pair      | two pairs of aimed shots, then it leaves|
//! | Gunship  | during its first fan                 | after a few fans    | it leaves after about 4 s               |
//! | Buoy     | before its rings fill the lane       | halfway down        | it drifts out, silent low on the screen |
//! | Heavy    | fast: its bullets cancel into shards | a few volleys in    | it leaves after about 8 s               |
//! | Derelict | break the turrets, then burn the core| the turrets only    | it leaves at 25 s                       |

use super::Event;
use super::geom::Rect;
use super::pattern::{Emitter, Gun, Hazards};
use super::rng::Rng;
use crate::tune::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    Drone,
    /// A fast three-spired dart.
    Courier,
    /// A horseshoe that stops, fires a pair from its prongs, and leaves.
    Usher,
    Gunship,
    Buoy,
    Heavy,
    /// The mid-boss: the derelict hull.
    Hull,
    /// A bone lance: it stops, takes aim at you, and rushes.
    Lancer,
    /// A little fish that swims across in a school.
    Remora,
    /// Scenery to shoot: drifting debris, a satellite, a monolith.
    Debris,
    Satellite,
    Monolith,
}

impl EnemyKind {
    #[cfg(test)]
    pub const ALL: [EnemyKind; 12] = [
        EnemyKind::Drone,
        EnemyKind::Courier,
        EnemyKind::Usher,
        EnemyKind::Gunship,
        EnemyKind::Buoy,
        EnemyKind::Heavy,
        EnemyKind::Hull,
        EnemyKind::Lancer,
        EnemyKind::Remora,
        EnemyKind::Debris,
        EnemyKind::Satellite,
        EnemyKind::Monolith,
    ];

    /// (hit points, hurtbox width, height)
    pub fn stats(self) -> (f32, f32, f32) {
        match self {
            EnemyKind::Drone => DRONE,
            EnemyKind::Courier => COURIER,
            EnemyKind::Usher => USHER,
            EnemyKind::Gunship => GUNSHIP,
            EnemyKind::Buoy => BUOY,
            EnemyKind::Heavy => HEAVY,
            EnemyKind::Hull => HULL,
            EnemyKind::Lancer => LANCER,
            EnemyKind::Remora => REMORA,
            EnemyKind::Debris => DEBRIS,
            EnemyKind::Satellite => SATELLITE,
            EnemyKind::Monolith => MONOLITH,
        }
    }

    /// Popcorn: low HP, small, many.
    pub fn fodder(self) -> bool {
        matches!(self, EnemyKind::Drone | EnemyKind::Courier | EnemyKind::Usher | EnemyKind::Lancer | EnemyKind::Remora)
    }

    /// Scenery: it never fires, it's harmless to touch, and it doesn't count toward
    /// the stage's kills.
    pub fn scenery(self) -> bool {
        matches!(self, EnemyKind::Debris | EnemyKind::Satellite | EnemyKind::Monolith)
    }

    /// High-HP enemies arrive one at a time, and their bullets cancel when they die.
    pub fn heavy(self) -> bool {
        matches!(self, EnemyKind::Heavy | EnemyKind::Hull)
    }

    /// Its guns, by variant.
    fn arms(self, variant: u8) -> &'static [Arm] {
        match (self, variant) {
            (EnemyKind::Drone, 0) => &[],
            (EnemyKind::Drone, _) => DRONE_GUNS,
            (EnemyKind::Courier, 0) => &[],
            (EnemyKind::Courier, _) => COURIER_GUNS,
            (EnemyKind::Usher, _) => USHER_GUNS,
            (EnemyKind::Gunship, _) => GUNSHIP_GUNS,
            (EnemyKind::Buoy, 0) => BUOY_RING,
            (EnemyKind::Buoy, _) => BUOY_SPIRAL,
            (EnemyKind::Heavy, _) => HEAVY_GUNS,
            (EnemyKind::Hull, _) => HULL_GUNS,
            (EnemyKind::Remora, 1) => REMORA_GUNS,
            (EnemyKind::Lancer | EnemyKind::Remora | EnemyKind::Debris | EnemyKind::Satellite | EnemyKind::Monolith, _) => &[],
        }
    }
}

/// One of an enemy's guns: the part carrying it (0 is the body), the phase it fires
/// in (0 always, 1 the first, 2 the second) and the gun.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arm {
    pub part: u8,
    pub phase: u8,
    pub gun: Gun,
}

pub const fn arm(part: u8, phase: u8, gun: Gun) -> Arm {
    Arm { part, phase, gun }
}

/// A Catmull-Rom spline through up to 6 points, walked at a steady speed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spline {
    pts: [(f32, f32); 6],
    n: usize,
    /// Distance along the curve to the start of each segment (and to the end).
    at: [f32; 6],
    speed: f32,
}

impl Spline {
    pub fn new(pts: &[(f32, f32)], speed: f32) -> Spline {
        assert!((2..=6).contains(&pts.len()), "a spline takes 2 to 6 points");
        let mut s = Spline { pts: [(0.0, 0.0); 6], n: pts.len(), at: [0.0; 6], speed };
        s.pts[..pts.len()].copy_from_slice(pts);
        for i in 0..s.n - 1 {
            let mut len = 0.0;
            let mut prev = s.seg(i, 0.0);
            for k in 1..=16 {
                let p = s.seg(i, k as f32 / 16.0);
                len += (p.0 - prev.0).hypot(p.1 - prev.1);
                prev = p;
            }
            s.at[i + 1] = s.at[i] + len;
        }
        s
    }

    /// Point `i`, with phantom points mirrored past both ends.
    fn point(&self, i: isize) -> (f32, f32) {
        let last = self.n - 1;
        let mirror = |a: (f32, f32), b: (f32, f32)| (2.0 * a.0 - b.0, 2.0 * a.1 - b.1);
        if i < 0 {
            mirror(self.pts[0], self.pts[1])
        } else if i as usize > last {
            mirror(self.pts[last], self.pts[last - 1])
        } else {
            self.pts[i as usize]
        }
    }

    /// Segment `i` (point i to point i + 1) at `u` in 0..1.
    fn seg(&self, i: usize, u: f32) -> (f32, f32) {
        let i = i as isize;
        let (a, b, c, d) = (self.point(i - 1), self.point(i), self.point(i + 1), self.point(i + 2));
        let (u2, u3) = (u * u, u * u * u);
        let f = |a: f32, b: f32, c: f32, d: f32| 0.5 * (2.0 * b + (c - a) * u + (2.0 * a - 5.0 * b + 4.0 * c - d) * u2 + (3.0 * b - a - 3.0 * c + d) * u3);
        (f(a.0, b.0, c.0, d.0), f(a.1, b.1, c.1, d.1))
    }

    pub fn length(&self) -> f32 {
        self.at[self.n - 1]
    }

    /// Where it is `t` frames along; past the end it carries straight on.
    pub fn pos(&self, t: u32) -> (f32, f32) {
        let s = t as f32 * self.speed;
        let total = self.length();
        if s >= total {
            let end = self.pts[self.n - 1];
            let before = self.seg(self.n - 2, 0.95);
            let (dx, dy) = (end.0 - before.0, end.1 - before.1);
            let d = dx.hypot(dy).max(1e-4);
            return (end.0 + dx / d * (s - total), end.1 + dy / d * (s - total));
        }
        let i = (0..self.n - 1).rev().find(|&i| self.at[i] <= s).unwrap_or(0);
        self.seg(i, (s - self.at[i]) / (self.at[i + 1] - self.at[i]).max(1e-4))
    }
}

/// How an enemy moves. Every path is a pure function of the enemy's age, so the
/// bot (and a replay) can tell exactly where it will be.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Path {
    Spline(Spline),
    /// Fly in from `from` to `at` over `enter` frames, hover there swaying `sway` px
    /// for `stay` frames, then leave upward, faster and faster.
    Hover { from: (f32, f32), at: (f32, f32), enter: u32, stay: u32, sway: f32 },
    /// Sink from `(x, y)` at `vy` px a frame, swaying `sway` px side to side.
    Drift { x: f32, y: f32, vy: f32, sway: f32 },
    /// Fly in from `from` to `at` over `enter` frames, take aim for `wait`, then rush
    /// along `dir` (set as it locks on) at `speed` px a frame.
    Rush { from: (f32, f32), at: (f32, f32), enter: u32, wait: u32, speed: f32, dir: (f32, f32) },
}

impl Path {
    pub fn pos(&self, t: u32) -> (f32, f32) {
        let wave = |t: f32| (t * std::f32::consts::TAU / SWAY_FRAMES as f32).sin();
        match *self {
            Path::Spline(s) => s.pos(t),
            Path::Hover { from, at, enter, stay, sway } => {
                if t < enter {
                    let k = 1.0 - (1.0 - t as f32 / enter as f32).powi(2);
                    return (from.0 + (at.0 - from.0) * k, from.1 + (at.1 - from.1) * k);
                }
                let h = (t - enter) as f32;
                let x = at.0 + sway * wave(h);
                let gone = t.saturating_sub(enter + stay) as f32;
                (x, at.1 - LEAVE_ACCEL * gone * gone * 0.5)
            }
            Path::Drift { x, y, vy, sway } => (x + sway * wave(t as f32), y + vy * t as f32),
            Path::Rush { from, at, enter, wait, speed, dir } => {
                if t < enter {
                    let k = 1.0 - (1.0 - t as f32 / enter as f32).powi(2);
                    (from.0 + (at.0 - from.0) * k, from.1 + (at.1 - from.1) * k)
                } else if t < enter + wait {
                    at
                } else {
                    let d = speed * (t - enter - wait) as f32;
                    (at.0 + dir.0 * d, at.1 + dir.1 * d)
                }
            }
        }
    }

    /// A lancer taking aim: frames into its aim and how long it lasts.
    pub fn aiming(&self, t: u32) -> Option<(u32, u32)> {
        match *self {
            Path::Rush { enter, wait, .. } if t >= enter && t < enter + wait => Some((t - enter, wait)),
            _ => None,
        }
    }

    /// Hovering in place: hovering enemies only fire then (not flying in or leaving).
    pub fn settled(&self, t: u32) -> bool {
        match *self {
            Path::Hover { enter, stay, .. } => t >= enter && t < enter + stay,
            Path::Rush { enter, .. } => t >= enter,
            _ => true,
        }
    }

    /// Still flying in (the derelict can't be hurt while it rises out of the disk).
    pub fn entering(&self, t: u32) -> bool {
        matches!(*self, Path::Hover { enter, .. } if t < enter)
    }
}

/// A piece of an enemy with its own hurtbox and health: the body (part 0), or one
/// of the derelict's weak-point turrets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub dx: f32,
    pub dy: f32,
    pub w: f32,
    pub h: f32,
    pub hp: f32,
    pub max_hp: f32,
    /// Hit flash frames left, and frames until it may flash again.
    pub flash: u8,
    flash_cd: u8,
    /// The current bomb has already hurt it.
    pub bombed: bool,
}

impl Part {
    fn new(dx: f32, dy: f32, (hp, w, h): (f32, f32, f32)) -> Part {
        Part { dx, dy, w, h, hp, max_hp: hp, flash: 0, flash_cd: 0, bombed: false }
    }

    pub fn alive(&self) -> bool {
        self.hp > 0.0
    }
}

/// What a hit did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hurt {
    /// Nothing (it was already broken).
    Nothing,
    Hit,
    /// A weak point broke off.
    Broke,
    /// The body is destroyed.
    Died,
}

#[derive(Clone, Debug)]
pub struct Gunner {
    pub part: u8,
    pub phase: u8,
    pub e: Emitter,
}

#[derive(Clone, Debug)]
pub struct Enemy {
    /// Unique in the run (never 0); its bullets carry it.
    pub id: u32,
    pub kind: EnemyKind,
    pub variant: u8,
    pub path: Path,
    /// Frames since it arrived.
    pub t: u32,
    pub x: f32,
    pub y: f32,
    /// How far it moved this frame.
    pub vx: f32,
    pub vy: f32,
    /// The body first, then any weak points.
    pub parts: Vec<Part>,
    pub guns: Vec<Gunner>,
    /// 1, or 2 once the derelict's hull has cracked.
    pub phase: u8,
    /// It may fire this frame (for the red charge-up telegraphs).
    pub armed: bool,
    /// It's been on the screen (so leaving it again means it's gone).
    pub seen: bool,
}

impl Enemy {
    /// A new enemy; `turn` sets where its rings and spirals start (seeded, so every run differs).
    pub fn new(id: u32, kind: EnemyKind, variant: u8, path: Path, turn: f32) -> Enemy {
        let mut parts = vec![Part::new(0.0, 0.0, kind.stats())];
        if kind == EnemyKind::Hull {
            parts.extend(HULL_TURRETS.iter().map(|&(dx, dy, w, h)| Part::new(dx, dy, (HULL_TURRET_HP, w, h))));
        }
        let guns = kind.arms(variant).iter().map(|a| Gunner { part: a.part, phase: a.phase, e: Emitter::new(a.gun, id, turn) }).collect();
        let (x, y) = path.pos(0);
        Enemy { id, kind, variant, path, t: 0, x, y, vx: 0.0, vy: 0.0, parts, guns, phase: 1, armed: false, seen: false }
    }

    pub fn alive(&self) -> bool {
        self.parts[0].alive()
    }

    /// Part `p`'s hurtbox.
    pub fn rect(&self, p: usize) -> Rect {
        let q = &self.parts[p];
        Rect::centered(self.x + q.dx, self.y + q.dy, q.w, q.h)
    }

    /// The body you crash into: a little inside the hurtbox.
    pub fn body(&self) -> Rect {
        let q = &self.parts[0];
        Rect::centered(self.x, self.y, q.w * BODY_SHRINK, q.h * BODY_SHRINK)
    }

    /// Shots can hurt it: below the top dead zone, and (the derelict) risen out of the disk.
    pub fn hittable(&self) -> bool {
        self.alive() && self.y >= DEAD_ZONE && !self.path.entering(self.t)
    }

    /// It may fire: on the screen, settled, above the bottom ceasefire zone (and buoys
    /// fall silent sooner).
    fn may_fire(&self, fire_ok: bool) -> bool {
        let floor = match self.kind {
            EnemyKind::Buoy => FIELD_H as f32 * BUOY_QUIET,
            _ => FIELD_H as f32 - CEASEFIRE,
        };
        fire_ok && self.alive() && self.path.settled(self.t) && !self.path.entering(self.t) && self.y >= DEAD_ZONE && self.y <= floor && self.x >= 0.0 && self.x <= FIELD_W as f32
    }

    /// Gone for good: it came on screen and has now left it.
    fn left(&self) -> bool {
        let m = 48.0;
        (self.seen && (self.y < -m || self.y > FIELD_H as f32 + m || self.x < -m || self.x > FIELD_W as f32 + m)) || self.t > ENEMY_MAX_AGE
    }

    /// Take a hit on part `p`.
    pub fn hurt(&mut self, p: usize, dmg: f32) -> Hurt {
        let q = &mut self.parts[p];
        if !q.alive() {
            return Hurt::Nothing;
        }
        q.hp -= dmg;
        if q.flash_cd == 0 {
            (q.flash, q.flash_cd) = ENEMY_FLASH;
        }
        if q.hp > 0.0 {
            return Hurt::Hit;
        }
        q.hp = 0.0;
        if p == 0 { Hurt::Died } else { Hurt::Broke }
    }

    /// The derelict's weak points left standing.
    pub fn turrets_left(&self) -> usize {
        self.parts[1..].iter().filter(|q| q.alive()).count()
    }

    fn step(&mut self, hz: &mut Hazards, player: (f32, f32), fire_ok: bool, events: &mut Vec<Event>) {
        self.t += 1;
        // a lancer locks on: it'll rush at where you are now
        if let Path::Rush { enter, wait, ref mut dir, .. } = self.path {
            if self.t == enter + wait {
                let (dx, dy) = (player.0 - self.x, player.1 - self.y);
                let d = dx.hypot(dy);
                *dir = if d < 1.0 { (0.0, 1.0) } else { (dx / d, dy / d) };
            }
        }
        let (x, y) = self.path.pos(self.t);
        (self.vx, self.vy) = (x - self.x, y - self.y);
        (self.x, self.y) = (x, y);
        self.seen |= y >= 0.0 && y <= FIELD_H as f32 && x >= 0.0 && x <= FIELD_W as f32;
        for q in &mut self.parts {
            q.flash = q.flash.saturating_sub(1);
            q.flash_cd = q.flash_cd.saturating_sub(1);
        }
        // the derelict cracks open when its weak points are gone, or after a while
        if self.kind == EnemyKind::Hull && self.phase == 1 && !self.path.entering(self.t) {
            if self.turrets_left() == 0 || self.t >= HULL_PHASE2 {
                self.phase = 2;
                events.push(Event::Crack { x: self.x, y: self.y });
            }
        }
        self.armed = self.may_fire(fire_ok);
        for g in &mut self.guns {
            let live = self.armed && self.parts[g.part as usize].alive() && (g.phase == 0 || g.phase == self.phase);
            hz.fire_gun(&mut g.e, player, (self.x, self.y, live));
        }
    }
}

/// Every enemy in play.
#[derive(Clone)]
pub struct Foes {
    pub list: Vec<Enemy>,
    next_id: u32,
    /// Gameplay randomness for the enemies (seeded from the run's seed).
    pub rng: Rng,
}

impl Default for Foes {
    fn default() -> Foes {
        Foes::new(0)
    }
}

impl Foes {
    pub fn new(seed: u64) -> Foes {
        Foes { list: Vec::with_capacity(64), next_id: 0, rng: Rng::new(seed ^ 0xF0E5) }
    }

    /// Bring one in. Returns its id.
    pub fn add(&mut self, kind: EnemyKind, variant: u8, path: Path) -> u32 {
        self.next_id += 1;
        let turn = self.rng.range(0.0, 360.0);
        self.list.push(Enemy::new(self.next_id, kind, variant, path, turn));
        self.next_id
    }

    /// One frame: the dead and departed go, the rest move and fire.
    pub fn step(&mut self, hz: &mut Hazards, player: (f32, f32), fire_ok: bool, events: &mut Vec<Event>) {
        self.list.retain(|e| e.alive() && !e.left());
        for e in &mut self.list {
            e.step(hz, player, fire_ok, events);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hover(stay: u32) -> Path {
        Path::Hover { from: (100.0, -20.0), at: (100.0, 80.0), enter: ENTER_FRAMES, stay, sway: 0.0 }
    }

    #[test]
    fn splines_pass_through_their_points_at_a_steady_speed() {
        let pts = [(100.0, -10.0), (100.0, 80.0), (160.0, 140.0), (230.0, 120.0)];
        let s = Spline::new(&pts, 2.0);
        assert_eq!(s.pos(0), pts[0]);
        // every point is passed on the way
        for p in &pts[1..] {
            let close = (0..400).map(|t| s.pos(t)).map(|q| (q.0 - p.0).hypot(q.1 - p.1)).fold(f32::MAX, f32::min);
            assert!(close < 1.5, "missed {p:?} by {close}");
        }
        // about 2 px every frame, and on past the end in a straight line
        for t in 0..300 {
            let (a, b) = (s.pos(t), s.pos(t + 1));
            let step = (b.0 - a.0).hypot(b.1 - a.1);
            assert!((1.4..2.6).contains(&step), "frame {t}: {step}");
        }
        let (a, b, c) = (s.pos(400), s.pos(401), s.pos(402));
        assert!(((b.0 - a.0) - (c.0 - b.0)).abs() < 1e-3 && ((b.1 - a.1) - (c.1 - b.1)).abs() < 1e-3);
    }

    #[test]
    fn hovering_enemies_fly_in_stay_then_leave_upward() {
        let p = hover(100);
        assert_eq!(p.pos(0), (100.0, -20.0));
        assert_eq!(p.pos(ENTER_FRAMES), (100.0, 80.0));
        assert!(!p.settled(ENTER_FRAMES - 1) && p.settled(ENTER_FRAMES) && p.settled(ENTER_FRAMES + 99));
        assert!(!p.settled(ENTER_FRAMES + 100), "leaving, so it stops firing");
        assert!(p.pos(ENTER_FRAMES + 200).1 < -100.0, "gone up the screen");
    }

    #[test]
    fn hits_flash_at_a_limited_rate_and_break_things() {
        let mut e = Enemy::new(1, EnemyKind::Hull, 0, hover(9999), 0.0);
        assert_eq!(e.parts.len(), 1 + HULL_TURRETS.len());
        let mut flashes = 0;
        for _ in 0..60 {
            let was = e.parts[0].flash;
            e.hurt(0, 1.0);
            if e.parts[0].flash > was {
                flashes += 1;
            }
            for q in &mut e.parts {
                q.flash = q.flash.saturating_sub(1);
                q.flash_cd = q.flash_cd.saturating_sub(1);
            }
        }
        assert_eq!(flashes, 60 / ENEMY_FLASH.1 as usize);
        assert_eq!(e.hurt(1, HULL_TURRET_HP), Hurt::Broke);
        assert_eq!(e.hurt(1, 5.0), Hurt::Nothing, "a broken turret takes no more");
        assert_eq!(e.turrets_left(), HULL_TURRETS.len() - 1);
        assert_eq!(e.hurt(0, HULL.0), Hurt::Died);
        assert!(!e.alive());
    }

    #[test]
    fn enemies_hold_fire_off_screen_while_moving_and_in_the_ceasefire_zone() {
        let mut hz = Hazards::new(0, 1);
        let mut ev = vec![];
        // a gunship: nothing while it flies in, fans while it hovers
        let mut f = Foes::default();
        f.add(EnemyKind::Gunship, 0, hover(GUNSHIP_STAY));
        for _ in 0..ENTER_FRAMES {
            f.step(&mut hz, (135.0, 300.0), true, &mut ev);
        }
        assert_eq!(hz.bullets.len(), 0, "not while flying in");
        for _ in 0..GUNSHIP_STAY {
            f.step(&mut hz, (135.0, 300.0), true, &mut ev);
        }
        assert!(hz.bullets.len() > 0, "fans while it hovers");
        assert!((0..hz.bullets.len()).all(|i| hz.bullets.owner[i] == f.list[0].id));
        // a buoy sinking through the screen goes quiet low down
        let mut hz = Hazards::new(0, 1);
        let mut f = Foes::default();
        f.add(EnemyKind::Buoy, 0, Path::Drift { x: 135.0, y: -14.0, vy: 1.0, sway: 0.0 });
        let mut lowest: f32 = 0.0;
        for _ in 0..400 {
            let n = hz.bullets.len();
            f.step(&mut hz, (135.0, 340.0), true, &mut ev);
            if hz.bullets.len() > n {
                lowest = lowest.max(f.list[0].y);
            }
            assert!(hz.bullets.len() == n || f.list[0].y >= DEAD_ZONE, "never from the dead zone");
        }
        assert!(lowest > 100.0 && lowest <= FIELD_H as f32 * BUOY_QUIET, "{lowest}");
        // the stage's opening seconds: no fire at all
        let mut hz = Hazards::new(0, 1);
        let mut f = Foes::default();
        f.add(EnemyKind::Gunship, 0, hover(GUNSHIP_STAY));
        for _ in 0..400 {
            f.step(&mut hz, (135.0, 300.0), false, &mut ev);
        }
        assert_eq!(hz.bullets.len(), 0);
    }

    #[test]
    fn the_derelict_cracks_when_its_turrets_are_gone() {
        let mut hz = Hazards::new(0, 1);
        let mut ev = vec![];
        let mut f = Foes::default();
        f.add(EnemyKind::Hull, 0, Path::Hover { from: (135.0, 90.0), at: (135.0, 80.0), enter: 30, stay: 9999, sway: 0.0 });
        for _ in 0..40 {
            f.step(&mut hz, (135.0, 300.0), true, &mut ev);
        }
        assert_eq!(f.list[0].phase, 1);
        for p in 1..f.list[0].parts.len() {
            f.list[0].hurt(p, 9999.0);
        }
        f.step(&mut hz, (135.0, 300.0), true, &mut ev);
        assert_eq!(f.list[0].phase, 2);
        assert!(ev.iter().any(|e| matches!(e, Event::Crack { .. })));
    }

    #[test]
    fn the_dead_and_departed_are_dropped() {
        let mut hz = Hazards::new(0, 1);
        let mut ev = vec![];
        let mut f = Foes::default();
        f.add(EnemyKind::Drone, 0, Path::Spline(Spline::new(&[(100.0, -10.0), (100.0, 400.0)], 3.0)));
        f.add(EnemyKind::Drone, 0, Path::Spline(Spline::new(&[(150.0, -10.0), (150.0, 400.0)], 3.0)));
        f.list[1].hurt(0, 99.0);
        f.step(&mut hz, (0.0, 0.0), true, &mut ev);
        assert_eq!(f.list.len(), 1);
        for _ in 0..200 {
            f.step(&mut hz, (0.0, 0.0), true, &mut ev);
        }
        assert!(f.list.is_empty(), "flew off the bottom");
    }
}
