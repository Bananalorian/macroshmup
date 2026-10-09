//! The fixed-step clock: turns display frames of any length into whole 60 Hz
//! simulation ticks. The screen renders every display frame; the game only
//! advances in ticks.

use crate::tune::{MAX_FRAME_DT, MAX_TICKS_PER_FRAME, TICK_HZ, VSYNC_SNAP};

pub const TICK: f64 = 1.0 / TICK_HZ as f64;

#[derive(Default)]
pub struct Clock {
    acc: f64,
}

impl Clock {
    pub fn new() -> Clock {
        Clock::default()
    }

    /// Feed in one display frame's length (seconds); returns how many ticks to run.
    pub fn advance(&mut self, dt: f64) -> u32 {
        let mut dt = if dt.is_finite() { dt.clamp(0.0, MAX_FRAME_DT) } else { 0.0 };
        // vsync jitter: a frame that's almost exactly k ticks long is k ticks long
        let k = (dt / TICK).round();
        if k >= 1.0 && (dt - k * TICK).abs() < VSYNC_SNAP {
            dt = k * TICK;
        }
        self.acc += dt;
        let n = ((self.acc + 1e-9) / TICK).floor();
        self.acc = (self.acc - n * TICK).max(0.0);
        let n = n as u32;
        if n > MAX_TICKS_PER_FRAME {
            // too far behind: run a few ticks and forget the rest rather than spiral
            self.acc = 0.0;
            MAX_TICKS_PER_FRAME
        } else {
            n
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(frames: usize, dt: impl Fn(usize) -> f64) -> Vec<u32> {
        let mut c = Clock::new();
        (0..frames).map(|i| c.advance(dt(i))).collect()
    }

    #[test]
    fn sixty_hz_screen_gives_one_tick_per_frame() {
        assert!(run(600, |_| TICK).iter().all(|&n| n == 1));
    }

    #[test]
    fn vsync_jitter_never_drops_or_doubles_a_tick() {
        // +-0.5 ms of jitter around 16.67 ms, the way real vsync timestamps wobble
        let ticks = run(600, |i| TICK + if i % 2 == 0 { 0.0005 } else { -0.0005 });
        assert!(ticks.iter().all(|&n| n == 1), "{ticks:?}");
    }

    #[test]
    fn faster_screens_keep_sixty_ticks_a_second() {
        for hz in [75.0, 120.0, 144.0, 165.0, 240.0] {
            let frames = hz as usize * 10;
            let total: u32 = run(frames, |_| 1.0 / hz).iter().sum();
            assert!((total as i64 - 600).abs() <= 1, "{hz} Hz gave {total} ticks in 10 s");
        }
    }

    #[test]
    fn thirty_hz_runs_two_ticks_a_frame() {
        assert!(run(300, |_| 2.0 * TICK).iter().all(|&n| n == 2));
    }

    #[test]
    fn a_stall_catches_up_a_little_then_drops_the_backlog() {
        let mut c = Clock::new();
        assert_eq!(c.advance(3.0), MAX_TICKS_PER_FRAME);
        assert_eq!(c.advance(TICK), 1, "no leftover backlog after a stall");
    }

    #[test]
    fn bad_frame_times_run_nothing() {
        let mut c = Clock::new();
        assert_eq!(c.advance(-1.0), 0);
        assert_eq!(c.advance(f64::NAN), 0);
        assert_eq!(c.advance(0.0), 0);
    }
}
