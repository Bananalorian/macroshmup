//! The Pilgrim's kit (Sprint 12): two augments at its sides and a shield, earned by
//! style (Jason's call). The first time a life reaches B, the left augment comes
//! online; A, the right one; S, the shield. A life keeps what it earns, and loses it
//! all with the ship.
//!
//! The augments are engine pods like the ship's own. They follow it, trailing a
//! little (the ship never drifts, but they may), spread wide for the wide shot and
//! tucked in beside the sword for the beam, and fire a bolt straight up every few
//! frames whenever you fire. The shield takes one hit (the bullets round you pop and
//! you blink a moment, untouchable), then recharges.

use super::shots::{Shot, ShotKind};
use crate::tune::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kit {
    /// What this life has earned: 0 nothing, 1 the left augment, 2 both, 3 the
    /// shield too.
    pub level: u8,
    /// Where each augment is (playfield px), left then right.
    pub aug: [(f32, f32); 2],
    /// Frames until the shield's back (0: it's up, once it's earned).
    pub recharge: u32,
    fire_cd: u32,
}

impl Default for Kit {
    fn default() -> Kit {
        Kit { level: 0, aug: [(START_X, START_Y); 2], recharge: 0, fire_cd: 0 }
    }
}

impl Kit {
    /// How many augments are online.
    pub fn augments(&self) -> usize {
        self.level.min(2) as usize
    }

    /// The shield's earned and charged: the next hit breaks it instead of you.
    pub fn shield_up(&self) -> bool {
        self.level >= 3 && self.recharge == 0
    }

    /// How charged the shield is, 0 (just broken) to 1 (up); 0 if it isn't earned.
    pub fn charge(&self) -> f32 {
        if self.level < 3 { 0.0 } else { 1.0 - self.recharge as f32 / SHIELD_RECHARGE as f32 }
    }

    /// Where augment `side` (0 left, 1 right) sits from the ship at focus blend
    /// `blend` (0 the wide shot, 1 the beam).
    pub fn post(side: usize, blend: f32) -> (f32, f32) {
        let s = if side == 0 { -1.0 } else { 1.0 };
        let x = AUG_WIDE.0 + (AUG_FOCUS.0 - AUG_WIDE.0) * blend;
        let y = AUG_WIDE.1 + (AUG_FOCUS.1 - AUG_WIDE.1) * blend;
        (s * x, y)
    }

    /// The style rank reached `rank`: everything it's earned that this life doesn't
    /// have yet comes online, a piece at a time. A new augment starts at the ship
    /// (`at`) and flies out to its post. Returns each new level.
    pub fn earn(&mut self, rank: u8, at: (f32, f32)) -> Vec<u8> {
        let mut out = vec![];
        while let Some(&need) = KIT_RANKS.get(self.level as usize) {
            if rank < need {
                break;
            }
            self.level += 1;
            if self.level <= 2 {
                self.aug[self.level as usize - 1] = at;
            } else {
                self.recharge = 0;
            }
            out.push(self.level);
        }
        out
    }

    /// One frame: the augments close on their posts round the ship at `ship`.
    pub fn follow(&mut self, ship: (f32, f32), blend: f32) {
        for side in 0..self.augments() {
            let (dx, dy) = Kit::post(side, blend);
            let (tx, ty) = (ship.0 + dx, ship.1 + dy);
            let a = &mut self.aug[side];
            a.0 += (tx - a.0) * AUG_FOLLOW;
            a.1 += (ty - a.1) * AUG_FOLLOW;
        }
    }

    /// One frame of the augments' guns: while you fire (the wide shot or the beam),
    /// each online augment fires a bolt straight up every `AUG_FIRE` frames.
    pub fn fire(&mut self, firing: bool, out: &mut Vec<Shot>) {
        if firing && self.fire_cd == 0 && self.augments() > 0 {
            for &(x, y) in &self.aug[..self.augments()] {
                out.push(Shot { kind: ShotKind::Aug, x, y: y + 2.0, vx: 0.0, vy: -AUG_SPEED, len: AUG_LEN, hit_w: AUG_HIT_W, dmg: AUG_DMG, age: 0 });
            }
            self.fire_cd = AUG_FIRE;
        }
        self.fire_cd = self.fire_cd.saturating_sub(1);
    }

