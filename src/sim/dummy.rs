//! Target dummies for the empty stage: one still, one moving, one big. They take
//! hits, flash, break and come back, and report the damage per second they took.

use super::geom::Rect;
use crate::tune::{DUMMY_HP, DUMMY_RESPAWN, DUMMY_SWING, HIT_FLASH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DummyKind {
    Still,
    Moving,
    Big,
}

impl DummyKind {
    /// Size in px.
    pub fn size(self) -> (f32, f32) {
        match self {
            DummyKind::Still => (26.0, 26.0),
            DummyKind::Moving => (20.0, 20.0),
            DummyKind::Big => (66.0, 42.0),
        }
    }
}

pub struct Dummy {
    pub kind: DummyKind,
    pub home: (f32, f32),
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub max_hp: f32,
    /// Hit flash frames left; frames until it may flash again.
    pub flash: u8,
    flash_cd: u8,
    /// Frames until it comes back (0 while it's up).
    pub down: u32,
    age: u32,
    /// Damage taken on each of the last 60 frames.
    log: [f32; 60],
}

impl Dummy {
    pub fn new(kind: DummyKind, x: f32, y: f32) -> Dummy {
        let hp = DUMMY_HP[kind as usize];
        Dummy { kind, home: (x, y), x, y, hp, max_hp: hp, flash: 0, flash_cd: 0, down: 0, age: 0, log: [0.0; 60] }
    }

    /// The three dummies of the empty stage.
    pub fn sandbox() -> Vec<Dummy> {
        vec![Dummy::new(DummyKind::Still, 62.0, 86.0), Dummy::new(DummyKind::Moving, 135.0, 170.0), Dummy::new(DummyKind::Big, 196.0, 78.0)]
    }

    pub fn alive(&self) -> bool {
        self.down == 0
    }

    pub fn rect(&self) -> Rect {
        let (w, h) = self.kind.size();
        Rect::centered(self.x, self.y, w, h)
    }

    /// Damage per second over the last second.
    pub fn dps(&self) -> f32 {
        self.log.iter().sum()
    }

    pub fn tick(&mut self, frame: u64) {
        self.log[(frame % 60) as usize] = 0.0;
        self.flash = self.flash.saturating_sub(1);
        self.flash_cd = self.flash_cd.saturating_sub(1);
        if self.down > 0 {
            self.down -= 1;
            if self.down == 0 {
                self.hp = self.max_hp;
                self.age = 0;
            }
            return;
        }
        self.age += 1;
        if self.kind == DummyKind::Moving {
            let (amp, period) = DUMMY_SWING;
            self.x = self.home.0 + amp * (std::f32::consts::TAU * self.age as f32 / period as f32).sin();
        }
    }

    /// Take a hit. Returns true if this one broke it.
    pub fn damage(&mut self, dmg: f32, frame: u64) -> bool {
        if !self.alive() {
            return false;
        }
        self.log[(frame % 60) as usize] += dmg;
        self.hp -= dmg;
        if self.flash_cd == 0 {
            (self.flash, self.flash_cd) = HIT_FLASH;
        }
        if self.hp <= 0.0 {
            self.hp = 0.0;
            self.down = DUMMY_RESPAWN;
            self.x = self.home.0;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummies_break_and_come_back() {
        let mut d = Dummy::new(DummyKind::Still, 50.0, 50.0);
        assert!(!d.damage(100.0, 0));
        assert!(d.damage(100.0, 1));
        assert!(!d.alive());
        assert!(!d.damage(5.0, 2), "can't be hit while down");
        for f in 0..DUMMY_RESPAWN as u64 {
            d.tick(f);
        }
        assert!(d.alive());
        assert_eq!(d.hp, d.max_hp);
    }

    #[test]
    fn the_hit_flash_is_rate_limited() {
        let mut d = Dummy::new(DummyKind::Big, 50.0, 50.0);
        let mut flashes = 0;
        for f in 0..60 {
            d.tick(f);
            let was = d.flash;
            d.damage(1.0, f);
            if d.flash > was {
                flashes += 1;
            }
        }
        assert_eq!(flashes, 60 / HIT_FLASH.1 as usize);
    }

    #[test]
    fn dps_counts_the_last_second() {
        let mut d = Dummy::new(DummyKind::Big, 50.0, 50.0);
        for f in 0..120 {
            d.tick(f);
            d.damage(2.0, f);
        }
        assert_eq!(d.dps(), 120.0);
        for f in 120..180 {
            d.tick(f);
        }
        assert_eq!(d.dps(), 0.0);
    }

    #[test]
    fn the_moving_one_swings_about_home() {
        let mut d = Dummy::new(DummyKind::Moving, 135.0, 100.0);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for f in 0..DUMMY_SWING.1 as u64 {
            d.tick(f);
            lo = lo.min(d.x);
            hi = hi.max(d.x);
        }
        assert!((hi - 135.0 - DUMMY_SWING.0).abs() < 0.5 && (135.0 - lo - DUMMY_SWING.0).abs() < 0.5);
        assert!(Dummy::sandbox().iter().all(|d| d.rect().x0 >= 20.0 && d.rect().x1 <= 250.0), "nothing hugs the edges");
    }
}
