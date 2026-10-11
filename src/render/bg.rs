//! The background: a three-layer parallax starfield, falling past the Pilgrim as it
//! flies up. Purely cosmetic: its own seed, never the gameplay RNG. Star positions
//! are a pure function of the frame, so screenshots and replays always match.

use super::art::Atlas;
use super::layout::IRect;
use super::palette::{SIGNAL, TEAL, luma, tone};
use crate::sim::rng::Rng;
use crate::tune::{BG_MAX_VALUE, STAR_LAYERS, STAR_TINT_SHARE, STAR_TWINKLE};
use macroquad::color::Color;

/// A star's color: the warm monochrome, or the odd red or teal one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tint {
    Plain,
    Red,
    Teal,
}

/// A star's color at `value` brightness (the same luma whatever its tint).
pub fn star_color(tint: Tint, value: f32) -> Color {
    let pure = match tint {
        Tint::Plain => return tone(value),
        Tint::Red => SIGNAL,
        Tint::Teal => TEAL,
    };
    let k = value / luma(pure);
    Color::new(pure.r * k, pure.g * k, pure.b * k, 1.0)
}

/// The cosmetic seed for the starfield. Fixed, so the sky is the same every run.
const SEED: u64 = 0x5747_4152_5321;

#[derive(Clone, Copy, Debug)]
struct Star {
    /// Position across and down the area, 0..1.
    u: f32,
    v: f32,
    layer: u8,
    plus: bool,
    tint: Tint,
    /// Twinkle phase and rate.
    phase: f32,
    rate: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drawn {
    pub x: i32,
    pub y: i32,
    pub layer: usize,
    pub plus: bool,
    pub tint: Tint,
    /// Brightness, 0..1 of white.
    pub value: f32,
}

pub struct Starfield {
    /// Stars for an area of `area` px², by layer, far to near.
    stars: Vec<Star>,
    area: f32,
    per_layer: [usize; STAR_LAYERS.len()],
}

impl Starfield {
    /// Enough stars for the biggest area we fill (the whole landscape canvas).
    pub fn new(area: f32) -> Starfield {
        let mut rng = Rng::new(SEED);
        let mut stars = Vec::new();
        for (layer, l) in STAR_LAYERS.iter().enumerate() {
            let n = (l.density * area / 10_000.0).round() as usize;
            for _ in 0..n {
                stars.push(Star {
                    u: rng.f32(),
                    v: rng.f32(),
                    layer: layer as u8,
                    plus: rng.chance(l.plus_share),
                    // the odd red or teal star, among the nearer layers
                    tint: if layer >= 1 && rng.chance(STAR_TINT_SHARE) {
                        if rng.chance(0.5) { Tint::Red } else { Tint::Teal }
                    } else {
                        Tint::Plain
                    },
                    phase: rng.range(0.0, std::f32::consts::TAU),
                    rate: rng.range(0.02, 0.06),
                });
            }
        }
        let mut per_layer = [0; STAR_LAYERS.len()];
        for s in &stars {
            per_layer[s.layer as usize] += 1;
        }
        Starfield { stars, area, per_layer }
    }

    /// Where every star in `rect` is at `frame`, snapped to whole pixels.
    pub fn at(&self, rect: IRect, frame: u64) -> impl Iterator<Item = Drawn> + '_ {
        // a smaller area shows a matching share of the stars, so density stays put
        let share = ((rect.w * rect.h) as f32 / self.area).min(1.0);
        let mut seen = [0usize; STAR_LAYERS.len()];
        self.stars.iter().filter_map(move |s| {
            let l = s.layer as usize;
            seen[l] += 1;
            if seen[l] as f32 > self.per_layer[l] as f32 * share {
                return None;
            }
            let layer = &STAR_LAYERS[l];
            let h = rect.h as f32;
            let fall = (s.v * h + frame as f32 * layer.speed).rem_euclid(h);
            let tw = 0.5 + 0.5 * (s.phase + frame as f32 * s.rate).sin();
            Some(Drawn {
                x: rect.x + (s.u * rect.w as f32) as i32,
                y: rect.y + fall as i32,
                layer: l,
                plus: s.plus,
                tint: s.tint,
                value: (layer.value * (1.0 - STAR_TWINKLE * tw)).min(BG_MAX_VALUE),
            })
        })
    }

