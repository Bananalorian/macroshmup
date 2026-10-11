//! The simulation: pure, deterministic Rust. Nothing in `sim/` draws, plays audio
//! or touches macroquad, so `cargo test` can play the game headlessly in CI.
//! The same seed with the same inputs always gives the same run.

pub mod bot;
pub mod bullets;
pub mod dummy;
pub mod enemy;
#[cfg(test)]
mod fairness;
pub mod geom;
#[cfg(test)]
mod lives;
pub mod pattern;
pub mod player;
pub mod rng;
pub mod shots;
pub mod stage1;
#[cfg(test)]
mod stage_tests;

use crate::tune::*;
use dummy::Dummy;
use enemy::{EnemyKind, Foes, Hurt};
use geom::Rect;
use stage1::Stage;
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
    /// A bullet's core passed close by.
    Graze { x: f32, y: f32 },
    /// The bomb went off at `(x, y)`.
    Bomb { x: f32, y: f32 },
    /// The bomb's ring has swept the whole screen (the moment to shake).
    BombSwept,
    /// The bomb button with no bombs left.
    NoBomb,
    /// A bullet hit the Pilgrim at `(x, y)`: the deathbomb window opens.
    Struck { x: f32, y: f32 },
    /// The window closed without a bomb: the Pilgrim is destroyed at `(x, y)`.
    Death { x: f32, y: f32 },
    /// Every bullet on screen cancelled (after a death).
    Cancel,
    /// The next ship flies in.
    Respawn,
    /// No ships left.
    GameOver,
    /// An enemy was destroyed at `(x, y)`.
    Kill { x: f32, y: f32, kind: EnemyKind },
    /// A weak point broke off the derelict.
    PartBreak { x: f32, y: f32 },
    /// The derelict's hull cracked open: its second pattern.
    Crack { x: f32, y: f32 },
    /// A new section of the stage began.
    Section(u8),
    /// The boss is coming.
    Warning,
    /// The end of the stage.
    StageEnd,
}

/// Something shots can hit: a target dummy, or part `p` of enemy `i` (0 is its body).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Dummy(usize),
    Foe(usize, usize),
}

/// Where the Pilgrim is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Life {
    Alive,
    /// Hit, with `left` frames to bomb and survive.
    Struck { left: u32 },
    /// Destroyed `t` frames ago.
    Dead { t: u32 },
    /// The next ship, `t` frames into flying up from the bottom.
    Respawning { t: u32 },
    GameOver,
}

/// A bomb going off.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BombBlast {
    pub x: f32,
    pub y: f32,
    /// Frames since it went off.
    pub t: u32,
    /// Which targets it's already hurt (once each).
    hit: [bool; 8],
}

impl BombBlast {
    /// The cancel ring's radius `t` frames in.
    pub fn radius(t: u32) -> f32 {
        BOMB_REACH * ((t + 1) as f32 / BOMB_GROW as f32).min(1.0)
    }
}

/// The focus beam this frame, in playfield px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Beam {
    pub x: f32,
    /// Where it stops: the top of the playfield or the bottom of what it hit.
    pub top: f32,
    /// Where its hitbox starts (low on the ship, like the shots).
    pub bottom: f32,
    pub hit: Option<Target>,
}

impl Beam {
    pub fn rect(&self) -> Rect {
        Rect { x0: self.x - BEAM_W * 0.5, y0: self.top, x1: self.x + BEAM_W * 0.5, y1: self.bottom }
    }
}

