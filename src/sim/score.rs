//! The score: kill points (doubled point-blank), shards from cancelled bullets,
//! grazes, the boss's phase grades and capture bonuses, the stage's bonuses and its
//! rank, and the extends. Arcade grades: S, A, B, C (and D for the stage).

use super::boss::Outcome;
use super::enemy::EnemyKind;
use crate::tune::*;

/// A grade, best first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    S,
    A,
    B,
    C,
    D,
}

impl Grade {
    pub fn letter(self) -> &'static str {
        match self {
            Grade::S => "S",
            Grade::A => "A",
            Grade::B => "B",
            Grade::C => "C",
            Grade::D => "D",
        }
    }
}

/// Where points came from (for the popups and the sounds).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Award {
    Kill,
    /// A kill point-blank: worth double.
    Blank,
    /// One of the derelict's weak points, or one of JAWSTHIC's fins.
    Part,
    /// Cancelled bullets, as shards flying to you.
    Shards,
    /// A boss phase ended with this grade (phase 1-4).
    Grade(u8, Grade),
    /// JAWSTHIC destroyed.
    BossKill,
    /// Grazes add up quietly.
    Graze,
}

/// How a run's stage ended: the tally's lines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tally {
    pub killed: u32,
    pub spawned: u32,
    /// JAWSTHIC destroyed (not lost to the hole).
    pub destroyed: bool,
    /// Its phases' grades (none for a phase you started past, in practice).
    pub grades: [Option<Grade>; 4],
    pub misses: u32,
    pub bombs: u32,
    pub grazes: u32,
    pub no_miss: u32,
    pub no_bomb: u32,
    /// Out of 100, and the rank it earns.
    pub rating: u32,
    pub rank: Grade,
    /// The score with the bonuses in.
    pub total: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Score {
    pub points: u64,
    /// Extra ships given so far.
    pub extends: u8,
    /// Each boss phase's grade, once it ends.
    pub grades: [Option<Grade>; 4],
    pub tally: Option<Tally>,
}

impl Score {
    /// Add points. Returns how many extends the new total earned.
    pub fn add(&mut self, n: u32) -> u8 {
        self.points += n as u64;
        let mut got = 0;
        while let Some(&at) = EXTENDS.get(self.extends as usize) {
            if self.points < at {
                break;
            }
            self.extends += 1;
            got += 1;
        }
        got
    }
}

/// Points for a kill `dist` px from the Pilgrim, and whether it was point-blank.
pub fn kill_points(kind: EnemyKind, dist: f32) -> (u32, bool) {
    let base = KILL_POINTS[kind as usize];
    let blank = dist <= POINT_BLANK.0;
    (if blank { base * POINT_BLANK.1 } else { base }, blank)
}

/// A boss phase's grade: S captured fast, A captured, B broken but not clean, C timed out.
pub fn phase_grade(outcome: Outcome, clean: bool, took: u32, limit: u32) -> Grade {
    match (outcome, clean) {
        (Outcome::TimedOut, _) => Grade::C,
        (Outcome::Broken, false) => Grade::B,
        (Outcome::Broken, true) if (took as f32) < limit as f32 * GRADE_S_TIME => Grade::S,
        (Outcome::Broken, true) => Grade::A,
    }
}

/// The bonus for phase `p` (1-4) ending with grade `g`.
pub fn phase_bonus(p: u8, g: Grade) -> u32 {
    let base = CAPTURE[(p as usize - 1).min(3)];
    match g {
        Grade::S => base * 2,
        Grade::A => base,
        Grade::B => base / 5,
        Grade::C | Grade::D => 0,
    }
}

