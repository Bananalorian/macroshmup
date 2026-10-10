//! Cosmetic particles: hit sparks, the engine trail, breaking targets. Pooled,
//! advanced once per sim tick, and seeded separately from gameplay so effects
//! can never change the game. (Sprint 3 grows this into the full system.)

use super::art::Atlas;
use super::palette::{ASH, CREAM, SIGNAL, SLATE, TEAL};
use crate::sim::rng::Rng;
use macroquad::color::Color;

const MAX: usize = 4096;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// A short streak along its velocity.
    Spark,
    /// A single pixel that fades.
    Dot,
    /// An expanding ring of dots.
    Ring,
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
    /// Rings: radius growth per frame.
    grow: f32,
}

pub struct Particles {
    list: Vec<P>,
    rng: Rng,
}

impl Particles {
    pub fn new() -> Particles {
        Particles { list: Vec::with_capacity(MAX), rng: Rng::new(0xC0FFEE) }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    fn push(&mut self, p: P) {
        if self.list.len() < MAX {
            self.list.push(p);
        }
    }

    pub fn tick(&mut self) {
        for p in &mut self.list {
            p.x += p.vx;
            p.y += p.vy;
            p.vx *= 0.92;
            p.vy *= 0.92;
            p.life -= 1;
        }
        self.list.retain(|p| p.life > 0);
    }

    /// A shot landing: a few sparks kicked back down.
    pub fn hit(&mut self, x: f32, y: f32, beam: bool) {
        let n = if beam { 2 } else { 3 };
        for _ in 0..n {
            let a = self.rng.range(0.4, 2.75); // mostly downward
            let s = self.rng.range(1.0, 3.2);
            let life = self.rng.range(5.0, 10.0) as u16;
            let color = if self.rng.chance(0.3) { Color { a: 0.8, ..SIGNAL } } else { CREAM };
            self.push(P { x, y, vx: a.cos() * s, vy: a.sin() * s, life, max: life, kind: Kind::Spark, color, grow: 0.0 });
        }
    }

    /// A graze: a couple of teal sparks flicking away from the bullet.
    pub fn graze(&mut self, x: f32, y: f32, px: f32, py: f32) {
        let away = (y - py).atan2(x - px);
        for _ in 0..2 {
            let a = away + self.rng.range(-0.6, 0.6);
            let s = self.rng.range(1.0, 2.2);
            self.push(P { x, y, vx: a.cos() * s, vy: a.sin() * s, life: 8, max: 8, kind: Kind::Spark, color: TEAL, grow: 0.0 });
        }
    }

    /// A bullet cancelled by a death: a red pop.
    pub fn cancel(&mut self, x: f32, y: f32) {
        let a = self.rng.range(0.0, std::f32::consts::TAU);
        let life = self.rng.range(8.0, 16.0) as u16;
        let color = if self.rng.chance(0.3) { CREAM } else { SIGNAL };
        self.push(P { x, y, vx: a.cos() * 0.6, vy: a.sin() * 0.6, life, max: life, kind: Kind::Dot, color, grow: 0.0 });
    }

    /// A target breaking: a ring, shards with red and teal fringes, embers.
    pub fn burst(&mut self, x: f32, y: f32, big: bool) {
        let scale = if big { 2.2 } else { 1.0 };
        self.push(P { x, y, vx: 0.0, vy: 0.0, life: 16, max: 16, kind: Kind::Ring, color: CREAM, grow: 2.2 * scale });
        let n = (18.0 * scale) as usize;
        for i in 0..n {
            let a = self.rng.range(0.0, std::f32::consts::TAU);
            let s = self.rng.range(0.8, 4.5) * scale.sqrt();
            let life = self.rng.range(14.0, 34.0) as u16;
            let color = match i % 6 {
                0 => SIGNAL,
                1 => TEAL,
                2 | 3 => ASH,
                _ => SLATE,
            };
            let kind = if i % 3 == 0 { Kind::Spark } else { Kind::Dot };
            self.push(P { x, y, vx: a.cos() * s, vy: a.sin() * s, life, max: life, kind, color, grow: 0.0 });
        }
    }

    /// The teal engine trail, falling away behind the ship.
    pub fn trail(&mut self, x: f32, y: f32, thrust: bool) {
        let life = self.rng.range(8.0, if thrust { 22.0 } else { 14.0 }) as u16;
        let vx = self.rng.range(-0.25, 0.25);
        let vy = self.rng.range(1.0, if thrust { 3.0 } else { 1.8 });
        let x = x + self.rng.range(-1.0, 1.0);
        self.push(P { x, y, vx, vy, life, max: life, kind: Kind::Dot, color: TEAL, grow: 0.0 });
    }

    /// Draw everything, offset by the playfield's corner.
    pub fn draw(&self, a: &Atlas, ox: i32, oy: i32) {
        for p in &self.list {
            let k = p.life as f32 / p.max as f32;
            let (x, y) = (ox + p.x.round() as i32, oy + p.y.round() as i32);
            match p.kind {
                Kind::Dot => a.fill(x, y, 1, 1, Color { a: p.color.a * k, ..p.color }),
                Kind::Spark => {
                    // a 2-3 px streak back along the velocity
                    let n = 3;
                    for s in 0..n {
                        let t = s as f32 * 0.6;
                        let (sx, sy) = (p.x - p.vx * t, p.y - p.vy * t);
                        let c = Color { a: p.color.a * k * (1.0 - s as f32 / n as f32), ..p.color };
                        a.fill(ox + sx.round() as i32, oy + sy.round() as i32, 1, 1, c);
                    }
                }
                Kind::Ring => {
                    let r = (p.max - p.life) as f32 * p.grow + 2.0;
                    let dots = (r * 1.2).clamp(8.0, 48.0) as i32;
                    for d in 0..dots {
                        let ang = d as f32 / dots as f32 * std::f32::consts::TAU;
                        let c = Color { a: 0.7 * k, ..p.color };
                        a.fill(ox + (p.x + ang.cos() * r).round() as i32, oy + (p.y + ang.sin() * r).round() as i32, 1, 1, c);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particles_die_and_the_pool_is_capped() {
        let mut p = Particles::new();
        for _ in 0..400 {
            p.burst(100.0, 100.0, true);
        }
        assert_eq!(p.len(), MAX);
        for _ in 0..60 {
            p.tick();
        }
        assert_eq!(p.len(), 0);
    }
}
