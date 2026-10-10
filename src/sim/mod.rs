//! The simulation: pure, deterministic Rust. Nothing in `sim/` draws, plays audio
//! or touches macroquad, so `cargo test` can play the game headlessly in CI.
//! The same seed with the same inputs always gives the same run.

pub mod bot;
pub mod bullets;
pub mod dummy;
#[cfg(test)]
mod fairness;
pub mod geom;
pub mod pattern;
pub mod player;
pub mod rng;
pub mod shots;

use crate::tune::*;
use dummy::Dummy;
use geom::Rect;
use pattern::Hazards;
use player::{Controls, Player, Tuning};
use rng::Rng;
use shots::{Shot, ShotKind};

/// What happened this frame, for sound, rumble and effects to pick up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// The wide shot fired a volley from the ship at `(x, y)`.
    Volley { x: f32, y: f32 },
    /// A shot (or the beam) hit something at `(x, y)`.
    Hit { x: f32, y: f32, beam: bool },
    /// A target broke.
    Break { x: f32, y: f32, big: bool },
    /// The hull bumped into a target.
    Bump,
    /// The bomb button (the bomb itself arrives in Sprint 4).
    BombStub,
    /// A bullet's core passed close by.
    Graze { x: f32, y: f32 },
    /// The placeholder death: the Pilgrim was hit at `(x, y)` and the screen resets.
    Death { x: f32, y: f32 },
}

/// The focus beam this frame, in playfield px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Beam {
    pub x: f32,
    /// Where it stops: the top of the playfield or the bottom of what it hit.
    pub top: f32,
    /// Where its hitbox starts (low on the ship, like the shots).
    pub bottom: f32,
    pub hit: Option<usize>,
}

impl Beam {
    pub fn rect(&self) -> Rect {
        Rect { x0: self.x - BEAM_W * 0.5, y0: self.top, x1: self.x + BEAM_W * 0.5, y1: self.bottom }
    }
}

/// One run of the game: for now the Pilgrim, three target dummies and the
/// pattern sandbox's guns.
pub struct World {
    pub seed: u64,
    /// Ticks since the run began. Every gameplay timing counts these.
    pub frame: u64,
    /// The gameplay RNG. Cosmetic effects use their own.
    pub rng: Rng,
    pub tuning: Tuning,
    pub player: Player,
    pub shots: Vec<Shot>,
    pub dummies: Vec<Dummy>,
    pub beam: Option<Beam>,
    /// This frame's events (the caller clears them after reading).
    pub events: Vec<Event>,
    /// Debug: keep this many extra shots alive (the 500-shot performance check).
    pub fill_shots: usize,
    /// The guns and their bullets.
    pub hazards: Hazards,
    /// Bullets can't hurt you (sandbox option, F5).
    pub invincible: bool,
    pub deaths: u32,
    pub grazes: u32,
    /// Where bullets were when a death cleared them (for the effects), this frame only.
    pub cancelled: Vec<(f32, f32)>,
}

impl World {
    pub fn new(seed: u64, tuning: Tuning) -> World {
        World {
            seed,
            frame: 0,
            rng: Rng::new(seed),
            tuning,
            player: Player::new(),
            shots: Vec::with_capacity(1024),
            dummies: Dummy::sandbox(),
            beam: None,
            events: Vec::with_capacity(64),
            fill_shots: 0,
            hazards: Hazards::new(0, seed),
            invincible: false,
            deaths: 0,
            grazes: 0,
            cancelled: Vec::with_capacity(crate::tune::BULLET_CAP),
        }
    }

    /// Switch the sandbox to preset `i`.
    pub fn set_pattern(&mut self, i: usize) {
        self.hazards.load(i);
    }

    /// Where the target dummies are, for guns that ride them.
    pub fn mounts(&self, buf: &mut [(f32, f32, bool); 8]) -> usize {
        let n = self.dummies.len().min(8);
        for (i, d) in self.dummies.iter().take(n).enumerate() {
            buf[i] = (d.x, d.y, d.alive());
        }
        n
    }

