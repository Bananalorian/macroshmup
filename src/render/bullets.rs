//! Enemy bullets and the guns that fire them. Bullets draw above everything but
//! the HUD, big and slow ones under small and fast ones, and their white cores
//! are the brightest pixels on screen.

use super::art::{Atlas, TURNS};
use super::palette::{SIGNAL, WHITE_A, hue_rims};
use crate::sim::World;
use crate::sim::bullets::{Behavior, Kind};
use crate::sim::pattern::Mount;
use crate::tune::{BLOOM_FRAMES, MUZZLE_FRAMES, STAR_SPIN};
use macroquad::color::Color;

/// Which pre-drawn turn suits a direction (rice and needles look the same turned 180 degrees).
pub fn turn(dx: f32, dy: f32) -> usize {
    let a = dy.atan2(dx).rem_euclid(std::f32::consts::PI);
    (a / (std::f32::consts::PI / TURNS as f32)).round() as usize % TURNS
}

/// The guns: turrets where they're fixed or sweeping, and a red flash when they fire (the telegraph).
pub fn draw_guns(a: &Atlas, w: &World, ox: i32, oy: i32) {
    for e in &w.hazards.emitters {
        if !e.live {
            continue;
        }
        let (x, y) = (ox + e.x.round() as i32, oy + e.y.round() as i32);
        if !matches!(e.gun.mount, Mount::Dummy(_)) {
            let t = a.r.bullets.turret;
            a.blit(t, x - 5, y - 5, 1, WHITE_A);
        }
        if e.since < MUZZLE_FRAMES {
            let k = 1.0 - e.since as f32 / MUZZLE_FRAMES as f32;
            let c = Color { a: k, ..SIGNAL };
            a.fill(x - 1, y - 1, 3, 3, c);
            a.fill(x - 3, y, 7, 1, Color { a: k * 0.6, ..SIGNAL });
            a.fill(x, y - 3, 1, 7, Color { a: k * 0.6, ..SIGNAL });
        }
    }
}

pub fn draw_bullets(a: &Atlas, w: &World, ox: i32, oy: i32) {
    let b = &w.hazards.bullets;
    let art = &a.r.bullets;
    for kind in Kind::DRAW_ORDER {
        let k = kind as usize;
        for i in 0..b.len() {
            if b.kind[i] != kind {
                continue;
            }
            let (x, y) = (b.x[i], b.y[i]);
            let age = b.age[i];
            let (rim, _) = hue_rims(b.hue[i]);
            // odd trajectories leave a short trail
            if matches!(b.behavior[i], Behavior::Curve { .. } | Behavior::Pull { .. }) && age > 6 {
                for s in 1..4 {
                    let d = s as f32 * 3.0 + kind.half_len();
                    let c = Color { a: 0.5 / s as f32, ..rim };
                    a.fill(ox + (x - b.dx[i] * d).round() as i32, oy + (y - b.dy[i] * d).round() as i32, 1, 1, c);
                }
            }
            let wob = ((age / 4) as usize + k) % 2;
            // stars turn as they drift (each from the way it was fired, so a ring of
            // them doesn't spin in step); everything else points where it flies
            let t = turn(b.dx[i], b.dy[i]);
            let t = if kind == Kind::Star { (t + (age / STAR_SPIN) as usize) % TURNS } else { t };
            let src = art.frames[b.hue[i] as usize][k][wob][t];
            let (sx, sy) = (ox + (x - src.w * 0.5).round() as i32, oy + (y - src.h * 0.5).round() as i32);
            if age < BLOOM_FRAMES {
                // the spawn bloom: harmless, and it looks it
                let ring = art.bloom[age as usize];
                let (rx, ry) = (ox + (x - ring.w * 0.5).round() as i32, oy + (y - ring.h * 0.5).round() as i32);
                a.blit(ring, rx, ry, 1, Color { a: 0.7 - age as f32 * 0.15, ..rim });
                a.blit(src, sx, sy, 1, Color { a: 0.35 + age as f32 * 0.15, ..WHITE_A });
            } else {
                a.blit(src, sx, sy, 1, WHITE_A);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_cover_every_direction_both_ways() {
        assert_eq!(turn(1.0, 0.0), 0);
        assert_eq!(turn(-1.0, 0.0), 0, "the same turned around");
        assert_eq!(turn(0.0, 1.0), TURNS / 2);
        assert_eq!(turn(0.0, -1.0), TURNS / 2);
        let a = std::f32::consts::PI / TURNS as f32;
        assert_eq!(turn(a.cos(), a.sin()), 1);
        assert!((0..360).all(|d| turn((d as f32).to_radians().cos(), (d as f32).to_radians().sin()) < TURNS));
    }
}
