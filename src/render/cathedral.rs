//! The Cathedral (Sprint 6 *proposal*): a gothic hulk guarding the event horizon,
//! in the same bone white as Stage 1's enemies. A nave with a spire and a rose window
//! (its red core at the heart), twin towers, flying buttresses out to cratered
//! chapels, and red stained glass that blazes as it charges each attack. For now it
//! only appears in the mock-up (PG_MOCKUP=1); it joins the game if Jason approves it.

use super::foes::{CORE, CRATER, GRILLE, LIGHT, P, PLATE, RECESSED, SIDE, SLOT, STAINED, bone, poly, raster};
use macroquad::prelude::*;
use std::f32::consts::TAU;

pub const SIZE: (i32, i32) = (191, 141);
/// The rose window's center, from the sprite's center.
pub const ROSE: (f32, f32) = (0.0, -14.0);
/// The wing turrets (phase 2's weak points), from the sprite's center.
pub const TURRETS: [(f32, f32); 2] = [(-78.0, 40.0), (78.0, 40.0)];

fn shapes() -> Vec<P> {
    let (rx, ry) = ROSE;
    let mut v = vec![
        // the chapels out on the wings, crenellated, cratered
        poly(&[(40.0, -26.0), (62.0, -30.0), (84.0, -26.0), (95.0, -14.0), (95.0, 18.0), (86.0, 28.0), (64.0, 30.0), (44.0, 24.0), (40.0, 10.0)], PLATE, true),
        poly(&[(44.0, 24.0), (64.0, 30.0), (86.0, 28.0), (95.0, 18.0), (95.0, 22.0), (86.0, 32.0), (64.0, 34.0), (44.0, 28.0)], SIDE, true),
        // buttresses down the chapels' faces, pinnacles on their roofline
        P::Rect(55.0, -28.0, 57.0, 28.0, SIDE, true),
        P::Rect(71.0, -29.0, 73.0, 30.0, SIDE, true),
        P::Rect(86.0, -25.0, 88.0, 27.0, SIDE, true),
        poly(&[(54.6, -28.0), (56.0, -36.0), (57.4, -28.0)], PLATE, true),
        poly(&[(70.6, -29.0), (72.0, -38.0), (73.4, -29.0)], PLATE, true),
        poly(&[(85.6, -25.0), (87.0, -32.0), (88.4, -25.0)], PLATE, true),
        // lancet windows between the buttresses
        poly(&[(64.0, -20.0), (66.0, -16.0), (66.0, 8.0), (62.0, 8.0), (62.0, -16.0)], STAINED, true),
        poly(&[(80.0, -18.0), (82.0, -14.0), (82.0, 8.0), (78.0, 8.0), (78.0, -14.0)], STAINED, true),
        poly(&[(48.0, -14.0), (50.0, -10.0), (50.0, 6.0), (46.0, 6.0), (46.0, -10.0)], STAINED, true),
        P::Rect(60.0, 14.0, 68.0, 16.0, GRILLE, true),
        P::Disc(74.0, 14.0, 2.2, CRATER, false),
        P::Disc(88.0, -6.0, 1.6, CRATER, false),
        P::Disc(52.0, 16.0, 1.4, CRATER, false),
        P::Disc(-60.0, 12.0, 2.4, CRATER, false),
        P::Disc(-84.0, -2.0, 1.8, CRATER, false),
        P::Disc(-70.0, -20.0, 1.2, CRATER, false),
        // flying buttresses from the towers to the chapels
        poly(&[(30.0, -6.0), (42.0, -16.0), (45.0, -12.0), (33.0, -2.0)], PLATE, true),
        poly(&[(30.0, 14.0), (42.0, 4.0), (45.0, 8.0), (33.0, 18.0)], PLATE, true),
        // the twin towers, spired, with pinnacles at their shoulders
        poly(&[(24.0, -46.0), (29.0, -54.0), (34.0, -67.0), (39.0, -54.0), (44.0, -46.0), (44.0, 36.0), (24.0, 36.0)], PLATE, true),
        poly(&[(24.0, -46.0), (25.0, -53.0), (27.0, -46.0)], PLATE, true),
        poly(&[(41.0, -46.0), (43.0, -53.0), (44.0, -46.0)], PLATE, true),
        P::Rect(24.0, 30.0, 44.0, 36.0, SIDE, true),
        poly(&[(34.0, -40.0), (37.0, -35.0), (37.0, -12.0), (31.0, -12.0), (31.0, -35.0)], STAINED, true),
        P::Disc(34.0, -48.0, 2.2, STAINED, true),
        P::Rect(31.0, -4.0, 37.0, 22.0, RECESSED, true),
        P::Rect(33.0, 0.0, 35.0, 20.0, STAINED, true),
        // the nave: a gable rising to the spire, a pointed portal at its foot
        poly(&[(0.0, -70.0), (4.0, -62.0), (8.0, -54.0), (10.0, -50.0), (20.0, -44.0), (22.0, -40.0), (22.0, 50.0), (14.0, 58.0), (0.0, 62.0)], PLATE, true),
        P::Rect(-2.0, -66.0, 2.0, -66.0, PLATE, false),
        poly(&[(10.0, -50.0), (12.0, -57.0), (14.0, -48.0)], PLATE, true),
        P::Disc(0.0, -41.0, 3.0, STAINED, false),
        P::Rect(-21.0, -32.0, 21.0, -32.0, STAINED, false),
        // the rose window: a stained-glass wheel round the red core
        P::Disc(rx, ry, 17.4, SIDE, false),
        P::Ring(rx, ry, 15.2, 17.4, PLATE, false),
        P::Disc(rx, ry, 15.2, STAINED, false),
        P::Ring(rx, ry, 10.0, 11.0, PLATE, false),
        P::Ring(rx, ry, 5.0, 6.4, PLATE, false),
        P::Disc(rx, ry, 3.6, CORE, false),
        // under it: vents, twin lancets, the portal with the main gun
        P::Rect(-18.0, 4.0, 18.0, 6.0, GRILLE, false),
        poly(&[(8.0, 12.0), (11.0, 17.0), (11.0, 36.0), (5.0, 36.0), (5.0, 17.0)], STAINED, true),
        P::Rect(-21.0, 40.0, 21.0, 40.0, STAINED, false),
        poly(&[(0.0, 43.0), (6.0, 47.0), (9.0, 52.0), (9.0, 60.0), (0.0, 60.0)], RECESSED, true),
        P::Rect(0.0, 54.0, 0.0, 56.0, LIGHT, false),
        P::Rect(-3.0, 59.0, 3.0, 60.0, SLOT, false),
    ];
    // the rose window's tracery: eight spokes
    for k in 0..8 {
        let a = k as f32 * TAU / 8.0 + TAU / 16.0;
        let (c, s) = (a.cos(), a.sin());
        let (px, py) = (-s * 0.7, c * 0.7);
        let (r0, r1) = (6.0, 15.4);
        v.push(P::Poly(vec![(rx + c * r0 + px, ry + s * r0 + py), (rx + c * r1 + px, ry + s * r1 + py), (rx + c * r1 - px, ry + s * r1 - py), (rx + c * r0 - px, ry + s * r0 - py)], PLATE, false));
    }
    v
}

