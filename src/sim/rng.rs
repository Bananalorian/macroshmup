//! A small seeded RNG (SplitMix64). The same seed always gives the same numbers on
//! every platform, which the bot tests and replays rely on. Gameplay and cosmetics
//! each own a separate one, so effects can never change gameplay.

#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform in [lo, hi).
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_numbers() {
        let (mut a, mut b) = (Rng::new(42), Rng::new(42));
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let (mut a, mut b) = (Rng::new(1), Rng::new(2));
        let same = (0..100).filter(|_| a.next_u64() == b.next_u64()).count();
        assert_eq!(same, 0);
    }

    #[test]
    fn known_sequence_is_stable() {
        // pinned so a refactor can't silently change every seeded run
        let mut r = Rng::new(0);
        assert_eq!(r.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(r.next_u64(), 0x6E78_9E6A_A1B9_65F4);
    }

    #[test]
    fn floats_stay_in_range_and_spread_out() {
        let mut r = Rng::new(7);
        let mut buckets = [0u32; 10];
        for _ in 0..100_000 {
            let x = r.f32();
            assert!((0.0..1.0).contains(&x));
            buckets[(x * 10.0) as usize] += 1;
            let y = r.range(-3.0, 5.0);
            assert!((-3.0..5.0).contains(&y));
        }
        assert!(buckets.iter().all(|&n| (9_000..11_000).contains(&n)), "{buckets:?}");
    }
}
