//! The simulation: pure, deterministic Rust. Nothing in `sim/` draws, plays audio
//! or touches macroquad, so `cargo test` can play the game headlessly in CI.
//! The same seed with the same inputs always gives the same run.

pub mod boss;
#[cfg(test)]
mod boss_tests;
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
pub mod score;
pub mod shots;
mod slam;
pub use slam::Slam;
pub mod style;
pub mod stage1;
#[cfg(test)]
mod stage_tests;

use crate::tune::*;
use boss::{BOSS_ID, Boss};
use dummy::Dummy;
use enemy::{EnemyKind, Foes, Hurt};
use geom::Rect;
use stage1::Stage;
use pattern::Hazards;
use player::{Controls, Player, Tuning};
use rng::Rng;
use score::{Award, Score, Tally};
use shots::{Shot, ShotKind};

/// What happened this frame, for sound and effects to pick up.
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
    /// JAWSTHIC is coming: its fin is about to cut through the disk.
    BossArrive,
    /// JAWSTHIC breached the disk at `(x, y)` (the splash).
    BossBreach { x: f32, y: f32 },
    /// It roared, in position: the fight is about to begin.
    BossRoar { x: f32, y: f32 },
    /// JAWSTHIC's jaws snapped shut at `(x, y)`, lunging.
    BossBite { x: f32, y: f32 },
    /// The style rank changed to `rank` (0 D to 6 SSS), up or down.
    Style { rank: u8, up: bool },
    /// The Slam: link `link` of a chain dashed off from `(x, y)`, and struck at `(x, y)`.
    SlamDash { x: f32, y: f32, link: u32 },
    Slam { x: f32, y: f32, link: u32 },
    /// Phase `n` (1-4) began.
    BossPhase(u8),
    /// One of phase 2's fin turrets broke off (`fin` 0 the dorsal, 1 the pectoral).
    FinBreak { x: f32, y: f32, fin: u8 },
    /// A phase ended: its share of health gone, or (`timeout`) its time. Its bullets cancel.
    BossBreak { x: f32, y: f32, timeout: bool },
    /// It glitched round to face the other way.
    BossTurn,
    /// A laser's warning line appeared, and the beam fired.
    LaserWarn,
    LaserFire,
    /// Destroyed at `(x, y)`: every bullet cancels and the chained blasts begin.
    BossDown { x: f32, y: f32 },
    /// One of the death's blasts; `last` is the big one.
    BossBlast { x: f32, y: f32, last: bool },
    /// Phase 4 timed out: the black hole takes it (no kill).
    BossFall,
    /// It's gone (destroyed or swallowed): the stage ends.
    BossGone,
    /// Points scored at `(x, y)`, and what for.
    Points { x: f32, y: f32, n: u32, how: Award },
    /// The score passed an extend: an extra ship.
    Extend,
}

/// Something shots can hit: a target dummy, part `p` of enemy `i` (0 is its body), or
/// part `p` of the boss (0 its body, 1 and 2 its fins' turrets).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Dummy(usize),
    Foe(usize, usize),
    Boss(usize),
}

