//! The simulation: pure, deterministic Rust. Nothing in `sim/` draws, plays audio
//! or touches macroquad, so `cargo test` can play the game headlessly in CI.
//! The same seed with the same inputs always gives the same run.

pub mod rng;

/// One run of the game. For now the playfield is empty; the Pilgrim arrives in
/// Sprint 1, and the gameplay RNG (seeded from `seed`) with the first bullets.
pub struct World {
    pub seed: u64,
    /// Ticks since the run began. Every gameplay timing counts these.
    pub frame: u64,
}

impl World {
    pub fn new(seed: u64) -> World {
        World { seed, frame: 0 }
    }

    /// Advance the game by exactly one 60 Hz tick.
    pub fn tick(&mut self) {
        self.frame += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tick_is_one_frame() {
        let mut w = World::new(3);
        for _ in 0..600 {
            w.tick();
        }
        assert_eq!(w.frame, 600);
        assert_eq!(w.seed, 3);
    }

    /// Non-negotiable 2: the sim never draws, plays audio or touches macroquad.
    #[test]
    fn the_sim_never_touches_macroquad() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sim");
        let mut checked = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "rs") {
                let src = std::fs::read_to_string(&path).unwrap();
                // the game code only: skip comments and the tests (this one names what it bans)
                let code: String = src
                    .split("#[cfg(test)]")
                    .next()
                    .unwrap()
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("//"))
                    .collect();
                for banned in ["macroquad", "miniquad", "crate::render", "crate::audio"] {
                    assert!(!code.contains(banned), "{} uses {banned}", path.display());
                }
                checked += 1;
            }
        }
        assert!(checked >= 2);
    }
}