    /// Advance the game by exactly one 60 Hz tick, with this frame's input.
    pub fn tick(&mut self, c: Controls) {
        self.frame += 1;
        let f = self.frame;
        self.cancelled.clear();
        self.player.step(&c, &self.tuning);
        for d in &mut self.dummies {
            d.tick(f);
        }
        let mut m = [(0.0, 0.0, false); 8];
        let n = self.mounts(&mut m);
        self.hazards.step((self.player.x, self.player.y), &m[..n]);

        for s in &mut self.shots {
            s.x += s.vx;
            s.y += s.vy;
            s.age += 1;
        }
        self.shots.retain(|s| !s.gone());

        // the wide shot without focus; the beam with it
        let p = &mut self.player;
        p.firing = c.fire && !c.focus;
        p.beaming = c.fire && c.focus;
        p.fire_t = if c.fire { p.fire_t + 1 } else { 0 };
        if p.firing && p.fire_cd == 0 {
            shots::volley(p, &mut self.shots);
            p.volley += 1;
            p.fire_cd = FIRE_INTERVAL;
            self.events.push(Event::Volley { x: p.x, y: p.y });
        }
        p.fire_cd = p.fire_cd.saturating_sub(1);

        self.collide_shots();
        self.fire_beam();
        self.bump();
        self.collide_bullets();
        if c.bomb {
            self.events.push(Event::BombStub);
        }
        self.refill_shots();
    }

    fn hurt(&mut self, i: usize, dmg: f32) {
        let f = self.frame;
        let d = &mut self.dummies[i];
        if d.damage(dmg, f) {
            let big = d.kind == dummy::DummyKind::Big;
            self.events.push(Event::Break { x: d.x, y: d.y, big });
        }
    }

    fn collide_shots(&mut self) {
        let mut i = 0;
        while i < self.shots.len() {
            let r = self.shots[i].rect();
            let live = self.shots[i].dmg > 0.0; // the debug filler shots pass through
            let hit = self.dummies.iter().position(|d| live && d.alive() && d.rect().overlaps(&r));
            if let Some(k) = hit {
                let s = self.shots.swap_remove(i);
                let (_, ty) = s.tip();
                let y = ty.max(self.dummies[k].rect().y0).min(self.dummies[k].rect().y1);
                self.events.push(Event::Hit { x: s.x, y, beam: false });
                self.hurt(k, s.dmg);
            } else {
                i += 1;
            }
        }
    }

    fn fire_beam(&mut self) {
        self.beam = None;
        if !self.player.beaming {
            return;
        }
        let (px, py) = (self.player.x, self.player.y);
        let bottom = py + MUZZLE_Y;
        let lane = Rect { x0: px - BEAM_W * 0.5, y0: -1000.0, x1: px + BEAM_W * 0.5, y1: bottom };
        // it stops at the nearest thing in its lane
        let hit = (0..self.dummies.len())
            .filter(|&k| self.dummies[k].alive() && self.dummies[k].rect().overlaps(&lane))
            .max_by(|&a, &b| self.dummies[a].rect().y1.total_cmp(&self.dummies[b].rect().y1));
        let top = hit.map_or(0.0, |k| self.dummies[k].rect().y1.min(bottom));
        self.beam = Some(Beam { x: px, top, bottom, hit });
        if let Some(k) = hit {
            self.events.push(Event::Hit { x: px, y: top, beam: true });
            self.hurt(k, BEAM_DMG);
        }
        // the point-blank aura burns anything close
        for k in 0..self.dummies.len() {
            if self.dummies[k].alive() && self.dummies[k].rect().dist(px, py) <= AURA_R {
                self.hurt(k, AURA_DMG);
            }
        }
    }

    /// The ship's body, for bumping into things.
    pub fn body(&self) -> Rect {
        Rect { x0: self.player.x - 7.0, y0: self.player.y - 10.0, x1: self.player.x + 7.0, y1: self.player.y + 14.0 }
    }

    fn bump(&mut self) {
        let body = self.body();
        if self.player.bump_cd == 0 && self.dummies.iter().any(|d| d.alive() && d.rect().overlaps(&body)) {
            self.player.hit_t = BUMP_FLASH;
            self.player.bump_cd = BUMP_COOLDOWN;
            self.events.push(Event::Bump);
        }
    }