/// The Cathedral's sprite.
pub fn image() -> Image {
    let (w, h) = SIZE;
    bone(w, h, &raster(w, h, &shapes()), 23, (0.0, 0.0))
}

/// Every stained-glass pixel (relative to the center): they blaze red as it charges.
pub fn glass() -> Vec<(i32, i32)> {
    let (w, h) = SIZE;
    let m = raster(w, h, &shapes());
    let (cx, cy) = (w / 2, h / 2);
    let mut v = vec![];
    for y in 0..h {
        for x in 0..w {
            if m[(y * w + x) as usize] == STAINED {
                v.push((x - cx, y - cy));
            }
        }
    }
    v
}

/// A wing turret: a spired pinnacle with a red core, its gun beneath.
pub fn turret() -> Image {
    let shapes = vec![
        poly(&[(0.0, -13.0), (3.0, -8.0), (6.0, -6.0), (6.0, 8.0), (3.0, 12.0), (0.0, 13.0)], PLATE, true),
        poly(&[(0.0, 9.0), (6.0, 6.0), (3.0, 12.0), (0.0, 13.0)], SIDE, true),
        poly(&[(0.0, -4.0), (2.0, -1.0), (2.0, 6.0), (0.0, 6.0)], STAINED, true),
        P::Disc(0.0, 1.0, 1.4, CORE, false),
        P::Rect(0.0, 11.0, 0.0, 13.0, SLOT, false),
    ];
    let m = raster(13, 27, &shapes);
    bone(13, 27, &m, 29, (0.0, 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::foes::BONE;
    use crate::render::palette::{CORE as BULLET_CORE, luma};

    #[test]
    fn the_cathedral_is_bone_white_with_red_glass_and_fits_the_playfield() {
        let im = image();
        let lumas: Vec<f32> = im.get_image_data().iter().filter(|p| p[3] > 0).map(|p| luma(Color::from_rgba(p[0], p[1], p[2], 255))).collect();
        let brightest = lumas.iter().cloned().fold(0.0, f32::max);
        assert!(brightest < luma(BULLET_CORE) - 0.08 && brightest >= luma(BONE[4]) - 0.01);
        assert!(glass().len() > 300, "plenty of stained glass to blaze: {}", glass().len());
        assert!(SIZE.0 < crate::tune::FIELD_W as i32 - 60, "room to dodge either side");
    }
}
