//! The explosion kit: small, medium, large and boss, each much bigger than what
//! died. A blast throws debris (particles.rs), sends out a shock ring, spikes the
//! chromatic aberration (more the closer it is to the Pilgrim) and, for the big
//! ones, flashes the screen (limited to 3 a second, and never with Reduce flashing).
//! Sounds come from the matching `synth` blast.

use super::particles::Particles;
use super::post::Fx;
use crate::tune::{BLASTS, Blast, CA_RANGE, REDUCED_CA};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    Small,
    Medium,
    Large,
    Boss,
}

impl Size {
    pub const ALL: [Size; 4] = [Size::Small, Size::Medium, Size::Large, Size::Boss];

    pub fn spec(self) -> &'static Blast {
        &BLASTS[self as usize]
    }

    pub fn label(self) -> &'static str {
        match self {
            Size::Small => "SMALL",
            Size::Medium => "MEDIUM",
            Size::Large => "LARGE",
            Size::Boss => "BOSS",
        }
    }
}

/// Set off a blast at playfield `(x, y)`. `player` is the Pilgrim's position (the
/// aberration is strongest up close).
pub fn explode(fx: &mut Fx, parts: &mut Particles, x: f32, y: f32, size: Size, player: (f32, f32), reduce_flash: bool, frame: u64) {
    let b = size.spec();
    parts.blast(x, y, b);
    fx.shock(x, y, b.shock.0, b.shock.1, b.shock.2);
    fx.aberrate(aberration(b, (x - player.0).hypot(y - player.1), reduce_flash));
    if b.flash && !reduce_flash {
        fx.flash.trigger(frame);
    }
}

/// How hard a blast `dist` px from the Pilgrim spikes the aberration.
pub fn aberration(b: &Blast, dist: f32, reduce_flash: bool) -> f32 {
    let near = (1.0 - dist / CA_RANGE).clamp(0.25, 1.0);
    b.ca * near * if reduce_flash { REDUCED_CA } else { 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bigger_blasts_hit_harder() {
        for w in BLASTS.windows(2) {
            assert!(w[1].shock.0 > w[0].shock.0 && w[1].ca > w[0].ca && w[1].sparks > w[0].sparks);
        }
    }

    #[test]
    fn closer_blasts_aberrate_more_and_reduce_flashing_tames_them() {
        let b = Size::Large.spec();
        assert!(aberration(b, 10.0, false) > aberration(b, 200.0, false));
        assert!(aberration(b, 10.0, true) < aberration(b, 10.0, false) * 0.5);
        assert!(aberration(b, 1e6, false) > 0.0, "even far away it's felt a little");
    }

    #[test]
    fn a_blast_rings_and_flashes_unless_flashing_is_reduced() {
        let (mut fx, mut parts) = (Fx::new(), Particles::new());
        explode(&mut fx, &mut parts, 100.0, 100.0, Size::Large, (100.0, 200.0), false, 10);
        assert_eq!(fx.shocks.len(), 1);
        assert!(fx.ca > 0.0 && fx.flash.level() > 0.0 && parts.len() > 0);
        let (mut fx, mut parts) = (Fx::new(), Particles::new());
        explode(&mut fx, &mut parts, 100.0, 100.0, Size::Large, (100.0, 200.0), true, 10);
        assert_eq!(fx.flash.level(), 0.0, "no flash with Reduce flashing");
        let (mut fx, mut parts) = (Fx::new(), Particles::new());
        explode(&mut fx, &mut parts, 100.0, 100.0, Size::Small, (100.0, 200.0), false, 10);
        assert_eq!(fx.flash.level(), 0.0, "small ones never flash");
    }
}