    /// Enemy bullets against the hitbox: hits, and grazes (once per bullet).
    fn collide_bullets(&mut self) {
        let (px, py) = (self.player.x, self.player.y);
        let safe = self.invincible || self.hazards.preset().fill > 0;
        let b = &mut self.hazards.bullets;
        let mut hit = false;
        for i in 0..b.len() {
            if b.age[i] < BLOOM_FRAMES {
                continue; // still blooming: harmless
            }
            let d = b.core_dist(i, px, py);
            if d < b.kind[i].hit_r() + HITBOX_R {
                hit |= !safe;
            } else if d <= GRAZE_R && !b.grazed[i] {
                b.grazed[i] = true;
                self.grazes += 1;
                self.events.push(Event::Graze { x: b.x[i], y: b.y[i] });
            }
        }
        if hit {
            self.die();
        }
    }

    /// The placeholder death: flash, clear the screen, start again from the bottom.
    fn die(&mut self) {
        let b = &self.hazards.bullets;
        self.cancelled.extend((0..b.len()).map(|i| (b.x[i], b.y[i])));
        self.events.push(Event::Death { x: self.player.x, y: self.player.y });
        self.hazards.reset();
        self.deaths += 1;
        self.player = Player::new();
        self.player.hit_t = BUMP_FLASH;
        self.player.bump_cd = BUMP_COOLDOWN;
    }