/// One run of the game: Stage 1, or the pattern sandbox (three target dummies and
/// the sandbox's guns). The Pilgrim, lives and bombs either way.
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
    pub life: Life,
    /// Ships left, including the one flying.
    pub lives: u8,
    pub bombs: u8,
    /// Bombs set off this run.
    pub bombs_used: u32,
    /// Frames left that bullets can't hurt you (after a bomb or a respawn), and
    /// whether it came from a respawn (the ship blinks) rather than a bomb.
    pub invuln: u32,
    pub invuln_blink: bool,
    pub bomb: Option<BombBlast>,
    /// Ticks left of the bomb's slow-motion dip (the caller runs them at half speed).
    pub dip: u32,
    /// Times struck (including ones a deathbomb saved), and times destroyed.
    pub hits: u32,
    pub deaths: u32,
    pub grazes: u32,
    /// Where bullets were when they were cancelled this frame (for the effects), and
    /// whether a bomb did it (they turn into teal shards that fly to you).
    pub cancelled: Vec<(f32, f32)>,
    pub cancel_shards: bool,
    pub foes: Foes,
    /// Stage 1's script and clock (none in the sandbox).
    pub stage: Option<Stage>,
    /// Everything shots can hit this frame, weak points before bodies.
    boxes: Vec<(Target, Rect)>,
    hits_now: Vec<Target>,
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
            life: Life::Alive,
            lives: LIVES,
            bombs: BOMBS,
            bombs_used: 0,
            invuln: 0,
            invuln_blink: false,
            bomb: None,
            dip: 0,
            hits: 0,
            deaths: 0,
            grazes: 0,
            cancelled: Vec::with_capacity(crate::tune::BULLET_CAP),
            cancel_shards: false,
            foes: Foes::new(seed),
            stage: None,
            boxes: Vec::with_capacity(64),
            hits_now: Vec::with_capacity(64),
        }
    }

    /// Stage 1 from the top.
    pub fn stage(seed: u64, tuning: Tuning) -> World {
        let mut w = World::new(seed, tuning);
        w.dummies.clear();
        w.stage = Some(Stage::new());
        w
    }

    /// Stage 1, starting `secs` seconds in (the enemies due earlier never came).
    pub fn stage_at(seed: u64, tuning: Tuning, secs: f32) -> World {
        let mut w = World::stage(seed, tuning);
        let t = stage1::frames(secs);
        if let Some(s) = &mut w.stage {
            s.skip_to(t);
            w.frame = s.t as u64;
        }
        w
    }

    /// The stage clock (frames since it began), or the run's frame in the sandbox.
    pub fn clock(&self) -> u64 {
        self.stage.as_ref().map_or(self.frame, |s| s.t as u64)
    }

    /// Debug (F6): straight to the next section, clearing the screen.
    pub fn skip_section(&mut self) {
        let Some(s) = &mut self.stage else { return };
        if let Some(next) = SECTIONS.get(s.section + 1) {
            s.skip_to(stage1::frames(next.0));
            self.foes.list.clear();
            self.hazards.bullets.clear();
        }
    }

    /// Flying and in control.
    pub fn alive(&self) -> bool {
        self.life == Life::Alive
    }

    /// Can a bullet hurt the Pilgrim right now?
    pub fn vulnerable(&self) -> bool {
        self.alive() && self.invuln == 0 && !self.invincible && self.hazards.preset().fill == 0
    }

    /// Can the Pilgrim bomb right now (in control, or inside the deathbomb window)?
    pub fn can_bomb(&self) -> bool {
        self.bombs > 0 && self.bomb.is_none() && matches!(self.life, Life::Alive | Life::Struck { .. })
    }

    /// Debug (PG_DIE): a live bullet right on the Pilgrim.
    pub fn debug_strike(&mut self) {
        let (x, y) = (self.player.x, self.player.y);
        self.hazards.bullets.push(bullets::Spawn { x, y, vx: 0.0, vy: 0.0, kind: bullets::Kind::Orb, behavior: bullets::Behavior::Straight, owner: 0 });
        let i = self.hazards.bullets.len() - 1;
        self.hazards.bullets.age[i] = 30;
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
        self.cancel_shards = false;

        // the bomb first, so a bomb pressed this frame already protects this frame
        let mut saved = false;
        if c.bomb {
            if self.can_bomb() {
                saved = matches!(self.life, Life::Struck { .. });
                self.start_bomb();
            } else if self.bombs == 0 && self.alive() {
                self.events.push(Event::NoBomb);
            }
        }
        self.advance_life(saved);

        let flying = self.alive();
        if flying {
            self.player.step(&c, &self.tuning);
        } else if let Life::Respawning { t } = self.life {
            // up from below the screen, easing to a stop
            let k = 1.0 - (1.0 - (t + 1) as f32 / RESPAWN_FLY as f32).powi(2);
            let y0 = FIELD_H as f32 + 24.0;
            self.player.place(START_X, y0 + (START_Y - y0) * k);
        }
        for d in &mut self.dummies {
            d.tick(f);
        }
        // the stage's timeline: who arrives this frame
        if let Some(s) = &mut self.stage {
            s.tick(&mut self.foes, &mut self.events);
        }
        let me = (self.player.x, self.player.y);
        let mut m = [(0.0, 0.0, false); 8];
        let n = self.mounts(&mut m);
        self.hazards.step(me, &m[..n]);
        let fire_ok = self.stage.as_ref().is_none_or(|s| s.fire_ok());
        self.foes.step(&mut self.hazards, me, fire_ok, &mut self.events);
        self.build_boxes();
        self.step_bomb();

        for s in &mut self.shots {
            s.x += s.vx;
            s.y += s.vy;
            s.age += 1;
        }
        self.shots.retain(|s| !s.gone());

        // the wide shot without focus; the beam with it (only while flying)
        let p = &mut self.player;
        let fire = c.fire && flying;
        p.firing = fire && !c.focus;
        p.beaming = fire && c.focus;
        p.fire_t = if fire { p.fire_t + 1 } else { 0 };
        if p.firing && p.fire_cd == 0 {
            shots::volley(p, &mut self.shots);
            p.volley += 1;
            p.fire_cd = FIRE_INTERVAL;
            self.events.push(Event::Volley { x: p.x, y: p.y });
        }
        p.fire_cd = p.fire_cd.saturating_sub(1);

        self.collide_shots();
        self.fire_beam();
        if flying {
            self.bump();
            self.collide_bullets();
        }
        self.refill_shots();
        self.invuln = self.invuln.saturating_sub(1);
        self.dip = self.dip.saturating_sub(1);
    }

    /// The deathbomb window, the death, the cancel, the respawn.
    fn advance_life(&mut self, saved: bool) {
        match self.life {
            Life::Struck { left } => {
                if saved {
                    self.life = Life::Alive;
                } else if left <= 1 {
                    self.final_death();
                } else {
                    self.life = Life::Struck { left: left - 1 };
                }
            }
            Life::Dead { t } => {
                let t = t + 1;
                let since_hit = t + DEATHBOMB;
                if since_hit == DEATH_CANCEL {
                    let mut out = std::mem::take(&mut self.cancelled);
                    self.hazards.cancel_within(0.0, 0.0, f32::MAX, &mut out);
                    self.cancelled = out;
                    self.events.push(Event::Cancel);
                }
                if since_hit < RESPAWN_AT {
                    self.life = Life::Dead { t };
                } else if self.lives == 0 {
                    self.life = Life::GameOver;
                    self.events.push(Event::GameOver);
                } else {
                    self.player = Player::new();
                    self.player.place(START_X, FIELD_H as f32 + 24.0);
                    self.player.history = [(START_X, FIELD_H as f32 + 24.0); player::HISTORY];
                    self.bombs = BOMBS;
                    self.invuln = RESPAWN_INVULN;
                    self.invuln_blink = true;
                    self.life = Life::Respawning { t: 0 };
                    self.events.push(Event::Respawn);
                }
            }
            Life::Respawning { t } => {
                self.life = if t + 1 >= RESPAWN_FLY { Life::Alive } else { Life::Respawning { t: t + 1 } };
            }
            Life::Alive | Life::GameOver => {}
        }
    }

    fn final_death(&mut self) {
        self.deaths += 1;
        if let Some(s) = &mut self.stage {
            s.deaths[s.section] += 1;
        }
        self.lives = self.lives.saturating_sub(1);
        self.life = Life::Dead { t: 0 };
        self.beam = None;
        self.events.push(Event::Death { x: self.player.x, y: self.player.y });
    }

    fn start_bomb(&mut self) {
        let (x, y) = (self.player.x, self.player.y);
        self.bombs -= 1;
        self.bombs_used += 1;
        self.bomb = Some(BombBlast { x, y, t: 0, hit: [false; 8] });
        for e in &mut self.foes.list {
            for q in &mut e.parts {
                q.bombed = false;
            }
        }
        self.invuln = self.invuln.max(BOMB_INVULN);
        self.invuln_blink = false;
        self.dip = BOMB_DIP;
        self.events.push(Event::Bomb { x, y });
    }

    /// The bomb's ring spreads, cancelling every bullet it crosses (into teal
    /// shards) and hurting every target once.
    fn step_bomb(&mut self) {
        let Some(mut b) = self.bomb else { return };
        let r = BombBlast::radius(b.t);
        let mut out = std::mem::take(&mut self.cancelled);
        let before = out.len();
        self.hazards.cancel_within(b.x, b.y, r, &mut out);
        self.cancel_shards |= out.len() > before;
        self.cancelled = out;
        self.hits_now.clear();
        for &(t, rect) in &self.boxes {
            if rect.dist(b.x, b.y) > r {
                continue;
            }
            match t {
                Target::Dummy(k) if k < 8 && !b.hit[k] => {
                    b.hit[k] = true;
                    self.hits_now.push(t);
                }
                Target::Foe(i, p) if !self.foes.list[i].parts[p].bombed => {
                    self.foes.list[i].parts[p].bombed = true;
                    self.hits_now.push(t);
                }
                _ => {}
            }
        }
        for k in 0..self.hits_now.len() {
            let t = self.hits_now[k];
            // the big ones only lose a share of their health to a bomb
            let cap = match t {
                Target::Dummy(k) if self.dummies[k].kind == dummy::DummyKind::Big => self.dummies[k].max_hp * BOMB_BOSS_CAP,
                Target::Foe(i, 0) if self.foes.list[i].kind == EnemyKind::Hull => self.foes.list[i].parts[0].max_hp * BOMB_BOSS_CAP,
                _ => f32::MAX,
            };
            self.hurt(t, BOMB_DAMAGE.min(cap));
        }
        if b.t + 1 == BOMB_GROW {
            self.events.push(Event::BombSwept);
        }
        b.t += 1;
        self.bomb = (b.t < BOMB_ACTIVE).then_some(b);
    }

    /// Everything shots can hit this frame: the dummies, and the enemies below the top
    /// dead zone (weak points before the body they sit on).
    fn build_boxes(&mut self) {
        self.boxes.clear();
        for (k, d) in self.dummies.iter().enumerate() {
            if d.alive() {
                self.boxes.push((Target::Dummy(k), d.rect()));
            }
        }
        for (i, e) in self.foes.list.iter().enumerate() {
            if !e.hittable() {
                continue;
            }
            for p in (1..e.parts.len()).filter(|&p| e.parts[p].alive()) {
                self.boxes.push((Target::Foe(i, p), e.rect(p)));
            }
            self.boxes.push((Target::Foe(i, 0), e.rect(0)));
        }
    }

    fn hurt(&mut self, t: Target, dmg: f32) {
        match t {
            Target::Dummy(i) => {
                let f = self.frame;
                let d = &mut self.dummies[i];
                if d.damage(dmg, f) {
                    let big = d.kind == dummy::DummyKind::Big;
                    self.events.push(Event::Break { x: d.x, y: d.y, big });
                    self.boxes.retain(|b| b.0 != t);
                }
            }
            Target::Foe(i, _) if !self.foes.list[i].alive() => {}
            Target::Foe(i, p) => match self.foes.list[i].hurt(p, dmg) {
                Hurt::Died => self.kill(i),
                Hurt::Broke => {
                    let r = self.foes.list[i].rect(p);
                    self.events.push(Event::PartBreak { x: (r.x0 + r.x1) * 0.5, y: (r.y0 + r.y1) * 0.5 });
                    self.boxes.retain(|b| b.0 != t);
                }
                Hurt::Hit | Hurt::Nothing => {}
            },
        }
    }

    /// Enemy `i` is destroyed. A high-HP one takes its bullets with it, as shards.
    fn kill(&mut self, i: usize) {
        let e = &self.foes.list[i];
        let (id, kind) = (e.id, e.kind);
        self.events.push(Event::Kill { x: e.x, y: e.y, kind });
        self.boxes.retain(|b| !matches!(b.0, Target::Foe(j, _) if j == i));
        if kind.heavy() {
            let mut out = std::mem::take(&mut self.cancelled);
            let before = out.len();
            self.hazards.cancel_owner(id, &mut out);
            self.cancel_shards |= out.len() > before;
            self.cancelled = out;
        }
        if let Some(s) = &mut self.stage {
            s.killed += 1;
            if kind == EnemyKind::Hull {
                s.hurry();
            }
        }
    }

    fn collide_shots(&mut self) {
        let mut i = 0;
        while i < self.shots.len() {
            let r = self.shots[i].rect();
            let live = self.shots[i].dmg > 0.0; // the debug filler shots pass through
            let hit = self.boxes.iter().position(|b| live && b.1.overlaps(&r));
            if let Some(k) = hit {
                let (t, rect) = self.boxes[k];
                let s = self.shots.swap_remove(i);
                let (_, ty) = s.tip();
                let y = ty.max(rect.y0).min(rect.y1);
                self.events.push(Event::Hit { x: s.x, y, beam: false });
                self.hurt(t, s.dmg);
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
        let hit = self.boxes.iter().filter(|b| b.1.overlaps(&lane)).max_by(|a, b| a.1.y1.total_cmp(&b.1.y1)).copied();
        let top = hit.map_or(0.0, |b| b.1.y1.min(bottom));
        self.beam = Some(Beam { x: px, top, bottom, hit: hit.map(|b| b.0) });
        if let Some((t, _)) = hit {
            self.events.push(Event::Hit { x: px, y: top, beam: true });
            self.hurt(t, BEAM_DMG);
        }
        // the point-blank aura burns anything close
        self.hits_now.clear();
        self.hits_now.extend(self.boxes.iter().filter(|b| b.1.dist(px, py) <= AURA_R).map(|b| b.0));
        for k in 0..self.hits_now.len() {
            let t = self.hits_now[k];
            if self.boxes.iter().any(|b| b.0 == t) {
                self.hurt(t, AURA_DMG);
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
        let safe = !self.vulnerable();
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
        // crashing into an enemy's body is as bad as a bullet
        if !safe {
            hit |= self.foes.list.iter().any(|e| e.alive() && e.body().dist(px, py) < HITBOX_R);
        }
        if hit {
            // struck: the deathbomb window opens
            self.hits += 1;
            self.life = Life::Struck { left: DEATHBOMB };
            self.player.hit_t = BUMP_FLASH;
            self.events.push(Event::Struck { x: px, y: py });
        }
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
        assert_eq!(b.hit, Some(Target::Dummy(1)));
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
