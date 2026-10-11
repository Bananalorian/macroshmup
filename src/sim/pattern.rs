//! The pattern library: guns that fire frame-timed, seeded patterns from fixed
//! points or from a moving enemy. Patterns are aimed, static or random, and the
//! presets in `tune::SANDBOX` layer them together.

use super::bullets::{Behavior, Bullets, Kind, Spawn};
use super::rng::Rng;
use crate::tune::{BULLET_MARGIN, BULLET_MAX_AGE, FIELD_H, FIELD_W, SAFE_SPAWN, SANDBOX};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// `n` bullets across `spread` degrees, centered on the player (aimed) or straight down.
    Fan { n: u8, spread: f32, aimed: bool },
    /// `n` bullets evenly around a circle; each shot turns the ring by `turn` degrees.
    Ring { n: u8, turn: f32 },
    /// `arms` streams turning `spin` degrees a shot, reversing every `flip` shots (0: never).
    Spiral { arms: u8, spin: f32, flip: u16 },
    /// A burst of `burst` aimed bullets, one every `gap` frames, once per cycle.
    Stream { burst: u8, gap: u8 },
    /// `n` bullets at random within `spread` degrees of straight down, speed varied by ±`jitter`.
    Cone { n: u8, spread: f32, jitter: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mount {
    /// Fixed in place.
    At(f32, f32),
    /// Rides target dummy `i`, and falls silent while it's broken.
    Dummy(u8),
    /// Sweeps side to side along `y` between `x0` and `x1`, once per `period` frames.
    Sweep { y: f32, x0: f32, x1: f32, period: u32 },
    /// Rides an enemy, offset `(dx, dy)` from its center (the enemy is `mounts[0]`).
    Owner { dx: f32, dy: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gun {
    pub mount: Mount,
    pub shape: Shape,
    pub kind: Kind,
    pub speed: f32,
    /// Frames between shots (or between bursts for streams).
    pub every: u32,
    /// Frames before the first shot.
    pub delay: u32,
    pub behavior: Behavior,
}

pub struct Preset {
    pub name: &'static str,
    pub guns: &'static [Gun],
    /// Stress presets: keep this many bullets on screen (the player can't be hit).
    pub fill: u32,
}

/// A gun in play.
#[derive(Clone, Debug)]
pub struct Emitter {
    pub gun: Gun,
    pub t: u32,
    /// The ring or spiral's current angle (degrees) and spin direction.
    angle: f32,
    dir: f32,
    shots: u32,
    /// Frames since it last fired (drives the red muzzle-flash telegraph).
    pub since: u32,
    pub x: f32,
    pub y: f32,
    pub live: bool,
    /// The enemy carrying it (0: none); its bullets carry the same id.
    pub owner: u32,
}

/// Where the guns' mounts are this frame: (x, y, alive) for each target dummy.
/// For an enemy's guns it's the one enemy: (x, y, may fire).
pub type Mounts<'a> = &'a [(f32, f32, bool)];

/// Everything that can hurt the player: the guns and their bullets.
#[derive(Clone)]
pub struct Hazards {
    pub preset: usize,
    pub emitters: Vec<Emitter>,
    pub bullets: Bullets,
    pub rng: Rng,
    spawns: Vec<Spawn>,
}

fn deg(a: f32) -> (f32, f32) {
    let (s, c) = a.to_radians().sin_cos();
    (c, s)
}

impl Hazards {
    pub fn new(preset: usize, seed: u64) -> Hazards {
        let mut h = Hazards { preset: 0, emitters: Vec::new(), bullets: Bullets::new(), rng: Rng::new(seed ^ 0xB0_11E7), spawns: Vec::with_capacity(256) };
        h.load(preset);
        h
    }

    pub fn preset(&self) -> &'static Preset {
        &SANDBOX[self.preset]
    }

    /// Switch to preset `i` (wrapping), clearing the screen.
    pub fn load(&mut self, i: usize) {
        self.preset = i % SANDBOX.len();
        self.bullets.clear();
        self.emitters = SANDBOX[self.preset]
            .guns
            .iter()
            .map(|g| Emitter::new(*g, 0, self.rng.range(0.0, 360.0)))
            .collect();
    }

    /// Fire one frame of an enemy's gun from `origin` (x, y, may fire) into the pool.
    pub fn fire_gun(&mut self, e: &mut Emitter, player: (f32, f32), origin: (f32, f32, bool)) {
        e.fire_frame(player, &[origin], &mut self.rng, &mut self.spawns);
        for s in self.spawns.drain(..) {
            self.bullets.push(s);
        }
    }

    /// Cancel every bullet `owner` fired, noting where each was.
    pub fn cancel_owner(&mut self, owner: u32, out: &mut Vec<(f32, f32)>) {
        let b = &mut self.bullets;
        let mut i = 0;
        while i < b.len() {
            if b.owner[i] == owner {
                out.push((b.x[i], b.y[i]));
                b.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Cancel every bullet whose core is within `r` of `(x, y)`, noting where each was.
    pub fn cancel_within(&mut self, x: f32, y: f32, r: f32, out: &mut Vec<(f32, f32)>) {
        let b = &mut self.bullets;
        let mut i = 0;
        while i < b.len() {
            if (b.x[i] - x).hypot(b.y[i] - y) <= r {
                out.push((b.x[i], b.y[i]));
                b.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Queue a bullet unless it would appear too close to the player.
    fn fire(spawns: &mut Vec<Spawn>, s: Spawn, player: (f32, f32)) {
        if (s.x - player.0).hypot(s.y - player.1) >= SAFE_SPAWN {
            spawns.push(s);
        }
    }

    /// One frame: move the guns, fire, move every bullet, drop the ones that left.
    pub fn step(&mut self, player: (f32, f32), mounts: Mounts) {
        // bullets first, so new ones start where they're fired
        let mut i = 0;
        while i < self.bullets.len() {
            let split = self.bullets.step(i, player, &mut self.spawns);
            let (x, y) = (self.bullets.x[i], self.bullets.y[i]);
            let m = BULLET_MARGIN + self.bullets.kind[i].extent();
            let out = x < -m || x > FIELD_W as f32 + m || y < -m || y > FIELD_H as f32 + m;
            if split || out || self.bullets.age[i] >= BULLET_MAX_AGE {
                self.bullets.swap_remove(i);
            } else {
                i += 1;
            }
        }
        // split children obey the no-spawn zone too
        let kids = std::mem::take(&mut self.spawns);
        for s in &kids {
            Self::fire(&mut self.spawns, *s, player);
        }

        for e in &mut self.emitters {
            e.fire_frame(player, mounts, &mut self.rng, &mut self.spawns);
        }
        for s in self.spawns.drain(..) {
            self.bullets.push(s);
        }

        let fill = SANDBOX[self.preset].fill as usize;
        while self.bullets.len() < fill {
            let k = Kind::ALL[(self.rng.next_u64() % 5) as usize];
            let x = self.rng.range(0.0, FIELD_W as f32);
            let a = self.rng.range(60.0, 120.0);
            let v = self.rng.range(0.8, 2.6);
            let (c, s) = deg(a);
            self.bullets.push(Spawn { x, y: -BULLET_MARGIN * 0.5, vx: c * v, vy: s * v, kind: k, behavior: Behavior::Straight, owner: 0 });
        }
    }
}

impl Emitter {
    pub fn new(gun: Gun, owner: u32, angle: f32) -> Emitter {
        Emitter { gun, t: 0, angle, dir: 1.0, shots: 0, since: u32::MAX, x: 0.0, y: 0.0, live: true, owner }
    }

    /// Ticks until it next fires (a burst's first shot, for streams), if its mount lets it.
    pub fn until_next(&self) -> u32 {
        let g = self.gun;
        let next = self.t + 1;
        if next <= g.delay {
            return g.delay + 1 - self.t;
        }
        let r = (next - g.delay - 1) % g.every.max(1);
        if r == 0 { 1 } else { g.every - r + 1 }
    }

    fn fire_frame(&mut self, player: (f32, f32), mounts: Mounts, rng: &mut Rng, out: &mut Vec<Spawn>) {
        let g = self.gun;
        // where the mount is
        (self.x, self.y, self.live) = match g.mount {
            Mount::At(x, y) => (x, y, true),
            Mount::Dummy(i) => mounts.get(i as usize).copied().unwrap_or((0.0, 0.0, false)),
            Mount::Sweep { y, x0, x1, period } => {
                let k = 0.5 - 0.5 * (std::f32::consts::TAU * self.t as f32 / period.max(1) as f32).cos();
                (x0 + (x1 - x0) * k, y, true)
            }
            Mount::Owner { dx, dy } => mounts.first().map_or((0.0, 0.0, false), |&(x, y, ok)| (x + dx, y + dy, ok)),
        };
        self.t += 1;
        self.since = self.since.saturating_add(1);
        if !self.live || self.t <= g.delay {
            return;
        }
        let k = self.t - g.delay - 1;
        let aim = (player.1 - self.y).atan2(player.0 - self.x).to_degrees();
        let shot = |angle: f32, speed: f32, out: &mut Vec<Spawn>| {
            let (c, s) = deg(angle);
            Hazards::fire(out, Spawn { x: self.x, y: self.y, vx: c * speed, vy: s * speed, kind: g.kind, behavior: g.behavior, owner: self.owner }, player);
        };
        let fired = match g.shape {
            Shape::Stream { burst, gap } => {
                let j = k % g.every;
                if j % gap.max(1) as u32 == 0 && j / (gap.max(1) as u32) < burst as u32 {
                    shot(aim, g.speed, out);
                    true
                } else {
                    false
                }
            }
            _ if k % g.every != 0 => false,
            Shape::Fan { n, spread, aimed } => {
                let center = if aimed { aim } else { 90.0 };
                for i in 0..n {
                    let a = if n > 1 { center - spread * 0.5 + spread * i as f32 / (n - 1) as f32 } else { center };
                    shot(a, g.speed, out);
                }
                true
            }
            Shape::Ring { n, turn } => {
                for i in 0..n {
                    shot(self.angle + 360.0 * i as f32 / n as f32, g.speed, out);
                }
                self.angle += turn;
                true
            }
            Shape::Spiral { arms, spin, flip } => {
                for i in 0..arms {
                    shot(self.angle + 360.0 * i as f32 / arms as f32, g.speed, out);
                }
                self.angle += spin * self.dir;
                self.shots += 1;
                if flip > 0 && self.shots % flip as u32 == 0 {
                    self.dir = -self.dir;
                }
                true
            }
            Shape::Cone { n, spread, jitter } => {
                for _ in 0..n {
                    let a = 90.0 + rng.range(-spread * 0.5, spread * 0.5);
                    let v = g.speed * (1.0 + rng.range(-jitter, jitter));
                    shot(a, v, out);
                }
                true
            }
        };
        if fired {
            self.since = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMMIES: [(f32, f32, bool); 3] = [(62.0, 86.0, true), (135.0, 170.0, true), (196.0, 78.0, true)];

    fn guns(i: usize) -> bool {
        !SANDBOX[i].guns.is_empty()
    }

    #[test]
    fn presets_are_named_and_shaped() {
        let mut names: Vec<&str> = SANDBOX.iter().map(|p| p.name).collect();
        names.dedup();
        assert_eq!(names.len(), SANDBOX.len(), "names are unique");
        assert_eq!(SANDBOX[0].name, "OFF");
        assert!(SANDBOX.iter().any(|p| p.fill >= 4000), "the stress scene");
        for p in SANDBOX {
            for g in p.guns {
                assert!(g.every > 0 && g.speed > 0.0, "{}", p.name);
            }
        }
    }

    #[test]
    fn every_pattern_fires_and_keeps_its_bullets_on_the_field() {
        for i in (0..SANDBOX.len()).filter(|&i| guns(i)) {
            let mut h = Hazards::new(i, 9);
            let mut most = 0;
            for _ in 0..1200 {
                h.step((135.0, 300.0), &DUMMIES);
                most = most.max(h.bullets.len());
            }
            assert!(most > 10, "{} fires", SANDBOX[i].name);
            assert!(most < 1500, "{} doesn't pile up: {most}", SANDBOX[i].name);
        }
    }

    #[test]
    fn stress_presets_hold_their_count() {
        let i = SANDBOX.iter().position(|p| p.fill == 4000).unwrap();
        let mut h = Hazards::new(i, 1);
        for _ in 0..300 {
            h.step((135.0, 300.0), &DUMMIES);
            assert!(h.bullets.len() >= 4000);
        }
    }

    #[test]
    fn a_broken_mount_falls_silent() {
        let i = SANDBOX.iter().position(|p| p.guns.iter().any(|g| g.mount == Mount::Dummy(1))).unwrap();
        let mut h = Hazards::new(i, 1);
        let dead = [(62.0, 86.0, true), (135.0, 170.0, false), (196.0, 78.0, true)];
        for _ in 0..600 {
            h.step((135.0, 300.0), &dead);
        }
        let from_dummy = h.emitters.iter().filter(|e| e.gun.mount == Mount::Dummy(1)).all(|e| e.since == u32::MAX);
        assert!(from_dummy, "never fired");
    }

    #[test]
    fn spirals_reverse_and_streams_burst() {
        let spiral = Gun { mount: Mount::At(135.0, 60.0), shape: Shape::Spiral { arms: 1, spin: 10.0, flip: 3 }, kind: Kind::Rice, speed: 2.0, every: 1, delay: 0, behavior: Behavior::Straight };
        let mut e = Emitter::new(spiral, 0, 0.0);
        let mut out = vec![];
        let mut rng = Rng::new(1);
        let mut angles = vec![];
        for _ in 0..6 {
            angles.push(e.angle);
            e.fire_frame((135.0, 300.0), &DUMMIES, &mut rng, &mut out);
        }
        assert_eq!(angles, vec![0.0, 10.0, 20.0, 30.0, 20.0, 10.0]);

        let stream = Gun { shape: Shape::Stream { burst: 4, gap: 3 }, every: 30, kind: Kind::Needle, ..spiral };
        let mut e = Emitter { gun: stream, angle: 0.0, ..e };
        e.t = 0;
        out.clear();
        let mut frames = vec![];
        for f in 0..60 {
            let before = out.len();
            e.fire_frame((135.0, 300.0), &DUMMIES, &mut rng, &mut out);
            if out.len() > before {
                frames.push(f);
            }
        }
        assert_eq!(frames, vec![0, 3, 6, 9, 30, 33, 36, 39]);
        assert!(out.iter().all(|s| s.vx.abs() < 1e-4 && s.vy > 0.0), "aimed straight down at the player below");
    }

    #[test]
    fn until_next_counts_down_to_each_shot() {
        let g = Gun { mount: Mount::Owner { dx: 0.0, dy: 5.0 }, shape: Shape::Fan { n: 3, spread: 20.0, aimed: true }, kind: Kind::Small, speed: 2.0, every: 10, delay: 4, behavior: Behavior::Straight };
        let mut e = Emitter::new(g, 7, 0.0);
        let (mut rng, mut out) = (Rng::new(1), vec![]);
        for _ in 0..40 {
            let promised = e.until_next();
            let mut fired_at = 0;
            for k in 1..=promised {
                let before = out.len();
                e.fire_frame((135.0, 300.0), &[(100.0, 50.0, true)], &mut rng, &mut out);
                if out.len() > before {
                    fired_at = k;
                    break;
                }
            }
            assert_eq!(fired_at, promised, "it fires exactly when it said it would");
        }
        assert!(out.iter().all(|s| s.owner == 7 && s.y == 55.0), "owned, and fired from the enemy plus the offset");
        // a gun whose enemy may not fire stays quiet
        let mut e = Emitter::new(g, 7, 0.0);
        out.clear();
        for _ in 0..60 {
            e.fire_frame((135.0, 300.0), &[(100.0, 50.0, false)], &mut rng, &mut out);
        }
        assert!(out.is_empty());
    }
}