    fn refill_shots(&mut self) {
        while self.shots.len() < self.fill_shots {
            let x = self.rng.range(10.0, FIELD_W as f32 - 10.0);
            let y = self.rng.range(20.0, FIELD_H as f32);
            let kind = if self.rng.chance(0.5) { ShotKind::Bolt } else { ShotKind::Pod };
            let vx = self.rng.range(-0.6, 0.6);
            let vy = -self.rng.range(1.0, 3.0);
            let (len, dmg) = if kind == ShotKind::Bolt { (BOLT_LEN, 0.0) } else { (POD_LEN, 0.0) };
            self.shots.push(Shot { kind, x, y, vx, vy, len, hit_w: 1.0, dmg, age: 0 });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::dummy::DummyKind;
    use super::*;

    fn fire() -> Controls {
        Controls { fire: true, ..Default::default() }
    }

    fn beam() -> Controls {
        Controls { fire: true, focus: true, ..Default::default() }
    }

    /// A world with one dummy, nothing else.
    fn with_dummy(kind: DummyKind, x: f32, y: f32) -> World {
        let mut w = World::new(1, Tuning::default());
        w.dummies = vec![Dummy::new(kind, x, y)];
        w
    }

    #[test]
    fn the_wide_shot_fires_every_three_frames() {
        let mut w = World::new(1, Tuning::default());
        w.dummies.clear();
        let mut volleys = 0;
        for _ in 0..30 {
            w.tick(fire());
            volleys += w.events.iter().filter(|e| matches!(e, Event::Volley { .. })).count();
            w.events.clear();
        }
        assert_eq!(volleys, 10);
    }

    #[test]
    fn shots_hit_a_target_the_ship_is_sitting_on() {
        let mut w = with_dummy(DummyKind::Still, START_X, START_Y);
        w.tick(fire());
        assert!(w.dummies[0].hp < w.dummies[0].max_hp, "the very first frame of fire lands");
        let mut w = with_dummy(DummyKind::Still, START_X, START_Y);
        w.tick(beam());
        assert!(w.dummies[0].hp < w.dummies[0].max_hp, "so does the beam");
    }

    #[test]
    fn the_beam_does_about_one_point_six_times_the_wide_shot() {
        // a small target far up the screen: only the two center bolts reach it
        let run = |c: Controls| {
            let mut w = with_dummy(DummyKind::Still, START_X, 60.0);
            w.dummies[0].hp = 50_000.0;
            for _ in 0..60 {
                w.tick(c); // let the first shots arrive
            }
            let before = w.dummies[0].hp;
            for _ in 0..600 {
                w.tick(c);
                w.events.clear();
            }
            before - w.dummies[0].hp
        };
        let ratio = run(beam()) / run(fire());
        assert!((ratio - 1.6).abs() < 0.05, "beam/wide = {ratio}");
    }

    #[test]
    fn the_beam_stops_at_the_nearest_target() {
        let mut w = World::new(1, Tuning::default());
        w.dummies = vec![Dummy::new(DummyKind::Big, START_X, 60.0), Dummy::new(DummyKind::Still, START_X, 200.0)];
        w.tick(beam());
        let b = w.beam.unwrap();
        assert_eq!(b.hit, Some(1));
        assert_eq!(b.top, w.dummies[1].rect().y1);
        assert_eq!(w.dummies[0].hp, w.dummies[0].max_hp, "the far one is shielded");
    }

    #[test]
    fn the_aura_burns_point_blank() {
        // beside the ship, out of the beam's lane
        let mut w = with_dummy(DummyKind::Still, START_X + 30.0, START_Y);
        w.tick(beam());
        assert_eq!(w.beam.unwrap().hit, None);
        assert_eq!(w.dummies[0].max_hp - w.dummies[0].hp, AURA_DMG);
    }

    #[test]
    fn targets_break_and_say_so() {
        let mut w = with_dummy(DummyKind::Moving, START_X, 200.0);
        let mut broke = false;
        for _ in 0..1200 {
            w.tick(fire());
            broke |= w.events.iter().any(|e| matches!(e, Event::Break { big: false, .. }));
            w.events.clear();
        }
        assert!(broke);
    }

    #[test]
    fn bumping_flashes_the_hull_with_a_cooldown() {
        let mut w = with_dummy(DummyKind::Still, START_X, START_Y);
        w.tick(Controls::default());
        assert_eq!(w.player.hit_t, BUMP_FLASH);
        assert!(w.events.contains(&Event::Bump));
        w.events.clear();
        for _ in 0..5 {
            w.tick(Controls::default());
        }
        assert!(!w.events.contains(&Event::Bump), "not again straight away");
    }

    #[test]
    fn the_bomb_button_is_heard() {
        let mut w = World::new(1, Tuning::default());
        w.tick(Controls { bomb: true, ..Default::default() });
        assert!(w.events.contains(&Event::BombStub));
    }

    #[test]
    fn shots_leave_the_screen_and_are_freed() {
        let mut w = World::new(1, Tuning::default());
        w.dummies.clear();
        let mut most = 0;
        for i in 0..1200 {
            let mv = if i % 120 < 60 { (1.0, 0.0) } else { (-1.0, 0.0) };
            w.tick(Controls { mv, fire: true, ..Default::default() });
            most = most.max(w.shots.len());
        }
        assert!(most < 120, "{most} shots alive at once");
        w.tick(Controls::default());
        for _ in 0..60 {
            w.tick(Controls::default());
        }
        assert!(w.shots.is_empty());
    }

    #[test]
    fn same_seed_same_inputs_same_run() {
        let script = |i: u32| Controls {
            mv: ((i as f32 * 0.05).cos(), (i as f32 * 0.03).sin()),
            fire: i % 200 < 150,
            focus: i % 300 > 200,
            ..Default::default()
        };
        let run = || {
            let mut w = World::new(77, Tuning::default());
            w.fill_shots = 50;
            for i in 0..900 {
                w.tick(script(i));
                w.events.clear();
            }
            (w.player.x, w.player.y, w.shots.len(), w.dummies.iter().map(|d| d.hp).collect::<Vec<_>>(), w.rng.next_u64())
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn a_tick_is_one_frame() {
        let mut w = World::new(3, Tuning::default());
        for _ in 0..600 {
            w.tick(Controls::default());
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
                for banned in ["macroquad", "miniquad", "crate::render", "crate::audio", "crate::input"] {
                    assert!(!code.contains(banned), "{} uses {banned}", path.display());
                }
                checked += 1;
            }
        }
        assert!(checked >= 6);
    }
}
