//! The style rank, Devil May Cry's in a shmup: playing with variety fills a meter that
//! drains all the time, climbing D, C, B, A, S, SS, SSS (100 points a rank), and the
//! rank multiplies the points you earn. A move you keep repeating earns less and less
//! until it cools (a few seconds); a hit drops you two ranks, a bomb one, a death to D.

use crate::tune::*;

/// What earns style. Each cools on its own, so mixing them up pays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    /// A kill with the wide shot, with the beam (or its aura), point-blank, or by a slam
    /// (or during a chain's slowdown).
    Kill,
    BeamKill,
    Blank,
    SlamKill,
    /// A slam's strike.
    Slam,
    Graze,
    /// Steady damage to the boss, with shots and with the beam.
    BossHit,
    BossBeam,
    /// A weak point, a fin, or a boss phase broken.
    Break,
}

pub const MOVES: usize = 9;
/// The ranks' letters, D to SSS.
pub const RANKS: [&str; 7] = ["D", "C", "B", "A", "S", "SS", "SSS"];

#[derive(Clone, Debug, Default)]
pub struct Style {
    /// The meter: 100 a rank, D at 0, up to the top of SSS.
    pub points: f32,
    /// How stale each move is: +1 each use, cooling over `STYLE_COOL` frames a use.
    heat: [f32; MOVES],
    /// Damage to the boss not yet turned into style (shots, the beam).
    boss_dmg: [f32; 2],
    /// Frames since the rank last changed, and whether it rose.
    pub since: u32,
    pub rose: bool,
    /// The best rank this run, and frames spent at each (for the balance report).
    pub peak: u8,
    pub frames: [u64; 7],
}

impl Style {
    pub fn rank(&self) -> u8 {
        ((self.points / 100.0) as u8).min(6)
    }

    /// How far through its rank the meter is, 0..1.
    pub fn fill(&self) -> f32 {
        ((self.points - self.rank() as f32 * 100.0) / 100.0).clamp(0.0, 1.0)
    }

    /// What the rank multiplies points by.
    pub fn mult(&self) -> f32 {
        STYLE_MULT[self.rank() as usize]
    }

    /// Move the meter to `points`; the new rank if it changed.
    fn set(&mut self, points: f32) -> Option<u8> {
        let was = self.rank();
        self.points = points.clamp(0.0, 700.0 - 0.01);
        let now = self.rank();
        self.peak = self.peak.max(now);
        (now != was).then(|| {
            (self.since, self.rose) = (0, now > was);
            now
        })
    }

    /// A move worth `base` points, less the more it's been repeated lately.
    pub fn gain(&mut self, m: Move, base: f32) -> Option<u8> {
        let h = &mut self.heat[m as usize];
        let fresh = 1.0 / (1.0 + *h * STYLE_STALE);
        *h += 1.0;
        self.set(self.points + base * fresh)
    }

    /// Damage to the boss: every `STYLE_BOSS.0` of it is a `BossHit` (or, by the
    /// beam, a `BossBeam`).
    pub fn boss_damage(&mut self, dmg: f32, beam: bool) -> Option<u8> {
        let k = beam as usize;
        self.boss_dmg[k] += dmg;
        let mut out = None;
        while self.boss_dmg[k] >= STYLE_BOSS.0 {
            self.boss_dmg[k] -= STYLE_BOSS.0;
            out = self.gain(if beam { Move::BossBeam } else { Move::BossHit }, STYLE_BOSS.1).or(out);
        }
        out
    }

    /// Down `n` ranks, to the bottom of the one it lands on.
    pub fn drop(&mut self, n: u8) -> Option<u8> {
        self.set(self.rank().saturating_sub(n) as f32 * 100.0)
    }

    /// Back to D (a death).
    pub fn reset(&mut self) -> Option<u8> {
        self.set(0.0)
    }

    /// A frame: the meter drains (faster the higher it is) and every move cools.
    pub fn tick(&mut self) -> Option<u8> {
        for h in &mut self.heat {
            *h = (*h - 1.0 / STYLE_COOL as f32).max(0.0);
        }
        self.since = self.since.saturating_add(1);
        self.frames[self.rank() as usize] += 1;
        self.set(self.points - STYLE_DRAIN[self.rank() as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Moves raise the meter through the ranks, and the rank multiplies points more
    /// the higher it is.
    #[test]
    fn moves_climb_the_ranks_and_the_rank_multiplies_points() {
        let mut s = Style::default();
        assert_eq!((s.rank(), s.mult()), (0, 1.0));
        let moves = [Move::Kill, Move::Graze, Move::BeamKill, Move::Blank, Move::Slam, Move::SlamKill, Move::Break, Move::BossHit, Move::BossBeam];
        let mut rose = vec![];
        for k in 0..64 {
            if let Some(r) = s.gain(moves[k % moves.len()], 40.0) {
                rose.push(r);
            }
        }
        assert_eq!(rose, vec![1, 2, 3, 4, 5, 6], "D to SSS a rank at a time");
        assert_eq!(s.rank(), 6);
        assert!(STYLE_MULT.windows(2).all(|w| w[1] > w[0]) && STYLE_MULT[0] == 1.0);
    }

    /// Doing the same thing over and over earns less and less; mixing it up doesn't.
    /// A move that's had time to cool pays in full again.
    #[test]
    fn repeating_yourself_earns_less_until_it_cools() {
        let mut same = Style::default();
        let mut mixed = Style::default();
        let moves = [Move::Kill, Move::Graze, Move::BeamKill, Move::Blank];
        for k in 0..8 {
            same.gain(Move::Kill, 20.0);
            mixed.gain(moves[k % 4], 20.0);
        }
        assert!(mixed.points > same.points * 1.6, "mixed {} vs the same move {}", mixed.points, same.points);
        let before = same.points;
        same.gain(Move::Kill, 20.0);
        let stale = same.points - before;
        for _ in 0..STYLE_COOL * 10 {
            same.heat.iter_mut().for_each(|h| *h = (*h - 1.0 / STYLE_COOL as f32).max(0.0));
        }
        let before = same.points;
        same.gain(Move::Kill, 20.0);
        assert!(stale < 20.0 * 0.3 && (same.points - before - 20.0).abs() < 1e-3, "stale {stale}, cooled {}", same.points - before);
    }

    /// Left alone the meter drains, faster the higher the rank, down to D.
    #[test]
    fn it_drains_faster_the_higher_it_is() {
        assert!(STYLE_DRAIN.windows(2).all(|w| w[1] >= w[0]));
        let mut s = Style::default();
        s.set(650.0);
        let mut frames_at = [0u32; 7];
        while s.points > 0.0 {
            frames_at[s.rank() as usize] += 1;
            s.tick();
        }
        assert!(frames_at[6] < frames_at[1], "SSS slips away faster than C: {frames_at:?}");
        assert_eq!(s.rank(), 0);
    }

    /// A hit drops two ranks, a bomb one, a death to D.
    #[test]
    fn hits_bombs_and_deaths_cost_ranks() {
        let mut s = Style::default();
        s.set(560.0);
        assert_eq!(s.drop(2), Some(3), "a hit: SS to A");
        assert!(!s.rose && s.since == 0);
        assert_eq!(s.drop(1), Some(2), "a bomb: A to B");
        assert_eq!(s.reset(), Some(0));
        assert_eq!(s.drop(2), None, "nothing below D");
    }
}
