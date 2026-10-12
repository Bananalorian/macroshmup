//! Cosmetic particles: sparks, embers, shards with red and teal fringes, rings,
//! spectral streaks, stipple dust, the engine trail. Pooled (never allocates during
//! play), advanced once per sim tick, and seeded apart from gameplay so effects can
//! never change the game. Sparks, embers and streaks add light (drawn in a second,
//! additive pass); the rest cover what's under them.

use super::art::Atlas;
use super::palette::{ASH, CREAM, SIGNAL, SLATE, TEAL};

/// The boss's stone, for its glitch smears (inside the enemies' value budget).
const BONE_HI: Color = Color::new(0.769, 0.741, 0.659, 0.9);
const BONE_MID: Color = Color::new(0.616, 0.588, 0.51, 0.85);
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
    /// One pixel of the shattered Pilgrim, in its own color.
    Pixel,
    /// A teal shard from a bomb-cancelled bullet, flying to the Pilgrim (additive).
    Homing,
    /// Dark smoke, swelling and thinning (the bomb, like Shinjuku's grenade).
    Smoke,
    /// A glitch smear: a short streak of the boss's stone, or red or teal, sliding
    /// back off it and flickering out (`extra` is its length).
    Smear,
    /// A thread of glitch dripping off the boss's belly (`extra` is its length).
    Drip,
}