    /// Draw the stars; `dim` scales their brightness (1 is full).
    pub fn draw(&self, atlas: &Atlas, rect: IRect, frame: u64, dim: f32) {
        for d in self.at(rect, frame) {
            let c = star_color(d.tint, d.value * dim);
            if d.plus {
                atlas.blit(atlas.r.plus, d.x - 1, d.y - 1, 1, c);
            } else {
                atlas.fill(d.x, d.y, 1, 1, c);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::palette::luma;

    const FIELD: IRect = IRect::new(185, 0, 270, 360);

    fn field() -> Starfield {
        Starfield::new(640.0 * 480.0)
    }

    #[test]
    fn stars_stay_inside_their_rect() {
        let s = field();
        for frame in (0..20_000).step_by(97) {
            for d in s.at(FIELD, frame) {
                assert!(d.x >= FIELD.x && d.x < FIELD.right(), "{d:?}");
                assert!(d.y >= FIELD.y && d.y < FIELD.bottom(), "{d:?}");
            }
        }
    }

    #[test]
    fn three_layers_far_to_near_get_faster_and_brighter() {
        assert_eq!(STAR_LAYERS.len(), 3);
        for w in STAR_LAYERS.windows(2) {
            assert!(w[1].speed > w[0].speed);
            assert!(w[1].value > w[0].value);
            assert!(w[1].density < w[0].density);
        }
        let s = field();
        for l in 0..3 {
            assert!(s.at(FIELD, 0).filter(|d| d.layer == l).count() >= 5, "layer {l} is empty");
        }
    }

    #[test]
    fn the_sky_is_the_same_every_time() {
        let (a, b) = (field(), field());
        assert!(a.at(FIELD, 1234).eq(b.at(FIELD, 1234)));
    }

    #[test]
    fn stars_fall_at_their_layer_speed() {
        let s = field();
        let (a, b): (Vec<_>, Vec<_>) = (s.at(FIELD, 0).collect(), s.at(FIELD, 600).collect());
        for (p, q) in a.iter().zip(&b) {
            let fell = (q.y - p.y).rem_euclid(FIELD.h) as f32;
            let want = (STAR_LAYERS[p.layer].speed * 600.0).rem_euclid(FIELD.h as f32);
            assert!((fell - want).abs() <= 1.0, "layer {} fell {fell}, want {want}", p.layer);
        }
    }

    /// The value budget: the background never goes past about 30% brightness.
    #[test]
    fn the_background_stays_inside_the_value_budget() {
        let s = field();
        for frame in (0..3_000).step_by(7) {
            for d in s.at(IRect::new(0, 0, 640, 360), frame) {
                assert!(luma(tone(d.value)) <= BG_MAX_VALUE + 0.005, "{d:?}");
            }
        }
    }

    #[test]
    fn the_odd_star_is_red_or_teal_and_still_dim() {
        let s = field();
        let all: Vec<Drawn> = s.at(IRect::new(0, 0, 640, 480), 0).collect();
        let red = all.iter().filter(|d| d.tint == Tint::Red).count();
        let teal = all.iter().filter(|d| d.tint == Tint::Teal).count();
        assert!(red >= 1 && teal >= 1, "red {red}, teal {teal}");
        assert!((red + teal) * 10 < all.len(), "only the odd one");
        for t in [Tint::Plain, Tint::Red, Tint::Teal] {
            assert!((luma(star_color(t, BG_MAX_VALUE)) - BG_MAX_VALUE).abs() < 0.01, "{t:?} keeps the value budget");
        }
    }

    #[test]
    fn density_is_the_same_in_a_smaller_rect() {
        let s = field();
        let big = s.at(IRect::new(0, 0, 640, 480), 0).count() as f32 / (640.0 * 480.0);
        let small = s.at(FIELD, 0).count() as f32 / (270.0 * 360.0);
        assert!((big / small - 1.0).abs() < 0.1, "{big} vs {small}");
    }
}