    /// The shield took a hit: it's down until it recharges.
    pub fn break_shield(&mut self) {
        self.recharge = SHIELD_RECHARGE;
    }

    /// One frame of the shield's recharge. True on the frame it comes back up.
    pub fn tick(&mut self) -> bool {
        if self.level >= 3 && self.recharge > 0 {
            self.recharge -= 1;
            return self.recharge == 0;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_milestones_bring_it_online_a_piece_at_a_time() {
        let mut k = Kit::default();
        assert_eq!(k.earn(1, (100.0, 300.0)), Vec::<u8>::new(), "C earns nothing");
        assert_eq!(k.earn(2, (100.0, 300.0)), vec![1], "B: the left augment");
        assert_eq!((k.augments(), k.shield_up()), (1, false));
        assert_eq!(k.aug[0], (100.0, 300.0), "it starts at the ship");
        assert_eq!(k.earn(2, (0.0, 0.0)), Vec::<u8>::new(), "B again earns nothing more");
        assert_eq!(k.earn(3, (0.0, 0.0)), vec![2], "A: the right augment");
        assert_eq!(k.earn(4, (0.0, 0.0)), vec![3], "S: the shield");
        assert!(k.shield_up());
        assert_eq!(k.earn(6, (0.0, 0.0)), Vec::<u8>::new(), "that's all of it");
        let mut jump = Kit::default();
        assert_eq!(jump.earn(4, (0.0, 0.0)), vec![1, 2, 3], "a rank that skips ahead earns each piece");
    }

    #[test]
    fn augments_trail_to_their_posts_spread_wide_or_tucked_in() {
        let mut k = Kit::default();
        k.earn(3, (100.0, 300.0));
        k.follow((100.0, 300.0), 0.0);
        let first = k.aug[1];
        assert!(first.0 > 100.0 && first.0 < 100.0 + AUG_WIDE.0, "it trails: {first:?}");
        for _ in 0..60 {
            k.follow((100.0, 300.0), 0.0);
        }
        assert!((k.aug[0].0 - (100.0 - AUG_WIDE.0)).abs() < 0.01 && (k.aug[1].0 - (100.0 + AUG_WIDE.0)).abs() < 0.01, "wide: {:?}", k.aug);
        for _ in 0..60 {
            k.follow((100.0, 300.0), 1.0);
        }
        assert!((k.aug[1].0 - (100.0 + AUG_FOCUS.0)).abs() < 0.01, "tucked in for the beam: {:?}", k.aug);
        assert!(AUG_FOCUS.0 < AUG_WIDE.0);
    }

    #[test]
    fn online_augments_fire_a_bolt_each_every_few_frames_while_you_fire() {
        let mut k = Kit::default();
        let mut out = vec![];
        for _ in 0..30 {
            k.fire(true, &mut out);
        }
        assert!(out.is_empty(), "no augments, no bolts");
        k.earn(3, (100.0, 300.0));
        for _ in 0..AUG_FIRE * 10 {
            k.fire(true, &mut out);
        }
        assert_eq!(out.len(), 2 * 10, "two bolts every {AUG_FIRE} frames");
        assert!(out.iter().all(|s| s.kind == ShotKind::Aug && s.vx == 0.0 && s.vy < 0.0 && s.dmg == AUG_DMG));
        out.clear();
        for _ in 0..30 {
            k.fire(false, &mut out);
        }
        assert!(out.is_empty(), "silent while you hold fire");
    }

    #[test]
    fn the_shield_breaks_on_a_hit_and_recharges() {
        let mut k = Kit::default();
        k.earn(4, (0.0, 0.0));
        assert!(k.shield_up() && k.charge() == 1.0);
        k.break_shield();
        assert!(!k.shield_up() && k.charge() == 0.0);
        let mut back = None;
        for t in 1..=SHIELD_RECHARGE + 10 {
            if k.tick() {
                back = Some(t);
            }
        }
        assert_eq!(back, Some(SHIELD_RECHARGE), "back up after {SHIELD_RECHARGE} frames, once");
        assert!(k.shield_up());
        let mut none = Kit::default();
        none.earn(3, (0.0, 0.0));
        assert!(!none.shield_up() && !none.tick(), "not earned: no shield");
    }
}