/// What's dealing damage (for the style a kill earns).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum By {
    Shot,
    Beam,
    Bomb,
    Slam,
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
    /// What it stopped on.
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
    /// The rush gauge (a segment a slam link), the chain under way, and in its aim,
    /// the target its next link would strike.
    pub rush: f32,
    pub slam: Option<slam::Slam>,
    pub reticle: Option<(Target, Rect)>,
    /// The style rank (Stage 1 only), and what's dealing damage right now.
    pub style: style::Style,
    by: By,
    pub foes: Foes,
    /// Stage 1's script and clock (none in the sandbox).
    pub stage: Option<Stage>,
    /// JAWSTHIC, once it's arrived.
    pub boss: Option<Boss>,
    /// Ticks left of the boss's death in slow motion (the caller runs them at quarter speed).
    pub slow: u32,
    /// Points, grades and the stage's tally (Stage 1 only; the sandbox doesn't score).
    pub score: Score,
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
            rush: RUSH_START,
            slam: None,
            reticle: None,
            style: style::Style::default(),
            by: By::Shot,
            foes: Foes::new(seed),
            stage: None,
            boss: None,
            slow: 0,
            score: Score::default(),
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

    /// Boss practice (PG_SCENE=boss:<phase>): Stage 1 at the boss, in phase `phase`,
    /// or from its intro (0).
    pub fn boss_at(seed: u64, tuning: Tuning, phase: u8) -> World {
        let mut w = World::stage_at(seed, tuning, JAWS_AT);
        if phase > 0 {
            let spin = w.foes.rng.range(0.0, 360.0);
            w.boss = Some(Boss::at_phase(spin, phase));
            if let Some(s) = &mut w.stage {
                s.boss_called = true;
                s.skip_to(stage1::frames(SECTIONS[7].0));
                w.frame = s.t as u64;
            }
        }
        w
    }

    /// The stage clock (frames since it began), or the run's frame in the sandbox.
    pub fn clock(&self) -> u64 {
        self.stage.as_ref().map_or(self.frame, |s| s.t as u64)
    }

    /// Debug (F6): straight to the next section, clearing the screen; during the
    /// boss, straight to its next phase.
    pub fn skip_section(&mut self) {
        if let Some(b) = self.boss.as_mut().filter(|b| b.act == boss::Act::Fight) {
            let from = self.events.len();
            b.hurt(0, JAWS_HP, &mut self.events);
            self.boss_aftermath(from);
            return;
        }
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
        self.hazards.bullets.push(bullets::Spawn { x, y, vx: 0.0, vy: 0.0, kind: bullets::Kind::Orb, behavior: bullets::Behavior::Straight, owner: 0, hue: Default::default() });
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
        let first_event = self.events.len();
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
        // the slam drives the ship while it dashes
        let dashing = flying && self.step_slam(&c);
        if flying && !dashing {
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
        // the stage's timeline: who arrives this frame, and the boss after the WARNING
        if let Some(s) = &mut self.stage {
            s.tick(&mut self.foes, &mut self.events);
            if !s.boss_called && s.t >= stage1::frames(JAWS_AT) {
                s.boss_called = true;
                let spin = self.foes.rng.range(0.0, 360.0);
                self.boss = Some(Boss::new(spin));
                self.events.push(Event::BossArrive);
            }
        }
        let me = (self.player.x, self.player.y);
        let mut m = [(0.0, 0.0, false); 8];
        let n = self.mounts(&mut m);
        self.hazards.step(me, &m[..n]);
        let fire_ok = self.stage.as_ref().is_none_or(|s| s.fire_ok());
        self.foes.step(&mut self.hazards, me, fire_ok, &mut self.events);
        if let Some(b) = &mut self.boss {
            b.step(&mut self.hazards, me, &mut self.events);
        }
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
        self.boss_aftermath(first_event);
        if self.stage.is_some() {
            let r = self.style.tick();
            self.restyled(r);
        }
        self.invuln = self.invuln.saturating_sub(1);
        self.dip = self.dip.saturating_sub(1);
        self.slow = self.slow.saturating_sub(1);
    }

    /// What the boss's moments this tick do to the rest of the game: a broken phase
    /// takes its bullets with it (as shards), its death cancels every bullet and runs
    /// in slow motion, and once it's gone the stage is over.
    fn boss_aftermath(&mut self, from: usize) {
        for k in from..self.events.len() {
            let mut out = std::mem::take(&mut self.cancelled);
            let before = out.len();
            let event = self.events[k];
            match event {
                Event::BossBreak { timeout, .. } => {
                    self.hazards.cancel_owner(BOSS_ID, &mut out);
                    if !timeout {
                        self.stylish(style::Move::Break, STYLE_BREAK);
                    }
                }
                Event::BossFall => self.hazards.cancel_owner(BOSS_ID, &mut out),
                Event::BossDown { .. } => {
                    self.hazards.cancel_within(0.0, 0.0, f32::MAX, &mut out);
                    self.slow = JAWS_SLOWMO;
                }
                Event::BossGone => {
                    if let Some(s) = self.stage.as_mut().filter(|s| !s.over) {
                        s.over = true;
                        self.events.push(Event::StageEnd);
                        self.finish_stage();
                    }
                }
                Event::FinBreak { x, y, .. } => {
                    self.award(x, y, FIN_POINTS, Award::Part);
                    self.stylish(style::Move::Break, STYLE_BREAK);
                }

                _ => {}
            }
            let shards = out.len() - before;
            self.cancel_shards |= shards > 0;
            self.cancelled = out;
            // a phase over: its grade and bonus, then its bullets' shards
            if matches!(event, Event::BossBreak { .. } | Event::BossDown { .. } | Event::BossFall) {
                self.grade_phase();
                if let Event::BossDown { x, y } = event {
                    self.award(x, y, BOSS_KILL_POINTS, Award::BossKill);
                }
            }
            self.award_shards(shards);
        }
    }

    /// Score `n` points at `(x, y)` (Stage 1 only), and give any extends they earn.
    fn award(&mut self, x: f32, y: f32, n: u32, how: Award) {
        if self.stage.is_none() || n == 0 {
            return;
        }
        // the style rank multiplies what you earn by playing (not the grades and bonuses)
        let n = match how {
            Award::Kill | Award::Blank | Award::Part | Award::Shards | Award::Graze => (n as f32 * self.style.mult()).round() as u32,
            _ => n,
        };
        let extends = self.score.add(n);
        if how != Award::Graze {
            self.events.push(Event::Points { x, y, n, how });
        }
        for _ in 0..extends {
            // up to the most a player can carry (the bot's trials run on endless lives)
            if self.lives < MAX_LIVES {
                self.lives += 1;
            }
            self.events.push(Event::Extend);
        }
    }

    /// A move worth `base` style points (Stage 1 only).
    fn stylish(&mut self, m: style::Move, base: f32) {
        if self.stage.is_some() {
            let r = self.style.gain(m, base);
            self.restyled(r);
        }
    }

    /// The style rank changed (or not): say so.
    fn restyled(&mut self, r: Option<u8>) {
        if let Some(rank) = r {
            self.events.push(Event::Style { rank, up: self.style.rose });
        }
    }

    /// Cancelled bullets turned into shards that fly to you.
    fn award_shards(&mut self, n: usize) {
        let (x, y) = (self.player.x, self.player.y);
        self.award(x, y, n as u32 * SHARD_POINTS, Award::Shards);
    }

    /// The boss's phase just ended: grade it once, and pay its bonus.
    fn grade_phase(&mut self) {
        let Some(b) = &self.boss else { return };
        let p = b.phase;
        let i = p as usize - 1;
        let (Some(outcome), false) = (b.outcomes[i], self.score.grades[i].is_some()) else { return };
        let grade = score::phase_grade(outcome, b.clean[i], b.took[i], JAWS_PHASES[i].1 * TICK_HZ);
        self.score.grades[i] = Some(grade);
        let (x, y) = (b.x, b.y);
        self.award(x, y, score::phase_bonus(p, grade), Award::Grade(p, grade));
        // even a C shows its letter
        if score::phase_bonus(p, grade) == 0 && self.stage.is_some() {
            self.events.push(Event::Points { x, y, n: 0, how: Award::Grade(p, grade) });
        }
    }

    /// The stage is over: the no-miss and no-bomb bonuses, the rating and the rank.
    fn finish_stage(&mut self) {
        let Some(s) = &self.stage else { return };
        let (killed, spawned) = (s.killed, s.spawned);
        let destroyed = self.boss.as_ref().is_some_and(|b| b.outcomes[3] == Some(boss::Outcome::Broken));
        let grades = self.score.grades;
        let no_miss = if self.deaths == 0 { NO_MISS_BONUS } else { 0 };
        let no_bomb = if self.bombs_used == 0 { NO_BOMB_BONUS } else { 0 };
        // too late for an extend to help, so they don't earn one
        self.score.points += (no_miss + no_bomb) as u64;
        let rating = score::rating(killed, spawned, destroyed, &grades, self.deaths, self.bombs_used);
        self.score.tally = Some(Tally {
            killed,
            spawned,
            destroyed,
            grades,
            misses: self.deaths,
            bombs: self.bombs_used,
            grazes: self.grazes,
            no_miss,
            no_bomb,
            rating,
            rank: score::rank(rating),
            total: self.score.points,
        });
    }

    /// The boss is fighting a phase you just died or bombed in: no capture for it.
    fn spoil_capture(&mut self) {
        if let Some(b) = self.boss.as_mut().filter(|b| b.act == boss::Act::Fight) {
            b.clean[b.phase as usize - 1] = false;
        }
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
        self.spoil_capture();
        if let Some(s) = &mut self.stage {
            s.deaths[s.section] += 1;
        }
        self.lives = self.lives.saturating_sub(1);
        self.life = Life::Dead { t: 0 };
        let r = self.style.reset();
        self.restyled(r);
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
        if let Some(b) = &mut self.boss {
            b.bombed = [false; 3];
        }
        self.spoil_capture();
        self.invuln = self.invuln.max(BOMB_INVULN);
        self.invuln_blink = false;
        self.dip = BOMB_DIP;
        self.events.push(Event::Bomb { x, y });
        let r = self.style.drop(1);
        self.restyled(r);
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
        self.by = By::Bomb;
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
                Target::Boss(p) => {
                    if let Some(boss) = self.boss.as_mut().filter(|boss| !boss.bombed[p]) {
                        boss.bombed[p] = true;
                        self.hits_now.push(t);
                    }
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
        if let Some(b) = &self.boss {
            for p in b.targets() {
                self.boxes.push((Target::Boss(p), b.rect(p)));
            }
            // its tail is it too: shots and the beam stop on it and hurt the body
            if b.hittable() {
                for r in b.tail_boxes() {
                    self.boxes.push((Target::Boss(0), r));
                }
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
                    self.rush = (self.rush + if big { RUSH_FILL[5] } else { RUSH_FILL[3] }).min(SLAM_LINKS as f32);
                    self.boxes.retain(|b| b.0 != t);
                }
            }
            Target::Boss(p) => {
                let Some(b) = &mut self.boss else { return };
                let hurt = b.hurt(p, dmg, &mut self.events);
                if hurt != Hurt::Nothing {
                    self.rush = (self.rush + dmg * RUSH_BOSS).min(SLAM_LINKS as f32);
                    if self.by != By::Bomb && self.stage.is_some() {
                        let r = self.style.boss_damage(dmg, self.by == By::Beam);
                        self.restyled(r);
                    }
                }
                match hurt {
                    // a turret broke off: just it; the phase broke: the whole boss, till the next
                    Hurt::Broke if p > 0 => self.boxes.retain(|x| x.0 != t),
                    Hurt::Broke | Hurt::Died => self.boxes.retain(|x| !matches!(x.0, Target::Boss(_))),
                    Hurt::Hit | Hurt::Nothing => {}
                }
            }
            Target::Foe(i, _) if !self.foes.list[i].alive() => {}
            Target::Foe(i, p) => match self.foes.list[i].hurt(p, dmg) {
                Hurt::Died => self.kill(i),
                Hurt::Broke => {
                    let r = self.foes.list[i].rect(p);
                    let (x, y) = ((r.x0 + r.x1) * 0.5, (r.y0 + r.y1) * 0.5);
                    self.events.push(Event::PartBreak { x, y });
                    self.award(x, y, PART_POINTS, Award::Part);
                    self.stylish(style::Move::Break, STYLE_BREAK);
                    self.boxes.retain(|b| b.0 != t);
                }
                Hurt::Hit | Hurt::Nothing => {}
            },
        }
    }

    /// Enemy `i` is destroyed. A high-HP one takes its bullets with it, as shards.
    fn kill(&mut self, i: usize) {
        let e = &self.foes.list[i];
        let (id, kind, x, y) = (e.id, e.kind, e.x, e.y);
        self.events.push(Event::Kill { x, y, kind });
        self.fill_rush(RUSH_FILL[kind as usize]);
        let (n, blank) = score::kill_points(kind, (x - self.player.x).hypot(y - self.player.y));
        self.award(x, y, n, if blank { Award::Blank } else { Award::Kill });
        // style: what killed it (a bomb's kills earn none); a kill in a chain's
        // slowdown counts as the slam's
        let by = if self.slam.is_some() { By::Slam } else { self.by };
        let m = match by {
            By::Shot => Some(style::Move::Kill),
            By::Beam => Some(style::Move::BeamKill),
            By::Slam => Some(style::Move::SlamKill),
            By::Bomb => None,
        };
        if let Some(m) = m {
            self.stylish(m, STYLE_KILL[kind as usize]);
            if blank {
                self.stylish(style::Move::Blank, STYLE_BLANK);
            }
        }
        self.boxes.retain(|b| !matches!(b.0, Target::Foe(j, _) if j == i));
        if kind.heavy() {
            let mut out = std::mem::take(&mut self.cancelled);
            let before = out.len();
            self.hazards.cancel_owner(id, &mut out);
            let shards = out.len() - before;
            self.cancel_shards |= shards > 0;
            self.cancelled = out;
            self.award_shards(shards);
        }
        // scenery breaks into shards that fly to you
        if kind.scenery() {
            let n = WRECK_SHARDS[kind as usize - EnemyKind::Debris as usize];
            for k in 0..n {
                let (a, r) = (k as f32 * 2.4, (k as f32).sqrt() * 3.0);
                self.cancelled.push((x + a.cos() * r, y + a.sin() * r));
            }
            self.cancel_shards = true;
            self.award_shards(n as usize);
        }
        if let Some(s) = self.stage.as_mut().filter(|_| !kind.scenery()) {
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
                self.by = By::Shot;
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
        self.by = By::Beam;
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
        let mut grazed = 0;
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
                grazed += 1;
                self.events.push(Event::Graze { x: b.x[i], y: b.y[i] });
            }
        }
        self.award(px, py, grazed * GRAZE_POINTS, Award::Graze);
        for _ in 0..grazed {
            self.stylish(style::Move::Graze, STYLE_GRAZE);
        }
        // crashing into an enemy's body is as bad as a bullet, and so is the boss's laser
        if !safe {
            hit |= self.foes.list.iter().any(|e| e.alive() && !e.kind.scenery() && e.body().dist(px, py) < HITBOX_R);
            hit |= self.boss.as_ref().is_some_and(|b| b.crashes(px, py, HITBOX_R) || b.laser.is_some_and(|l| l.touches(px, py, HITBOX_R)));
        }
        if hit {
            // struck: the deathbomb window opens
            self.hits += 1;
            self.life = Life::Struck { left: DEATHBOMB };
            let r = self.style.drop(2);
            self.restyled(r);
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

    // ---------------------------------------------------------------- the score

    /// Stage 1 in the breather: nobody else about.
    fn quiet_stage() -> World {
        World::stage_at(1, Tuning::default(), SECTIONS[3].0 + 1.0)
    }

    fn points(w: &World) -> Vec<(u32, Award)> {
        w.events.iter().filter_map(|e| if let Event::Points { n, how, .. } = e { Some((*n, *how)) } else { None }).collect()
    }

    /// An enemy of `kind` parked at `(x, y)`; returns its index.
    fn parked(w: &mut World, kind: EnemyKind, x: f32, y: f32) -> usize {
        w.foes.add(kind, 0, enemy::Path::Drift { x, y, vy: 0.0, sway: 0.0 });
        let i = w.foes.list.len() - 1;
        (w.foes.list[i].x, w.foes.list[i].y) = (x, y);
        i
    }

    #[test]
    fn kills_score_and_point_blank_ones_score_double() {
        let mut w = quiet_stage();
        let (px, py) = (w.player.x, w.player.y);
        let far = parked(&mut w, EnemyKind::Gunship, px, py - 200.0);
        let near = parked(&mut w, EnemyKind::Gunship, px, py - (POINT_BLANK.0 - 10.0));
        w.kill(far);
        w.kill(near);
        let base = KILL_POINTS[EnemyKind::Gunship as usize];
        assert_eq!(points(&w), vec![(base, Award::Kill), (base * POINT_BLANK.1, Award::Blank)]);
        assert_eq!(w.score.points, (base + base * POINT_BLANK.1) as u64);
    }

    /// The style rank multiplies what kills earn; kills raise it (a bomb's don't); a
    /// hit drops it two ranks; the sandbox keeps none.
    #[test]
    fn the_style_rank_multiplies_kills_and_falls_when_hit() {
        let mut w = quiet_stage();
        let (px, py) = (w.player.x, w.player.y);
        w.style = style::Style::default();
        w.style.gain(style::Move::Break, 450.0);
        assert_eq!(w.style.rank(), 4, "S");
        let i = parked(&mut w, EnemyKind::Gunship, px, py - 200.0);
        let before = w.style.points;
        w.kill(i);
        let base = KILL_POINTS[EnemyKind::Gunship as usize];
        assert_eq!(points(&w), vec![((base as f32 * STYLE_MULT[4]).round() as u32, Award::Kill)]);
        assert!(w.style.points > before, "the kill raised it");
        let j = parked(&mut w, EnemyKind::Gunship, px, py - 200.0);
        w.by = By::Bomb;
        let before = w.style.points;
        w.kill(j);
        assert_eq!(w.style.points, before, "a bomb's kill earns no style");
        w.events.clear();
        w.debug_strike();
        w.tick(Controls::default());
        assert!(w.events.contains(&Event::Style { rank: 2, up: false }), "struck: S to B");
        let mut sandbox = World::new(1, Tuning::default());
        sandbox.stylish(style::Move::Kill, 500.0);
        assert_eq!(sandbox.style.points, 0.0, "the sandbox keeps no style");
    }

    #[test]
    fn a_big_kill_turns_its_bullets_into_shard_points() {
        let mut w = quiet_stage();
        let i = parked(&mut w, EnemyKind::Heavy, 100.0, 80.0);
        let id = w.foes.list[i].id;
        for k in 0..12 {
            w.hazards.bullets.push(bullets::Spawn { x: 20.0 + k as f32 * 10.0, y: 150.0, vx: 0.0, vy: 1.0, kind: bullets::Kind::Small, behavior: bullets::Behavior::Straight, owner: id, hue: Default::default() });
        }
        w.kill(i);
        assert!(points(&w).contains(&(12 * SHARD_POINTS, Award::Shards)), "{:?}", points(&w));
        assert!(w.cancel_shards);
    }

    #[test]
    fn grazes_score_quietly_and_the_sandbox_keeps_no_score() {
        let mut w = quiet_stage();
        w.award(0.0, 0.0, GRAZE_POINTS, Award::Graze);
        assert_eq!(w.score.points, GRAZE_POINTS as u64);
        assert!(points(&w).is_empty(), "no popup for a graze");
        let mut w = with_dummy(DummyKind::Still, 135.0, 100.0);
        w.award(10.0, 10.0, 1_000_000, Award::Kill);
        assert_eq!((w.score.points, w.lives), (0, LIVES));
    }

    #[test]
    fn extends_give_a_ship_once_each_up_to_the_most() {
        let mut w = quiet_stage();
        w.award(0.0, 0.0, EXTENDS[0] as u32, Award::Kill);
        assert_eq!(w.lives, LIVES + 1);
        assert_eq!(w.events.iter().filter(|e| **e == Event::Extend).count(), 1);
        w.award(0.0, 0.0, 10, Award::Kill);
        assert_eq!(w.lives, LIVES + 1, "once");
        w.lives = MAX_LIVES;
        w.award(0.0, 0.0, EXTENDS[1] as u32, Award::Kill);
        assert_eq!((w.lives, w.score.extends), (MAX_LIVES, 2), "never past the most");
        // the bot's trials fly on endless lives, which an extend leaves alone
        let mut w = quiet_stage();
        w.lives = u8::MAX;
        w.award(0.0, 0.0, EXTENDS[1] as u32 * 2, Award::Kill);
        assert_eq!(w.lives, u8::MAX);
    }

    #[test]
    fn each_boss_phase_is_graded_once_and_paid() {
        // broken at once without a death or a bomb: an S, double its capture
        let mut w = World::boss_at(1, Tuning::default(), 1);
        w.skip_section();
        assert_eq!(w.score.grades[0], Some(score::Grade::S));
        let bonus = score::phase_bonus(1, score::Grade::S);
        assert!(points(&w).contains(&(bonus, Award::Grade(1, score::Grade::S))));
        w.events.clear();
        for _ in 0..JAWS_BREAK + 30 {
            w.tick(Controls::default());
            assert!(!points(&w).iter().any(|p| matches!(p.1, Award::Grade(1, _))), "graded once");
            w.events.clear();
        }
        // a bomb in the phase costs the capture: a B
        let mut w = World::boss_at(1, Tuning::default(), 2);
        w.start_bomb();
        w.skip_section();
        assert_eq!(w.score.grades[1], Some(score::Grade::B));
        assert!(points(&w).contains(&(score::phase_bonus(2, score::Grade::B), Award::Grade(2, score::Grade::B))));
    }

    #[test]
    fn the_stage_end_pays_its_bonuses_and_ranks_the_run() {
        let mut w = World::boss_at(1, Tuning::default(), 4);
        w.skip_section();
        for _ in 0..JAWS_DEATH + 60 {
            if w.score.tally.is_some() {
                break;
            }
            w.tick(Controls::default());
            w.events.clear();
        }
        let t = w.score.tally.expect("the stage ended");
        assert!(t.destroyed);
        assert_eq!((t.no_miss, t.no_bomb), (NO_MISS_BONUS, NO_BOMB_BONUS));
        assert_eq!(t.grades, [None, None, None, Some(score::Grade::S)], "practice from phase 4");
        assert_eq!((t.total, t.rank), (w.score.points, score::rank(t.rating)));
        // the bonuses come too late for an extend
        let (lives, extends) = (w.lives, w.score.extends);
        w.score.points = EXTENDS[1] - 1;
        w.finish_stage();
        assert!(w.score.points > EXTENDS[1]);
        assert_eq!((w.lives, w.score.extends), (lives, extends));
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

    /// The boss's tail is it too: the beam stops on a tail plate and hurts the boss,
    /// where it used to pass straight through.
    #[test]
    fn the_beam_stops_on_the_bosss_tail_and_hurts_it() {
        let mut w = World::boss_at(7, Tuning::default(), 1);
        w.invincible = true;
        w.hazards.bullets.clear();
        let b = w.boss.as_mut().unwrap();
        b.y = 120.0;
        let plate = b.tail_boxes().nth(2).unwrap();
        let body = b.rect(0);
        assert!(plate.x1 < body.x0 || plate.x0 > body.x1, "this plate is clear of the body");
        let hp = b.hp;
        w.player.x = (plate.x0 + plate.x1) * 0.5;
        w.player.y = 300.0;
        for _ in 0..30 {
            w.tick(beam());
            w.hazards.bullets.clear();
        }
        let beam = w.beam.unwrap();
        assert_eq!(beam.hit, Some(Target::Boss(0)), "the beam stops on the tail");
        assert!(w.boss.as_ref().unwrap().hp < hp, "and hurts the boss");
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
