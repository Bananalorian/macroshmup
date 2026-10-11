//! Cosmetic particles: sparks, embers, shards with red and teal fringes, rings,
//! spectral streaks, stipple dust, the engine trail. Pooled (never allocates during
//! play), advanced once per sim tick, and seeded apart from gameplay so effects can
//! never change the game. Sparks, embers and streaks add light (drawn in a second,
//! additive pass); the rest cover what's under them.

use super::art::Atlas;
use super::palette::{ASH, CREAM, SIGNAL, SLATE, TEAL};
use crate::sim::rng::Rng;
use crate::tune::{Blast, PARTICLE_CAP};
use macroquad::color::Color;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    /// A streak along its velocity (additive).
    Spark,
    /// A glowing dot that drifts, slows and flickers (additive).
    Ember,
    /// A 1-2 px chip of debris with a red or teal fringe pixel.
    Shard,
    /// An expanding ring of dots.
    Ring,
    /// A thin red streak with hints of green and blue beside it, trailing fast debris (additive).
    Spectral,
    /// Slow stipple dust that hangs after a blast.
    Dust,
    /// A single fading pixel (the engine trail).
    Dot,
    /// A small cross that flares and fades (grazes, sparkles in blasts) (additive).
    Cross,
    /// A blast's hot core: a dithered disc that swells and fades fast (additive).
    Core,
}

impl Kind {
    fn additive(self) -> bool {
        matches!(self, Kind::Spark | Kind::Ember | Kind::Spectral | Kind::Cross | Kind::Core)
    }
}

#[derive(Clone, Copy)]
struct P {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: u16,
    max: u16,
    kind: Kind,
    color: Color,
    /// Velocity kept each frame.
    drag: f32,
    /// Rings: growth per frame. Shards: which side the fringe is on (+-1).
    extra: f32,
}

pub struct Particles {
    list: Vec<P>,
    rng: Rng,
}