impl Kind {
    fn additive(self) -> bool {
        matches!(self, Kind::Spark | Kind::Ember | Kind::Spectral | Kind::Cross | Kind::Core | Kind::Homing)
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
    /// Kills' impact frames: (silhouette, top-left, frames left, of).
    impacts: Vec<(macroquad::math::Rect, (i32, i32), u8, u8)>,
    rng: Rng,
}

impl Particles {
    pub fn new() -> Particles {
        Particles { list: Vec::with_capacity(PARTICLE_CAP), impacts: Vec::new(), rng: Rng::new(0xC0FFEE) }
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

    /// A kill's impact frame: the silhouette `sil` of what died, its top-left corner
    /// at `at`, lit bone-bright for `frames` frames before it's gone.
    pub fn impact(&mut self, sil: macroquad::math::Rect, at: (i32, i32), frames: u8) {
        if frames > 0 && self.impacts.len() < 64 {
            self.impacts.push((sil, at, frames, frames));
        }
    }

    /// Advance a frame. `target` is the Pilgrim, which teal shards fly to.
    pub fn tick(&mut self, target: (f32, f32)) {
        for i in &mut self.impacts {
            i.2 -= 1;
        }
        self.impacts.retain(|i| i.2 > 0);
        for p in &mut self.list {
            p.x += p.vx;
            p.y += p.vy;
            p.vx *= p.drag;
            p.vy *= p.drag;
            match p.kind {
                Kind::Ember => p.vy -= 0.02, // embers rise a little
                Kind::Homing if p.max - p.life > 8 => {
                    // after a short burst outward, home in, faster and faster
                    let (dx, dy) = (target.0 - p.x, target.1 - p.y);
                    let d = dx.hypot(dy).max(0.001);
                    let pull = 0.9 + (p.max - p.life) as f32 * 0.04;
                    p.vx = p.vx * 0.8 + dx / d * pull;
                    p.vy = p.vy * 0.8 + dy / d * pull;
                    if d < 6.0 {
                        p.life = 1; // collected
                    }
                }
                _ => {}
            }
            p.life -= 1;
        }
        self.list.retain(|p| p.life > 0);
    }

    /// The Pilgrim shattering into its own pixels, flung outward from its center.
    pub fn shatter(&mut self, x: f32, y: f32, pixels: &[(i8, i8, Color)]) {
        for &(dx, dy, c) in pixels {
            let (fx, fy) = (dx as f32, dy as f32);
            let a = fy.atan2(fx) + self.rng.range(-0.5, 0.5);
            let s = self.rng.range(0.3, 2.4) * (0.6 + fx.hypot(fy) * 0.06);
            let life = self.rng.range(30.0, 90.0);
            self.spawn(Kind::Pixel, x + fx, y + fy, a, s, life, c, 0.965);
        }
    }

    /// A bomb-cancelled bullet: a teal shard that flies to the Pilgrim.
    pub fn shard_to_you(&mut self, x: f32, y: f32) {
        let a = self.rng.range(0.0, std::f32::consts::TAU);
        let s = self.rng.range(0.6, 1.8);
        self.spawn(Kind::Homing, x, y, a, s, 90.0, TEAL, 0.94);
    }

    /// Dark smoke rolling out from a blast.
    pub fn smoke(&mut self, x: f32, y: f32, n: u16, spread: f32) {
        for _ in 0..n {
            let a = self.rng.range(0.0, std::f32::consts::TAU);
            let s = self.rng.range(0.2, 1.0) * spread * 0.05;
            let life = self.rng.range(40.0, 80.0);
            let r = self.rng.range(2.0, 4.5);
            let (ox, oy) = (a.cos() * spread * 0.3, a.sin() * spread * 0.3);
            self.push(P { x: x + ox, y: y + oy, vx: a.cos() * s, vy: a.sin() * s - 0.15, life: life as u16, max: life as u16, kind: Kind::Smoke, color: SLATE, drag: 0.97, extra: r });
        }
    }

    /// A big hot glow, like the grenade's temporary light.
    pub fn glow(&mut self, x: f32, y: f32, grow: f32, life: u16) {
        self.push(P { x, y, vx: 0.0, vy: 0.0, life, max: life, kind: Kind::Core, color: CREAM, drag: 1.0, extra: grow });
    }

    /// An expanding ring of dots.
    pub fn ring(&mut self, x: f32, y: f32, grow: f32, life: u16, color: Color) {
        self.push(P { x, y, vx: 0.0, vy: 0.0, life, max: life, kind: Kind::Ring, color, drag: 1.0, extra: grow });
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
        // now and then a chip knocked off what was hit
        if !beam && self.rng.chance(0.5) {
            let a = self.rng.range(0.6, 2.55) + std::f32::consts::PI;
            let s = self.rng.range(0.6, 1.6);
            let life = self.rng.range(10.0, 18.0);
            let c = if self.rng.chance(0.5) { ASH } else { CREAM };
            self.spawn(Kind::Shard, x, y, a, s, life, c, 0.92);
        }
    }

    /// Where the beam lands: teal sparks spraying out sideways along what it hit.
    pub fn beam_spray(&mut self, x: f32, y: f32) {
        for side in [0.0, std::f32::consts::PI] {
            let a = side + self.rng.range(-0.35, 0.15) * if side == 0.0 { 1.0 } else { -1.0 };
            let s = self.rng.range(1.2, 2.8);
            let life = self.rng.range(5.0, 11.0);
            let c = if self.rng.chance(0.3) { CREAM } else { TEAL };
            self.spawn(Kind::Spark, x, y, a, s, life, c, 0.88);
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

    /// The Pilgrim's blades locking together: teal sparks flicked sideways off the
    /// crossbar where they meet, and a glint at the sword's tip.
    pub fn lock(&mut self, x: f32, bar: f32, tip: f32) {
        for side in [0.0, std::f32::consts::PI] {
            for _ in 0..2 {
                let a = side + self.rng.range(-0.5, 0.5);
                let s = self.rng.range(1.0, 2.2);
                let life = self.rng.range(5.0, 9.0);
                self.spawn(Kind::Spark, x, bar, a, s, life, TEAL, 0.85);
            }
        }
        self.spawn(Kind::Cross, x, tip, 0.0, 0.0, 8.0, TEAL, 1.0);
    }

    /// An enemy's engine: a gray dot thrown up behind it (they fly nose down).
    pub fn exhaust(&mut self, x: f32, y: f32) {
        let a = -std::f32::consts::FRAC_PI_2 + self.rng.range(-0.25, 0.25);
        let s = self.rng.range(0.6, 1.6);
        let life = self.rng.range(6.0, 14.0);
        let x = x + self.rng.range(-1.0, 1.0);
        self.spawn(Kind::Dot, x, y, a, s, life, Color { a: 0.8, ..ASH }, 0.92);
    }

    /// A flake of metal shed by the derelict as it rises out of the disk.
    pub fn flake(&mut self, x: f32, y: f32) {
        let a = self.rng.range(0.0, std::f32::consts::TAU);
        let s = self.rng.range(0.1, 0.6);
        let life = self.rng.range(30.0, 70.0);
        let c = if self.rng.chance(0.5) { ASH } else { SLATE };
        self.spawn(Kind::Shard, x, y, a, s, life, c, 0.98);
    }

    /// A damaged enemy: a puff of smoke, and now and then an ember or a spark.
    pub fn wound(&mut self, x: f32, y: f32) {
        self.smoke(x, y, 1, 4.0);
        if self.rng.chance(0.3) {
            let a = self.rng.range(0.0, std::f32::consts::TAU);
            let life = self.rng.range(12.0, 24.0);
            let c = if self.rng.chance(0.5) { SIGNAL } else { CREAM };
            self.spawn(Kind::Ember, x, y, a, 0.5, life, c, 0.95);
        }
    }

    /// Somewhere inside a `w` x `h` box centered on `(x, y)` (cosmetic randomness).
    pub fn somewhere(&mut self, x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
        (x + self.rng.range(-w * 0.5, w * 0.5), y + self.rng.range(-h * 0.5, h * 0.5))
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

    /// A glitch smear coming off the boss at `(x, y)`, sliding the way it's leaving
    /// (`back`: -1 left, 1 right); `hot` makes more of them red and teal.
    pub fn smear(&mut self, x: f32, y: f32, back: f32, hot: bool) {
        let len = self.rng.range(2.0, 10.0).round();
        let roll = self.rng.f32();
        let color = if roll < if hot { 0.22 } else { 0.1 } {
            SIGNAL
        } else if roll < if hot { 0.4 } else { 0.18 } {
            TEAL
        } else if roll < 0.7 {
            BONE_HI
        } else {
            BONE_MID
        };
        let speed = self.rng.range(0.3, 2.4);
        let life = self.rng.range(6.0, 22.0);
        self.spawn(Kind::Smear, x, y, if back < 0.0 { std::f32::consts::PI } else { 0.0 }, speed, life, color, 0.95);
        if let Some(p) = self.list.last_mut().filter(|p| p.kind == Kind::Smear) {
            p.extra = len;
        }
    }

    /// A thread of glitch dripping off the boss's belly.
    pub fn drip(&mut self, x: f32, y: f32) {
        let color = if self.rng.chance(0.25) { SIGNAL } else { BONE_MID };
        let life = self.rng.range(24.0, 50.0);
        let speed = self.rng.range(0.15, 0.5);
        self.spawn(Kind::Drip, x, y, std::f32::consts::FRAC_PI_2, speed, life, Color { a: 0.8, ..color }, 1.0);
        if let Some(p) = self.list.last_mut().filter(|p| p.kind == Kind::Drip) {
            p.extra = self.rng.range(2.0, 6.0).round();
        }
    }

    /// The disk churned up where something breaks its surface: cream dust thrown up
    /// and out along the line.
    pub fn splash(&mut self, x: f32, y: f32, width: f32, n: u16) {
        for _ in 0..n {
            let dx = self.rng.range(-width * 0.5, width * 0.5);
            let a = -std::f32::consts::FRAC_PI_2 + self.rng.range(-0.9, 0.9);
            let s = self.rng.range(0.4, 2.2);
            let life = self.rng.range(20.0, 50.0);
            let kind = if self.rng.chance(0.6) { Kind::Dust } else { Kind::Shard };
            let c = if kind == Kind::Dust { CREAM } else { ASH };
            self.spawn(kind, x + dx, y, a, s, life, c, 0.95);
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
        if !additive {
            for &(sil, (x, y), left, of) in &self.impacts {
                a.blit(sil, ox + x, oy + y, 1, Color { a: 0.95 * (left as f32 / of as f32).sqrt(), ..CREAM });
            }
        }
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
                Kind::Pixel => a.fill(x, y, 1, 1, fade(p.color, (k * 1.6).min(1.0))),
                Kind::Homing => {
                    a.fill(x, y, 1, 1, fade(p.color, 0.9));
                    let (tx, ty) = ((p.x - p.vx).round() as i32, (p.y - p.vy).round() as i32);
                    a.fill(ox + tx, oy + ty, 1, 1, fade(p.color, 0.45));
                }
                Kind::Smoke => {
                    // a dithered blob that swells and thins out
                    let age = (p.max - p.life) as f32;
                    disc(a, &a.r.stipple, x, y, p.extra + age * 0.12, fade(Color { a: 0.32, ..p.color }, k));
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
                    // a red disc, dithered at the rim, with a cream heart
                    let age = (p.max - p.life) as f32;
                    let r = (2.0 + age * p.extra * 0.5).min(p.extra * 4.0);
                    disc(a, &a.r.discs, x, y, r, Color { a: 0.55 * k, ..SIGNAL });
                    disc(a, &a.r.discs, x, y, r * 0.45, Color { a: 0.55 * k, ..CREAM });
                }
                Kind::Smear => {
                    // flickers in and out as it slides away
                    if (p.life / 2) % 3 != 0 {
                        let w = p.extra as i32;
                        a.fill(x - w / 2, y, w, 1, fade(p.color, (k * 1.5).min(1.0)));
                    }
                }
                Kind::Drip => a.fill(x, y, 1, p.extra as i32, fade(p.color, k)),
            }
        }
    }
}

/// Blit the pre-drawn disc nearest radius `r` (doubled up past 16 px), centered on `(x, y)`.
fn disc(a: &Atlas, set: &[macroquad::math::Rect; 16], x: i32, y: i32, r: f32, c: Color) {
    let (scale, r) = if r > 16.5 { (2, r * 0.5) } else { (1, r) };
    let i = (r.round() as usize).clamp(1, 16) - 1;
    let half = (i as i32 + 1) * scale;
    a.blit(set[i], x - half, y - half, scale, c);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tune::BLASTS;

    /// Shots knock chips off what they hit (the beam doesn't); where the beam lands,
    /// sparks spray out sideways along what it hit.
    #[test]
    fn hits_chip_and_the_beam_sprays_sideways() {
        let mut p = Particles::new();
        for _ in 0..20 {
            p.hit(50.0, 50.0, false);
        }
        assert!(p.list.iter().filter(|q| q.kind == Kind::Shard).count() >= 4, "chips fly");
        let mut p = Particles::new();
        for _ in 0..20 {
            p.hit(50.0, 50.0, true);
        }
        assert_eq!(p.list.iter().filter(|q| q.kind == Kind::Shard).count(), 0, "the beam burns, it doesn't chip");
        let mut p = Particles::new();
        for _ in 0..20 {
            p.beam_spray(50.0, 50.0);
        }
        assert_eq!(p.len(), 40);
        assert!(p.list.iter().all(|q| q.vx.abs() > q.vy.abs()), "sideways");
        assert!(p.list.iter().filter(|q| q.vx < 0.0).count() == 20, "both ways");
    }

    #[test]
    fn a_kills_impact_frame_lasts_its_frames() {
        let mut p = Particles::new();
        p.impact(macroquad::math::Rect::new(0.0, 0.0, 8.0, 8.0), (10, 10), 3);
        for left in [2, 1] {
            p.tick((0.0, 0.0));
            assert_eq!(p.impacts.len(), 1, "{left} left");
        }
        p.tick((0.0, 0.0));
        assert!(p.impacts.is_empty());
    }

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
            p.tick((0.0, 0.0));
        }
        assert_eq!(p.len(), 0, "everything burns out");
    }

    /// The blades locking: two sparks flicked out each side of the crossbar, a
    /// sparkle at the tip, all teal and gone in a few frames.
    #[test]
    fn the_lock_flicks_sparks_both_ways_and_flares_the_tip() {
        let mut p = Particles::new();
        p.lock(100.0, 109.0, 79.0);
        let sparks: Vec<&P> = p.list.iter().filter(|q| q.kind == Kind::Spark).collect();
        assert_eq!(sparks.len(), 4);
        assert_eq!(sparks.iter().filter(|q| q.vx < 0.0).count(), 2, "two each way");
        assert!(p.list.iter().any(|q| q.kind == Kind::Cross && q.y == 79.0));
        assert!(p.list.iter().all(|q| q.color.g == TEAL.g && q.life <= 10));
        for _ in 0..10 {
            p.tick((0.0, 0.0));
        }
        assert_eq!(p.len(), 0);
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
    fn bomb_shards_fly_to_you_and_are_collected() {
        let mut p = Particles::new();
        for i in 0..50 {
            p.shard_to_you(i as f32 * 5.0, 20.0);
        }
        let target = (135.0, 300.0);
        for _ in 0..80 {
            p.tick(target);
        }
        assert_eq!(p.len(), 0, "every shard reached the Pilgrim");
    }

    #[test]
    fn the_shatter_is_one_particle_per_pixel() {
        let mut p = Particles::new();
        let px = vec![(0i8, 0i8, TEAL), (3, -2, SIGNAL), (-5, 4, CREAM)];
        p.shatter(100.0, 100.0, &px);
        assert_eq!(p.len(), 3);
        assert!(p.list.iter().all(|q| q.kind == Kind::Pixel));
        assert!(p.list.iter().any(|q| q.color == SIGNAL), "pixels keep their colors");
    }

    #[test]
    fn stress_fill_keeps_the_count_up() {
        let mut p = Particles::new();
        for _ in 0..100 {
            p.fill(6000, 270.0, 360.0);
            p.tick((0.0, 0.0));
        }
        p.fill(6000, 270.0, 360.0);
        assert!(p.len() >= 6000);
    }
}