/// The stage's rating out of 100.
pub fn rating(killed: u32, spawned: u32, destroyed: bool, grades: &[Option<Grade>; 4], misses: u32, bombs: u32) -> u32 {
    let kills = RATE_KILLS * killed.min(spawned) / spawned.max(1);
    let boss: u32 = grades.iter().flatten().map(|g| RATE_GRADES[(*g as usize).min(3)]).sum();
    let destroyed = if destroyed { RATE_DESTROYED } else { 0 };
    let no_miss = RATE_NO_MISS.0.saturating_sub(misses * RATE_NO_MISS.1);
    let no_bomb = RATE_NO_BOMB.0.saturating_sub(bombs * RATE_NO_BOMB.1);
    (kills + boss + destroyed + no_miss + no_bomb).min(100)
}

/// The rank a rating earns.
pub fn rank(rating: u32) -> Grade {
    [Grade::S, Grade::A, Grade::B, Grade::C].into_iter().zip(RANKS).find(|&(_, at)| rating >= at).map_or(Grade::D, |(g, _)| g)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kill_point_blank_scores_double() {
        let (far, b) = kill_points(EnemyKind::Gunship, 120.0);
        assert_eq!((far, b), (KILL_POINTS[EnemyKind::Gunship as usize], false));
        let (near, b) = kill_points(EnemyKind::Gunship, POINT_BLANK.0 - 1.0);
        assert_eq!((near, b), (far * POINT_BLANK.1, true));
        assert!(kill_points(EnemyKind::Hull, 300.0).0 > kill_points(EnemyKind::Heavy, 300.0).0, "bigger kills, bigger points");
        assert!(kill_points(EnemyKind::Heavy, 300.0).0 > kill_points(EnemyKind::Drone, 300.0).0);
    }

    #[test]
    fn phase_grades_reward_clean_fast_breaks() {
        let limit = 40 * TICK_HZ;
        assert_eq!(phase_grade(Outcome::Broken, true, limit / 3, limit), Grade::S);
        assert_eq!(phase_grade(Outcome::Broken, true, limit * 3 / 4, limit), Grade::A);
        assert_eq!(phase_grade(Outcome::Broken, false, limit / 3, limit), Grade::B, "a death or a bomb costs the capture");
        assert_eq!(phase_grade(Outcome::TimedOut, true, limit, limit), Grade::C);
        assert_eq!(phase_bonus(2, Grade::A), CAPTURE[1]);
        assert_eq!(phase_bonus(2, Grade::S), CAPTURE[1] * 2);
        assert_eq!(phase_bonus(2, Grade::B), CAPTURE[1] / 5);
        assert_eq!(phase_bonus(2, Grade::C), 0);
    }

    #[test]
    fn extends_come_once_each_at_their_scores() {
        let mut s = Score::default();
        assert_eq!(s.add((EXTENDS[0] - 1) as u32), 0);
        assert_eq!(s.add(1), 1, "the first at exactly its score");
        assert_eq!(s.add(10), 0, "and only once");
        assert_eq!(s.add((EXTENDS[1] * 2) as u32), 1, "a big jump past the second gives just it");
        assert_eq!(s.add(u32::MAX / 2), 0, "and there are only two");
        assert_eq!(s.extends, 2);
    }

    #[test]
    fn the_rank_follows_the_rating() {
        let all = |g: Grade| [Some(g); 4];
        let perfect = rating(100, 100, true, &all(Grade::S), 0, 0);
        assert_eq!(perfect, 100);
        assert_eq!(rank(perfect), Grade::S);
        assert_eq!(rank(rating(80, 100, true, &all(Grade::A), 0, 0)), Grade::A);
        // 18 for kills, 16 for the phases, 10 destroyed, 5 and 1 left after the misses and bombs
        let messy = rating(60, 100, true, &all(Grade::B), 2, 2);
        assert_eq!((messy, rank(messy)), (50, Grade::C));
        assert_eq!(rank(rating(20, 100, false, &all(Grade::C), 5, 6)), Grade::D);
        assert!(rating(90, 100, true, &all(Grade::S), 1, 0) < perfect, "a miss costs");
        assert!(RANKS.windows(2).all(|w| w[0] > w[1]));
    }
}