impl Particles {
    pub fn new() -> Particles {
        Particles { list: Vec::with_capacity(PARTICLE_CAP), rng: Rng::new(0xC0FFEE) }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    fn push(&mut self, p: P) {
        if self.list.len() < PARTICLE_CAP {
            self.list.push(p);
        }
    }

    fn spawn(&mut self, kind: Kind, x: f32, y: f32, angle: f32, speed: f32, life: f32, color: Color, drag: f32) {
        let life = life.max(1.0) as u16;
        let extra = if self.rng.chance(0.5) { 1.0 } else { -1.0 };
        self.push(P { x, y, vx: angle.cos() * speed, vy: angle.sin() * speed, life, max: life, kind, color, drag, extra });
    }

    pub fn tick(&mut self) {
        for p in &mut self.list {
            p.x += p.vx;
            p.y += p.vy;
            p.vx *= p.drag;
            p.vy *= p.drag;
            if p.kind == Kind::Ember {
                p.vy -= 0.02; // embers rise a little
            }
            p.life -= 1;
        }
        self.list.retain(|p| p.life > 0);
    }

    /// A shot landing: a few sparks kicked back down.
    pub fn hit(&mut self, x: f32, y: f32, beam: bool) {
        for _ in 0..if beam { 2 } else { 3 } {
            let a = self.rng.range(0.4, 2.75);
            let s = self.rng.range(1.0, 3.2);
            let life = self.rng.range(5.0, 10.0);
            let c = if self.rng.chance(0.3) { Color { a: 0.8, ..SIGNAL } } else { CREAM };
            self.spawn(Kind::Spark, x, y, a, s, life, c, 0.9);
        }
    }

    /// A graze: teal sparks flicking away from the bullet, and a flare where it
    /// passed (out on the dark beside the hull, so it reads as teal).
    pub fn graze(&mut self, x: f32, y: f32, px: f32, py: f32) {
        let away = (y - py).atan2(x - px);
        let (gx, gy) = (px + away.cos() * 9.0, py + away.sin() * 9.0);
        for _ in 0..7 {
            let a = away + self.rng.range(-0.9, 0.9);
            let s = self.rng.range(1.4, 3.2);
            let life = self.rng.range(10.0, 18.0);
            self.spawn(Kind::Spark, gx, gy, a, s, life, TEAL, 0.9);
        }
        self.spawn(Kind::Cross, gx, gy, 0.0, 0.0, 10.0, TEAL, 1.0);
    }

    /// A bullet cancelled by a death: a red pop.
    pub fn cancel(&mut self, x: f32, y: f32) {
        let a = self.rng.range(0.0, std::f32::consts::TAU);
        let life = self.rng.range(8.0, 16.0);
        let c = if self.rng.chance(0.3) { CREAM } else { SIGNAL };
        self.spawn(Kind::Ember, x, y, a, 0.6, life, c, 0.92);
    }

    /// The teal engine trail, falling away behind the ship.
    pub fn trail(&mut self, x: f32, y: f32, thrust: bool) {
        let life = self.rng.range(8.0, if thrust { 22.0 } else { 14.0 });
        let a = std::f32::consts::FRAC_PI_2 + self.rng.range(-0.15, 0.15);
        let s = self.rng.range(1.0, if thrust { 3.0 } else { 1.8 });
        let x = x + self.rng.range(-1.0, 1.0);
        self.spawn(Kind::Dot, x, y, a, s, life, TEAL, 0.92);
    }

    /// An explosion's debris: sparks, embers, fringed shards, spectral streaks,
    /// rings and dust, all scaled by its size.
    pub fn blast(&mut self, x: f32, y: f32, b: &Blast) {
        use std::f32::consts::TAU;
        // the hot core, swelling and gone in a few frames
        let life = 6 + 2 * b.rings as u16;
        self.push(P { x, y, vx: 0.0, vy: 0.0, life, max: life, kind: Kind::Core, color: CREAM, drag: 1.0, extra: b.speed * 1.6 });
        // sparkles: little crosses in red, teal and cream, like the sheet's destroy frames
        for i in 0..(2 + 3 * b.rings as u16) {
            let a = self.rng.range(0.0, TAU);
            let s = self.rng.range(0.3, 0.9) * b.speed;
            let c = [SIGNAL, TEAL, CREAM][i as usize % 3];
            let life = self.rng.range(10.0, 26.0);
            self.spawn(Kind::Cross, x, y, a, s, life, Color { a: 0.8, ..c }, 0.9);
        }
        for i in 0..b.rings {
            let grow = b.speed * (0.7 + 0.45 * i as f32);
            let life = 12 + 5 * i as u16;
            self.push(P { x, y, vx: 0.0, vy: 0.0, life, max: life, kind: Kind::Ring, color: CREAM, drag: 1.0, extra: grow });
        }
        for _ in 0..b.sparks {
            let a = self.rng.range(0.0, TAU);
            let s = self.rng.range(0.4, 1.0) * b.speed * 1.6;
            let c = if self.rng.chance(0.35) { SIGNAL } else { CREAM };
            let life = self.rng.range(8.0, 22.0);
            self.spawn(Kind::Spark, x, y, a, s, life, c, 0.9);
        }
        for _ in 0..b.embers {
            let a = self.rng.range(0.0, TAU);
            let s = self.rng.range(0.2, 0.7) * b.speed;
            let c = if self.rng.chance(0.5) { SIGNAL } else { Color { a: 0.8, ..CREAM } };
            let life = self.rng.range(24.0, 60.0);
            self.spawn(Kind::Ember, x, y, a, s, life, c, 0.94);
        }
        for _ in 0..b.shards {
            let a = self.rng.range(0.0, TAU);
            let s = self.rng.range(0.3, 1.0) * b.speed;
            let c = if self.rng.chance(0.5) { ASH } else { SLATE };
            let life = self.rng.range(20.0, 46.0);
            self.spawn(Kind::Shard, x, y, a, s, life, c, 0.95);
        }
        for _ in 0..b.streaks {
            let a = self.rng.range(0.0, TAU);
            let s = self.rng.range(0.8, 1.3) * b.speed * 1.8;
            let life = self.rng.range(10.0, 20.0);
            self.spawn(Kind::Spectral, x, y, a, s, life, SIGNAL, 0.93);
        }
        for _ in 0..b.dust {
            let a = self.rng.range(0.0, TAU);
            let s = self.rng.range(0.05, 0.4) * b.speed;
            let life = self.rng.range(40.0, 90.0);
            let (dx, dy) = (self.rng.range(-4.0, 4.0), self.rng.range(-4.0, 4.0));
            self.spawn(Kind::Dust, x + dx, y + dy, a, s, life, CREAM, 0.97);
        }
    }

    /// The stress scenes: keep at least `n` particles alive across a `w` x `h` field.
    pub fn fill(&mut self, n: usize, w: f32, h: f32) {
        while self.list.len() < n.min(PARTICLE_CAP) {
            let (x, y) = (self.rng.range(0.0, w), self.rng.range(0.0, h));
            let a = self.rng.range(0.0, std::f32::consts::TAU);
            let life = self.rng.range(20.0, 90.0);
            match self.rng.next_u64() % 3 {
                0 => self.spawn(Kind::Ember, x, y, a, 0.3, life, SIGNAL, 0.98),
                1 => self.spawn(Kind::Dust, x, y, a, 0.2, life, CREAM, 0.98),
                _ => self.spawn(Kind::Spark, x, y, a, 1.5, life * 0.35, CREAM, 0.95),
            }
        }
    }

    /// Draw one of the two passes (normal, then additive), offset by the playfield's corner.
    pub fn draw(&self, a: &Atlas, ox: i32, oy: i32, additive: bool) {
        for p in self.list.iter().filter(|p| p.kind.additive() == additive) {
            let k = p.life as f32 / p.max as f32;
            let (x, y) = (ox + p.x.round() as i32, oy + p.y.round() as i32);
            let fade = |c: Color, m: f32| Color { a: c.a * m, ..c };
            match p.kind {
                Kind::Dot => a.fill(x, y, 1, 1, fade(p.color, k)),
                Kind::Ember => {
                    let flick = if (p.life / 3) % 2 == 0 { 1.0 } else { 0.6 };
                    a.fill(x, y, 1, 1, fade(p.color, k * flick));
                }
                Kind::Spark => {
                    // a 3 px streak back along its velocity
                    for s in 0..3 {
                        let t = s as f32 * 0.6;
                        let (sx, sy) = ((p.x - p.vx * t).round() as i32, (p.y - p.vy * t).round() as i32);
                        a.fill(ox + sx, oy + sy, 1, 1, fade(p.color, k * (1.0 - s as f32 / 3.0)));
                    }
                }
                Kind::Spectral => {
                    // red-dominant, with green and blue a pixel to either side
                    let len = 5;
                    let v = p.vx.hypot(p.vy).max(0.01);
                    let (ux, uy) = (p.vx / v, p.vy / v);
                    let (nx, ny) = (-uy, ux);
                    for s in 0..len {
                        let t = s as f32;
                        let (sx, sy) = (p.x - ux * t, p.y - uy * t);
                        let m = k * (1.0 - t / len as f32);
                        a.fill(ox + sx.round() as i32, oy + sy.round() as i32, 1, 1, Color::new(0.97, 0.24, 0.19, 0.9 * m));
                        if s % 2 == 0 {
                            a.fill(ox + (sx + nx).round() as i32, oy + (sy + ny).round() as i32, 1, 1, Color::new(0.1, 0.8, 0.3, 0.35 * m));
                            a.fill(ox + (sx - nx).round() as i32, oy + (sy - ny).round() as i32, 1, 1, Color::new(0.2, 0.35, 1.0, 0.35 * m));
                        }
                    }
                }
                Kind::Shard => {
                    a.fill(x, y, 2, 1, fade(p.color, k.min(1.0) * 0.9 + 0.1));
                    let fringe = if p.extra > 0.0 { SIGNAL } else { TEAL };
                    a.fill(x + if p.extra > 0.0 { 2 } else { -1 }, y, 1, 1, fade(fringe, k));
                }
                Kind::Dust => {
                    // stipple: blinks on and off as it fades
                    if (p.life + (p.x as u16)) % 3 != 0 {
                        a.fill(x, y, 1, 1, fade(Color { a: 0.45, ..p.color }, k));
                    }
                }
                Kind::Ring => {
                    let r = (p.max - p.life) as f32 * p.extra + 2.0;
                    let dots = (r * 1.4).clamp(8.0, 96.0) as i32;
                    for d in 0..dots {
                        let ang = d as f32 / dots as f32 * std::f32::consts::TAU;
                        let c = fade(Color { a: 0.75, ..p.color }, k);
                        a.fill(ox + (p.x + ang.cos() * r).round() as i32, oy + (p.y + ang.sin() * r).round() as i32, 1, 1, c);
                    }
                }
                Kind::Cross => {
                    let r = (1 + ((p.max - p.life) / 3) as i32).min(3);
                    let c = fade(p.color, k);
                    a.fill(x - r, y, 2 * r + 1, 1, c);
                    a.fill(x, y - r, 1, 2 * r + 1, c);
                }
                Kind::Core => {
                    // a disc, solid in the middle and dithered at the rim
                    let age = (p.max - p.life) as f32;
                    let r = (2.0 + age * p.extra * 0.5).min(p.extra * 4.0);
                    let ri = r.ceil() as i32;
                    for dy in -ri..=ri {
                        for dx in -ri..=ri {
                            let d = ((dx * dx + dy * dy) as f32).sqrt();
                            if d > r || (d > r * 0.6 && (dx + dy + p.life as i32) % 2 != 0) {
                                continue;
                            }
                            let c = if d < r * 0.45 { CREAM } else { SIGNAL };
                            a.fill(x + dx, y + dy, 1, 1, Color { a: 0.55 * k, ..c });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tune::BLASTS;

    #[test]
    fn the_pool_is_capped_and_never_grows() {
        let mut p = Particles::new();
        let cap = p.list.capacity();
        for _ in 0..200 {
            p.blast(100.0, 100.0, &BLASTS[3]);
        }
        assert_eq!(p.len(), PARTICLE_CAP);
        assert_eq!(p.list.capacity(), cap);
        for _ in 0..120 {
            p.tick();
        }
        assert_eq!(p.len(), 0, "everything burns out");
    }

    #[test]
    fn bigger_blasts_throw_more_of_everything() {
        let mut last = 0;
        for b in &BLASTS {
            let mut p = Particles::new();
            p.blast(0.0, 0.0, b);
            assert!(p.len() > last);
            last = p.len();
            let kinds: Vec<Kind> = p.list.iter().map(|q| q.kind).collect();
            for k in [Kind::Spark, Kind::Ember, Kind::Shard, Kind::Spectral, Kind::Dust, Kind::Ring] {
                assert!(kinds.contains(&k), "{k:?}");
            }
        }
    }

    #[test]
    fn grazes_are_easy_to_see() {
        let mut p = Particles::new();
        p.graze(10.0, 0.0, 0.0, 0.0);
        let teal = p.list.iter().filter(|q| q.color == TEAL).count();
        assert!(teal >= 7);
        let flare = p.list.iter().find(|q| q.kind == Kind::Cross).expect("a flare");
        assert!(flare.x > 5.0, "out beside the hull, toward the bullet");
        assert!(p.list.iter().all(|q| q.max >= 8), "they last long enough to notice");
    }

    #[test]
    fn stress_fill_keeps_the_count_up() {
        let mut p = Particles::new();
        for _ in 0..100 {
            p.fill(6000, 270.0, 360.0);
            p.tick();
        }
        p.fill(6000, 270.0, 360.0);
        assert!(p.len() >= 6000);
    }
}
