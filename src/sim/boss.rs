//! JAWSTHIC, Stage 1's boss: a great white carved from moon rock, a gothic cathedral
//! for a body (Jason's concept). It rises out of the black hole's disk fin first, then
//! fights in four phases, each with its share of a segmented health bar and a timer:
//!
//!   1. TEETH: aimed fans from its jaws over slow rings from its windows (30 s)
//!   2. FINS: its pectoral fin and its tail fin spin opposing spirals while its jaws
//!      fire aimed needle bursts; break a fin to silence its spiral (40 s)
//!   3. GLITCH: a curtain of rice with a lane drifting through it, sprinkles off its
//!      tail, and laser sweeps from its jaws behind a red warning line (40 s)
//!   4. EVENT HORIZON, at 15% health: a flower and rings that curve into the black
//!      hole, and aimed bursts (35 s); let it time out and the hole takes it
//!
//! A phase ends when its share of health is gone or its time runs out: its bullets
//! cancel into shards and it shakes the break off for a moment. Killed, it dies in a
//! chain of blasts walking up from its tail. It's all deterministic, so the bot can
//! look ahead.

use super::Event;
use super::enemy::Hurt;
use super::geom::Rect;
use super::pattern::{Emitter, Gun, Hazards};
use crate::tune::*;

/// The owner id its bullets carry (enemies count up from 1, so this never clashes).
pub const BOSS_ID: u32 = u32::MAX;

/// Where one of its guns fires from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Muzzle {
    /// Between the jaws (they open before it fires).
    Mouth,
    /// Lancet window `i`, 0 nearest the tail.
    Window(u8),
    /// The pectoral fin's joint: one of phase 2's turrets.
    Pectoral,
    /// The tail fin's root: the other turret.
    Tail,
    /// Under the belly.
    Belly,
}

/// One of its guns, by phase (`tune::JAWS_GUNS`). An `enraged` gun only fires once
/// the phase's turrets are all broken.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BossGun {
    pub muzzle: Muzzle,
    pub gun: Gun,
    pub enraged: bool,
}

pub const fn bgun(muzzle: Muzzle, gun: Gun) -> BossGun {
    BossGun { muzzle, gun, enraged: false }
}

/// A gun that opens up once the turrets are gone.
pub const fn enraged(muzzle: Muzzle, gun: Gun) -> BossGun {
    BossGun { muzzle, gun, enraged: true }
}

/// Where it is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    /// Rising out of the disk. It can't be hurt.
    Intro,
    Fight,
    /// Between phases, shaking the break off.
    Break,
    /// Destroyed: the chained blasts.
    Dying,
    /// Timed out in phase 4: the hole is swallowing it.
    Falling,
    Gone,
}

/// How a phase ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Broken,
    TimedOut,
}

/// A laser sweep, `t` frames in: it warns along `from` (degrees), then the beam sweeps
/// from `from` to `to`, out of the jaws at `(x, y)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Laser {
    pub t: u32,
    pub x: f32,
    pub y: f32,
    pub from: f32,
    pub to: f32,
}

impl Laser {
    pub fn firing(&self) -> bool {
        (JAWS_LASER.0..JAWS_LASER.0 + JAWS_LASER.1).contains(&self.t)
    }

    /// Where it points now (degrees): along its start through the warning, then
    /// sweeping, eased at both ends.
    pub fn angle(&self) -> f32 {
        let k = (self.t.saturating_sub(JAWS_LASER.0) as f32 / JAWS_LASER.1 as f32).min(1.0);
        self.from + (self.to - self.from) * k * k * (3.0 - 2.0 * k)
    }

    /// Does the beam touch a circle of radius `r` at `(px, py)`?
    pub fn touches(&self, px: f32, py: f32, r: f32) -> bool {
        if !self.firing() {
            return false;
        }
        let (s, c) = self.angle().to_radians().sin_cos();
        let (dx, dy) = (px - self.x, py - self.y);
        dx * c + dy * s > 0.0 && (dx * s - dy * c).abs() < JAWS_LASER.3 * 0.3 + r
    }
}

/// Health left when phase `p` (1-4) is broken (0: before the first, all of it).
pub fn floor(p: u8) -> f32 {
    if p >= 4 {
        return 0.0;
    }
    let gone: f32 = JAWS_PHASES[..p as usize].iter().map(|x| x.0).sum();
    JAWS_HP * (1.0 - gone)
}

/// Where it is `t` frames into the intro: (x, y, facing), and the disk's surface it's
/// still under (none once it's breached). The fin cuts across the disk left to right,
/// it dives and turns under the surface, breaches high, and settles.
pub fn intro_pose(t: u32) -> (f32, f32, f32, Option<f32>) {
    let [pass, under, peak, done] = JAWS_INTRO;
    let surface = HOLE_Y;
    let smooth = |k: f32| {
        let k = k.clamp(0.0, 1.0);
        k * k * (3.0 - 2.0 * k)
    };
    if t < pass {
        let k = t as f32 / pass as f32;
        (-20.0 + FIELD_W as f32 * k, surface + 36.0, 1.0, Some(surface))
    } else if t < under {
        let k = (t - pass) as f32 / (under - pass) as f32;
        let facing = if k < 0.5 { 1.0 } else { -1.0 };
        let x = if k < 0.5 { FIELD_W as f32 - 20.0 + 40.0 * k } else { JAWS_HOME.1 };
        (x, surface + 36.0 + 60.0 * smooth(k * 2.0), facing, Some(surface))
    } else if t < peak {
        let k = (t - under) as f32 / (peak - under) as f32;
        let ease = 1.0 - (1.0 - k) * (1.0 - k);
        let y = surface + 96.0 + (JAWS_Y - 18.0 - surface - 96.0) * ease;
        (JAWS_HOME.1, y, -1.0, (t + 6 < peak).then_some(surface))
    } else {
        let k = (t - peak) as f32 / (done - peak) as f32;
        (JAWS_HOME.1, JAWS_Y - 18.0 + 18.0 * smooth(k), -1.0, None)
    }
}

/// One of its attacks (see `JAWS_ATTACKS`): the guns that fire (by place in the
/// phase's list), its wind-up, how long it fires, its rest after, where it fires
/// from (px ahead of its home, px down), and whether it's the laser.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attack {
    pub guns: &'static [usize],
    pub wind: u32,
    pub fire: u32,
    pub rest: u32,
    pub at: (f32, f32),
    pub laser: bool,
}

pub const fn attack(guns: &'static [usize], fire: u32, at: (f32, f32)) -> Attack {
    Attack { guns, wind: JAWS_WIND, fire, rest: JAWS_REST, at, laser: false }
}

/// Where it is in an attack.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Beat {
    /// Winding up, 0..1 of the way.
    Wind(f32),
    /// Firing, this many frames in.
    Fire(u32),
    /// Resting after it.
    Rest,
}

/// How close its guns are to firing, 0..1 (the art glows them red before they do):
/// the windows, the jaws, and the pectoral fin, the belly and the tail (these only
/// as an attack winds up).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Charges {
    pub windows: [f32; 4],
    pub mouth: f32,
    pub others: [f32; 3],
}

/// A turret's health and hit flash.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fin {
    pub hp: f32,
    pub flash: u8,
    flash_cd: u8,
}

impl Fin {
    pub fn alive(&self) -> bool {
        self.hp > 0.0
    }
}

#[derive(Clone, Debug)]
struct Gunner {
    muzzle: Muzzle,
    e: Emitter,
    enraged: bool,
}

#[derive(Clone, Debug)]
pub struct Boss {
    pub act: Act,
    /// Frames into the act.
    pub t: u32,
    /// Frames since it arrived (its sway and bob run on this).
    pub age: u32,
    /// The phase it's in or just broke, 1-4 (0 in the intro).
    pub phase: u8,
    /// Its body's pivot, playfield px.
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// 1 facing right, -1 facing left.
    pub facing: f32,
    /// Frames left in a turn (0: not turning).
    pub turn: u32,
    /// The whole fight's health; each phase breaks at its floor.
    pub hp: f32,
    /// The body's hit flash, and frames before it may flash again.
    pub flash: u8,
    flash_cd: u8,
    /// Phase 2's turrets: the pectoral fin and the tail fin. (They hang below and
    /// behind the body, so shots can reach them; the dorsal fin is just stone.)
    pub fins: [Fin; 2],
    /// The current bomb has hurt the body, the pectoral turret, the tail fin.
    pub bombed: [bool; 3],
    /// How far its jaw has dropped, 0 shut to 1 wide.
    pub jaw: f32,
    /// The tail's wave, radians (the body swims on it too).
    pub swim: f32,
    /// Where you are, for its eye to follow.
    pub look: (f32, f32),
    /// The damage it's taken lately; frames into a flinch (0: not flinching), and
    /// before it may flinch again.
    pain: f32,
    pub flinch: u32,
    flinch_cd: u32,
    /// Frames into a lunge (0: not lunging), when the next comes, and when it next
    /// chomps (frames into the act).
    pub lunge: u32,
    next_lunge: u32,
    next_chomp: u32,
    pub laser: Option<Laser>,
    /// The attack it's on (its place in the phase's cycle), and frames into it.
    pub attack: usize,
    beat: u32,
    next_turn: u32,
    guns: Vec<Gunner>,
    /// Where its rings and spirals start turning from (seeded per run).
    spin: f32,
    /// How each phase ended, whether you got through it without a death or a bomb,
    /// and how many frames it took.
    pub outcomes: [Option<Outcome>; 4],
    pub clean: [bool; 4],
    pub took: [u32; 4],
}

impl Boss {
    /// It arrives: the intro begins.
    pub fn new(spin: f32) -> Boss {
        let (x, y, facing, _) = intro_pose(0);
        let fin = Fin { hp: JAWS_FIN_HP, flash: 0, flash_cd: 0 };
        Boss {
            act: Act::Intro,
            t: 0,
            age: 0,
            phase: 0,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            facing,
            turn: 0,
            hp: JAWS_HP,
            flash: 0,
            flash_cd: 0,
            fins: [fin; 2],
            bombed: [false; 3],
            jaw: 0.0,
            swim: 0.0,
            look: (0.0, 0.0),
            pain: 0.0,
            flinch: 0,
            flinch_cd: 0,
            lunge: 0,
            next_lunge: 0,
            next_chomp: 0,
            laser: None,
            attack: 0,
            beat: 0,
            next_turn: 0,
            guns: Vec::new(),
            spin,
            outcomes: [None; 4],
            clean: [true; 4],
            took: [0; 4],
        }
    }

    /// Straight into phase `p` (boss practice), in position, at that phase's health.
    pub fn at_phase(spin: f32, p: u8) -> Boss {
        let mut b = Boss::new(spin);
        let p = p.clamp(1, 4);
        (b.x, b.y, b.facing) = (JAWS_HOME.1, JAWS_Y, -1.0);
        b.hp = if p == 1 { JAWS_HP } else { floor(p - 1) };
        for k in 1..p {
            b.outcomes[k as usize - 1] = Some(Outcome::Broken);
        }
        b.begin(p);
        b
    }

    fn begin(&mut self, p: u8) {
        self.phase = p;
        self.act = Act::Fight;
        self.t = 0;
        self.laser = None;
        self.turn = 0;
        let (_, _, _, turns) = JAWS_PHASES[p as usize - 1];
        self.next_turn = if turns > 0 { turns } else { u32::MAX };
        (self.attack, self.beat) = (0, 0);
        self.lunge = 0;
        self.next_lunge = self.later(JAWS_LUNGE_EVERY, 1);
        self.next_chomp = self.later((JAWS_CHOMP.0, JAWS_CHOMP.1), 2);
        let spin = self.spin + 37.0 * p as f32;
        self.guns = JAWS_GUNS[p as usize - 1].iter().map(|g| Gunner { muzzle: g.muzzle, e: Emitter::new(g.gun, BOSS_ID, spin), enraged: g.enraged }).collect();
    }

    /// Facing-right px from its pivot to playfield px, its body's swim bend and all.
    pub fn at(&self, (lx, ly): (f32, f32)) -> (f32, f32) {
        (self.x + self.facing * lx, self.y + ly + self.bend(lx))
    }

    /// How far its body at `lx` (px from its pivot, facing right) is bent down now
    /// (up if negative): a wave rolling from its snout, which sways a little, back to
    /// where its tail joins, which swings more, on the tail's beat.
    pub fn bend(&self, lx: f32) -> f32 {
        let (head, root, k) = JAWS_BODY_WAVE;
        let snout = JAWS_MOUTH.0 + 22.0;
        let t = ((snout - lx) / (snout - JAWS_TAIL_ROOT.0)).clamp(0.0, 1.0);
        (head + (root - head) * t * t) * (self.swim + (lx - JAWS_TAIL_ROOT.0) * k).sin()
    }

    /// `t` frames from now plus a share of `extra` picked by the fight's seed and `n`.
    fn later(&self, (every, extra): (u32, u32), n: u32) -> u32 {
        let h = (self.spin.to_bits() ^ (self.t + n).wrapping_mul(0x9E37_79B9) ^ (self.phase as u32).wrapping_mul(0x85EB_CA6B)).wrapping_mul(0xC2B2_AE35);
        self.t + every + (h >> 8) % extra.max(1)
    }

    /// Mid-lunge: drawing back, striking (biting), or easing home.
    fn lunge_offset(&self) -> f32 {
        let (back, wind, reach, _, strike) = JAWS_LUNGE;
        let t = self.lunge as f32;
        let (w, s) = (wind as f32, strike as f32);
        if self.lunge == 0 {
            0.0
        } else if t <= w {
            -back * (t / w).powi(2)
        } else if t <= w + s {
            let u = (t - w) / s;
            -back + (reach + back) * (1.0 - (1.0 - u) * (1.0 - u))
        } else {
            reach
        }
    }

    /// Biting: the strike of a lunge, and a moment after.
    pub fn biting(&self) -> bool {
        let (_, wind, _, _, strike) = JAWS_LUNGE;
        self.lunge > wind && self.lunge <= wind + strike + 8
    }

    pub fn muzzle(&self, m: Muzzle) -> (f32, f32) {
        match m {
            Muzzle::Mouth => self.at((JAWS_MOUTH.0 + JAWS_MOUTH_OPEN.0 * self.jaw, JAWS_MOUTH.1 + JAWS_MOUTH_OPEN.1 * self.jaw)),
            Muzzle::Window(i) => self.at((JAWS_WINDOWS[i as usize % JAWS_WINDOWS.len()], JAWS_WINDOW_Y)),
            Muzzle::Pectoral => self.at(JAWS_PECTORAL_AT),
            Muzzle::Belly => self.at(JAWS_BELLY),
            Muzzle::Tail => {
                let j = self.tail()[JAWS_TAIL.len()];
                (j.0, j.1)
            }
        }
    }

    /// The tail's joints, root first and the tail fin's last: where each hangs
    /// (playfield px) and its angle (radians, as if it faced right; the art mirrors it).
    /// A wave runs down it, growing toward the fin; rising, it trails low.
    pub fn tail(&self) -> [(f32, f32, f32); 6] {
        let mut out = [(0.0, 0.0, 0.0); 6];
        let (mut lx, mut ly) = JAWS_TAIL_ROOT;
        let lean = (self.vy * 0.05).clamp(-0.1, 0.1);
        // it hangs where the body's bent to, and leaves it the way the body runs
        let root = self.bend(lx);
        let mut wave = (self.bend(lx + 1.0) - root).atan();
        for (i, joint) in out.iter_mut().enumerate() {
            wave += JAWS_BEND[i] * (self.swim - i as f32 * 0.6).sin() + lean * 0.3;
            // the plate's own slope as drawn, the whole tail's sweep, the wave
            let ang = JAWS_TAIL_ART[i] + JAWS_TAIL_TILT + wave;
            let (x, y) = (self.x + self.facing * lx, self.y + ly + root);
            *joint = (x, y, ang);
            if let Some(&(len, _)) = JAWS_TAIL.get(i) {
                lx -= len * ang.cos();
                ly -= len * ang.sin();
            }
        }
        out
    }

    fn boxed(&self, (cx, cy, w, h): (f32, f32, f32, f32)) -> Rect {
        let (x, y) = self.at((cx, cy));
        Rect::centered(x, y, w, h)
    }

    /// Part `p`'s hurtbox: 0 the body, 1 the pectoral turret, 2 the tail fin.
    pub fn rect(&self, p: usize) -> Rect {
        match p {
            1 => self.boxed(JAWS_PECTORAL),
            2 => {
                // the tail fin swings with the tail: its middle, back along its plate
                let (x, y, a) = self.tail()[JAWS_TAIL.len()];
                let (back, w, h) = JAWS_TAIL_FIN;
                Rect::centered(x - self.facing * back * a.cos(), y - back * a.sin(), w, h)
            }
            _ => self.boxed(JAWS_BODY),
        }
    }

    /// Shots can hurt it: fighting, and below the top dead zone.
    pub fn hittable(&self) -> bool {
        self.act == Act::Fight && self.y >= DEAD_ZONE
    }

    /// The parts shots can hit now, weak points first.
    pub fn targets(&self) -> impl Iterator<Item = usize> + '_ {
        let fins = self.phase == 2;
        [1usize, 2].into_iter().filter(move |&p| fins && self.fins[p - 1].alive()).chain(std::iter::once(0)).filter(move |_| self.hittable())
    }

    /// Its tail, which shots and the beam hit as its body: each plate and (outside
    /// phase 2, where it's a turret of its own) the tail fin.
    pub fn tail_boxes(&self) -> impl Iterator<Item = Rect> + '_ {
        let fin = (self.phase != 2).then(|| self.rect(2));
        self.crash_boxes().into_iter().skip(1).chain(fin)
    }

    /// What you crash into: its body (a little inside its hurtbox) and each tail plate.
    pub fn crash_boxes(&self) -> [Rect; 6] {
        let (cx, cy, w, h) = JAWS_BODY;
        let j = self.tail();
        let mut out = [self.boxed((cx, cy, w * BODY_SHRINK, h * BODY_SHRINK)); 6];
        for i in 0..JAWS_TAIL.len() {
            let (len, h) = JAWS_TAIL[i];
            out[i + 1] = Rect::centered((j[i].0 + j[i + 1].0) * 0.5, (j[i].1 + j[i + 1].1) * 0.5, len, h * 0.6);
        }
        out
    }

    /// Would a hitbox of radius `r` at `(px, py)` crash into its body or its tail?
    pub fn crashes(&self, px: f32, py: f32, r: f32) -> bool {
        matches!(self.act, Act::Fight | Act::Break) && self.crash_boxes().iter().any(|b| b.dist(px, py) < r)
    }

    /// The disk's surface it's under, so it only shows above it (rising out of the
    /// hole, or falling back in).
    pub fn surface(&self) -> Option<f32> {
        match self.act {
            Act::Intro => intro_pose(self.t).3,
            Act::Falling => Some(HOLE_Y),
            _ => None,
        }
    }

    /// How much of it the death's blasts have blown away: the tail fin and plates from
    /// the back (6 of them), then the dorsal fin, the pectoral fins, ..., all of it.
    pub fn blown(&self) -> usize {
        if self.act != Act::Dying {
            return 0;
        }
        JAWS_BLASTS.iter().filter(|&&f| f < self.t).count()
    }

    /// Where the death's blast `k` goes off: up the tail, the fins, the flank, the head.
    pub fn blast_at(&self, k: usize) -> (f32, f32) {
        let j = self.tail();
        match k {
            0 => (j[5].0, j[5].1),
            1..=5 => (j[5 - k].0, j[5 - k].1),
            6 => self.at(JAWS_DORSAL_CORE),
            7 => self.at(JAWS_PECTORAL_AT),
            8 => self.at((JAWS_WINDOWS[1], JAWS_WINDOW_Y)),
            9 => self.at(JAWS_EYE),
            _ => self.at((JAWS_BODY.0, JAWS_BODY.1)),
        }
    }

    /// Take a hit on part `p` (0 the body, 1 the pectoral turret, 2 the tail fin).
    pub fn hurt(&mut self, p: usize, dmg: f32, events: &mut Vec<Event>) -> Hurt {
        if self.act != Act::Fight {
            return Hurt::Nothing;
        }
        self.pain += dmg;
        if p > 0 {
            let r = self.rect(p);
            let fin = &mut self.fins[p - 1];
            if !fin.alive() || self.phase != 2 {
                return Hurt::Nothing;
            }
            fin.hp -= dmg;
            if fin.flash_cd == 0 {
                (fin.flash, fin.flash_cd) = ENEMY_FLASH;
            }
            if fin.alive() {
                return Hurt::Hit;
            }
            fin.hp = 0.0;
            events.push(Event::FinBreak { x: (r.x0 + r.x1) * 0.5, y: (r.y0 + r.y1) * 0.5, fin: p as u8 - 1 });
            self.pain = f32::MAX;
            // breaking a turret tears a chunk out of the phase's health
            let (hi, lo) = (floor(self.phase - 1), floor(self.phase));
            self.hp = (self.hp - (hi - lo) * JAWS_FIN_BLOW).max(lo);
            if self.hp <= lo {
                self.end(Outcome::Broken, events);
            }
            return Hurt::Broke;
        }
        let floor = floor(self.phase);
        self.hp = (self.hp - dmg * JAWS_PHASES[self.phase as usize - 1].2).max(floor);
        if self.flash_cd == 0 {
            (self.flash, self.flash_cd) = ENEMY_FLASH;
        }
        if self.hp > floor {
            return Hurt::Hit;
        }
        self.end(Outcome::Broken, events);
        if self.act == Act::Dying { Hurt::Died } else { Hurt::Broke }
    }

    /// The phase is over, broken or timed out.
    fn end(&mut self, how: Outcome, events: &mut Vec<Event>) {
        let p = self.phase;
        self.outcomes[p as usize - 1] = Some(how);
        self.took[p as usize - 1] = self.t;
        self.hp = floor(p);
        self.laser = None;
        self.turn = 0;
        self.t = 0;
        let (x, y) = (self.x, self.y);
        match (p, how) {
            (4, Outcome::Broken) => {
                self.act = Act::Dying;
                events.push(Event::BossDown { x, y });
            }
            (4, Outcome::TimedOut) => {
                self.act = Act::Falling;
                events.push(Event::BossFall);
            }
            _ => {
                self.act = Act::Break;
                events.push(Event::BossBreak { x, y, timeout: how == Outcome::TimedOut });
            }
        }
    }

    /// How close each of its guns is to firing (the telegraph): as an attack winds
    /// up, all its muzzles glow brighter and brighter; as it fires, each window and
    /// the jaws glow before each shot.
    pub fn charges(&self) -> Charges {
        let mut c = Charges::default();
        if self.act != Act::Fight || self.turn > 0 {
            return c;
        }
        let Some((a, beat)) = self.attack_now() else { return c };
        for k in self.firing(a) {
            let g = &self.guns[k];
            let charge = match beat {
                Beat::Wind(w) => w,
                Beat::Fire(_) => 1.0 - (g.e.until_next() as f32 / CHARGE_FRAMES as f32).min(1.0),
                Beat::Rest => 0.0,
            };
            match g.muzzle {
                Muzzle::Window(i) => c.windows[i as usize % 4] = c.windows[i as usize % 4].max(charge),
                Muzzle::Mouth => c.mouth = c.mouth.max(charge),
                Muzzle::Pectoral if matches!(beat, Beat::Wind(_)) => c.others[0] = charge,
                Muzzle::Belly if matches!(beat, Beat::Wind(_)) => c.others[1] = charge,
                Muzzle::Tail if matches!(beat, Beat::Wind(_)) => c.others[2] = charge,
                _ => {}
            }
        }
        match beat {
            Beat::Wind(w) if a.laser => c.mouth = w,
            _ if self.laser.is_some() => c.mouth = 1.0,
            _ => {}
        }
        c
    }

    /// The attack it's on and where it is in it (fighting only).
    pub fn attack_now(&self) -> Option<(&'static Attack, Beat)> {
        if self.act != Act::Fight {
            return None;
        }
        let a = &JAWS_ATTACKS[self.phase as usize - 1][self.attack];
        let beat = if self.beat < a.wind {
            Beat::Wind(self.beat as f32 / a.wind as f32)
        } else if self.beat < a.wind + a.fire {
            Beat::Fire(self.beat - a.wind)
        } else {
            Beat::Rest
        };
        Some((a, beat))
    }

    /// The guns attack `a` fires that can fire now: not a broken fin's, and the
    /// enraged ones only once both fins are gone.
    fn firing(&self, a: &Attack) -> impl Iterator<Item = usize> + '_ {
        let bare = self.fins.iter().all(|f| !f.alive());
        a.guns.iter().copied().filter(move |&k| {
            let g = &self.guns[k];
            (!g.enraged || bare)
                && match g.muzzle {
                    Muzzle::Pectoral => self.fins[0].alive(),
                    Muzzle::Tail if self.phase == 2 => self.fins[1].alive(),
                    _ => true,
                }
        })
    }

    /// An attack's rest: shorter past half the phase's health.
    fn rest_of(&self, a: &Attack) -> u32 {
        let (hi, lo) = (if self.phase == 1 { JAWS_HP } else { floor(self.phase - 1) }, floor(self.phase));
        if (self.hp - lo) / (hi - lo) < 0.5 { (a.rest as f32 * JAWS_HURRY) as u32 } else { a.rest }
    }

    /// Resting between attacks, with at least `frames` of the rest still to go.
    fn resting(&self, frames: u32) -> bool {
        self.attack_now().is_some_and(|(a, beat)| beat == Beat::Rest && a.wind + a.fire + self.rest_of(a) >= self.beat + frames)
    }

    /// Seconds left in the phase (for the HUD's timer).
    pub fn time_left(&self) -> f32 {
        if self.act != Act::Fight {
            return 0.0;
        }
        let secs = JAWS_PHASES[self.phase as usize - 1].1;
        (secs * TICK_HZ).saturating_sub(self.t) as f32 / TICK_HZ as f32
    }

    /// The health bar: how full each phase's segment is, 0..1.
    pub fn bar(&self) -> [f32; 4] {
        let mut out = [0.0; 4];
        for (p, seg) in out.iter_mut().enumerate() {
            let (hi, lo) = (if p == 0 { JAWS_HP } else { floor(p as u8) }, floor(p as u8 + 1));
            *seg = ((self.hp - lo) / (hi - lo)).clamp(0.0, 1.0);
        }
        out
    }

    /// Ease toward its patrol: a slow sway across its home, a bob; fighting, toward
    /// where its attack fires from (resting, where the next one will), swaying less
    /// while it fires.
    fn drift(&mut self) {
        let home = if self.facing > 0.0 { JAWS_HOME.0 } else { JAWS_HOME.1 };
        let ((ax, ay), sway) = match self.attack_now() {
            Some((_, Beat::Rest)) => {
                let cycle = JAWS_ATTACKS[self.phase as usize - 1];
                (cycle[(self.attack + 1) % cycle.len()].at, 1.0)
            }
            Some((a, Beat::Fire(_))) => (a.at, 0.35),
            Some((a, Beat::Wind(_))) => (a.at, 1.0),
            None => ((0.0, 0.0), 1.0),
        };
        let a = self.age as f32 * std::f32::consts::TAU;
        let tx = home + self.facing * ax + sway * JAWS_SWAY.0 * (a / JAWS_SWAY.1 as f32).sin();
        let ty = JAWS_Y + ay + JAWS_BOB.0 * (a / JAWS_BOB.1 as f32).sin();
        self.x += (tx - self.x) * JAWS_FOLLOW;
        self.y += (ty - self.y) * JAWS_FOLLOW;
    }

    /// Halfway through a turn it faces the other way; it shifts so the whole fish
    /// stays about where it was.
    fn turning(&mut self) {
        if self.turn == 0 {
            return;
        }
        self.turn -= 1;
        if self.turn == JAWS_TURN / 2 {
            self.x -= JAWS_TURN_SHIFT * self.facing;
            self.facing = -self.facing;
        }
    }

    /// Frames before its mouth next fires (or it's firing now: 0): only in an attack
    /// that fires from it.
    fn mouth_due(&self) -> u32 {
        if self.laser.is_some() {
            return 0;
        }
        let Some((a, beat)) = self.attack_now() else { return u32::MAX };
        let mouth: Vec<usize> = self.firing(a).filter(|&k| self.guns[k].muzzle == Muzzle::Mouth).collect();
        if mouth.is_empty() && !a.laser {
            return u32::MAX;
        }
        match beat {
            Beat::Wind(_) => a.wind - self.beat + mouth.iter().map(|&k| self.guns[k].e.gun.delay).min().unwrap_or(0),
            Beat::Fire(_) => mouth.iter().map(|&k| if self.guns[k].e.since < 12 { 0 } else { self.guns[k].e.until_next() }).min().unwrap_or(u32::MAX),
            Beat::Rest => u32::MAX,
        }
    }

    fn jaw_wanted(&self) -> f32 {
        let open = match self.act {
            Act::Intro => (JAWS_INTRO[2] + 8..JAWS_INTRO[3]).contains(&self.t),
            Act::Fight => self.turn == 0 && self.mouth_due() <= JAWS_JAW_LEAD,
            Act::Break => self.t < 40,
            Act::Dying | Act::Falling => true,
            Act::Gone => false,
        };
        if open || self.flinch > 0 && self.flinch < 10 {
            return 1.0;
        }
        if self.act != Act::Fight {
            return 0.0;
        }
        // a lunge: jaws open as it draws back, snapping shut as it strikes
        let (_, wind, _, _, strike) = JAWS_LUNGE;
        if self.lunge > 0 && self.lunge <= wind + strike {
            return if self.lunge <= wind + strike / 2 { 1.0 } else { 0.0 };
        }
        // between attacks, two quick snaps
        let (_, _, wide, snap) = JAWS_CHOMP;
        let since = self.t.wrapping_sub(self.next_chomp);
        if since < 4 * snap && (since / snap) % 2 == 0 { wide } else { 0.0 }
    }

    /// Its body: chomping and lunging between attacks, flinching when hit hard.
    fn behave(&mut self, events: &mut Vec<Event>) {
        let (keep, hard, back, up, rest) = JAWS_FLINCH;
        self.flinch_cd = self.flinch_cd.saturating_sub(1);
        if self.flinch > 0 {
            self.flinch += 1;
            if self.flinch > 24 {
                self.flinch = 0;
            }
        }
        if self.pain > hard && self.flinch_cd == 0 && matches!(self.act, Act::Fight | Act::Break) {
            // it recoils (not while it holds still for its laser), and its tail lashes
            (self.flinch, self.flinch_cd, self.pain) = (1, rest, 0.0);
            if self.laser.is_none() {
                self.x -= self.facing * back;
                self.y -= up;
            }
            self.swim += 1.2;
        }
        self.pain = if self.pain == f32::MAX { 0.0 } else { self.pain * keep };
        if self.act != Act::Fight {
            return;
        }
        let (_, wind, reach, drop, strike) = JAWS_LUNGE;
        let quiet = self.turn == 0 && self.laser.is_none();
        // a chomp waits for the mouth to be quiet
        let chomp = 4 * JAWS_CHOMP.3;
        if self.t == self.next_chomp && (!quiet || self.mouth_due() < chomp + JAWS_JAW_LEAD) {
            self.next_chomp += 30;
        } else if self.t >= self.next_chomp + chomp {
            self.next_chomp = self.later((JAWS_CHOMP.0, JAWS_CHOMP.1), 2);
        }
        if self.lunge == 0 {
            if self.t >= self.next_lunge && quiet && self.resting(JAWS_LUNGE.1) {
                self.lunge = 1;
            }
            return;
        }
        // drawing back, then striking forward and down, then home (the drift eases it)
        let before = self.lunge_offset();
        self.lunge += 1;
        let d = self.lunge_offset() - before;
        self.x += self.facing * d;
        self.y += d / reach.max(1.0) * drop;
        if self.lunge == wind + strike && self.mouth_due() > JAWS_JAW_LEAD {
            let (x, y) = self.muzzle(Muzzle::Mouth);
            events.push(Event::BossBite { x, y });
        }
        if self.lunge > wind + strike + 40 {
            self.lunge = 0;
            self.next_lunge = self.later(JAWS_LUNGE_EVERY, 1);
        }
    }

    /// One frame.
    pub fn step(&mut self, hz: &mut Hazards, player: (f32, f32), events: &mut Vec<Event>) {
        if self.act == Act::Gone {
            return;
        }
        self.t += 1;
        self.age += 1;
        self.flash = self.flash.saturating_sub(1);
        self.flash_cd = self.flash_cd.saturating_sub(1);
        for f in &mut self.fins {
            f.flash = f.flash.saturating_sub(1);
            f.flash_cd = f.flash_cd.saturating_sub(1);
        }
        let (x0, y0) = (self.x, self.y);
        self.look = player;
        match self.act {
            Act::Intro => {
                let (x, y, facing, _) = intro_pose(self.t);
                (self.x, self.y, self.facing) = (x, y, facing);
                if self.t + 6 == JAWS_INTRO[2] {
                    events.push(Event::BossBreach { x, y: HOLE_Y });
                }
                if self.t == JAWS_INTRO[2] + 8 {
                    events.push(Event::BossRoar { x, y });
                }
                if self.t >= JAWS_INTRO[3] {
                    self.begin(1);
                    events.push(Event::BossPhase(1));
                }
            }
            Act::Fight => self.fight(hz, player, events),
            Act::Break => {
                self.drift();
                if self.t >= JAWS_BREAK {
                    self.begin(self.phase + 1);
                    events.push(Event::BossPhase(self.phase));
                }
            }
            Act::Dying => {
                if let Some(k) = JAWS_BLASTS.iter().position(|&f| f + 1 == self.t) {
                    let (x, y) = self.blast_at(k);
                    events.push(Event::BossBlast { x, y, last: k + 1 == JAWS_BLASTS.len() });
                }
                if self.t >= JAWS_DEATH {
                    self.act = Act::Gone;
                    events.push(Event::BossGone);
                }
            }
            Act::Falling => {
                // dragged to the hole and down under its disk
                let k = (self.t as f32 / JAWS_FALL as f32).min(1.0);
                self.x += (HOLE_X - self.x) * 0.03;
                self.y += 0.1 + 1.4 * k * k;
                if self.t >= JAWS_FALL {
                    self.act = Act::Gone;
                    events.push(Event::BossGone);
                }
            }
            Act::Gone => {}
        }
        self.behave(events);
        (self.vx, self.vy) = (self.x - x0, self.y - y0);
        self.jaw += (self.jaw_wanted() - self.jaw) * JAWS_JAW_SPEED;
        let pace = if self.turn > 0 || self.act == Act::Falling { JAWS_SWIM.1 } else { JAWS_SWIM.0 };
        self.swim += pace + self.vx.abs().min(2.0) * 0.04;
    }

    fn fight(&mut self, hz: &mut Hazards, player: (f32, f32), events: &mut Vec<Event>) {
        let (_, secs, _, turns) = JAWS_PHASES[self.phase as usize - 1];
        if self.t >= secs * TICK_HZ {
            self.end(Outcome::TimedOut, events);
            return;
        }
        self.lase(events);
        // one attack at a time: it winds up, fires, rests, and on to the next
        let cycle = JAWS_ATTACKS[self.phase as usize - 1];
        self.beat += 1;
        let a = &cycle[self.attack];
        if self.beat >= a.wind + a.fire + self.rest_of(a) {
            (self.attack, self.beat) = ((self.attack + 1) % cycle.len(), 0);
        }
        let a = &cycle[self.attack];
        if self.beat == a.wind {
            // its first shot: its guns start afresh (their delays count from now)
            let spin = self.spin + 37.0 * self.phase as f32 + 53.0 * self.attack as f32;
            for &k in a.guns {
                self.guns[k].e = Emitter::new(self.guns[k].e.gun, BOSS_ID, spin);
            }
            if a.laser {
                self.start_laser(player, events);
            }
        }
        // now and then it turns round, as a rest begins
        if self.t >= self.next_turn && self.turn == 0 && self.lunge == 0 && self.resting(JAWS_TURN + 2) {
            self.turn = JAWS_TURN;
            self.next_turn = self.t + turns;
            events.push(Event::BossTurn);
        }
        self.turning();
        // it holds still while it fires the laser
        if self.laser.is_none() {
            self.drift();
        }
        let Some((a, Beat::Fire(_))) = self.attack_now() else { return };
        if self.turn > 0 {
            return;
        }
        let guns: Vec<usize> = self.firing(a).collect();
        for k in guns {
            let (x, y) = self.muzzle(self.guns[k].muzzle);
            hz.fire_gun(&mut self.guns[k].e, player, (x, y, true));
        }
    }

    /// Phase 3's laser attack: it warns along a line just beside you, on the open
    /// side, then sweeps through where you were toward your nearer wall. The way out
    /// is back across the warning line, into the open.
    fn start_laser(&mut self, player: (f32, f32), events: &mut Vec<Event>) {
        let (mx, my) = self.at((JAWS_MOUTH.0 + JAWS_MOUTH_OPEN.0, JAWS_MOUTH.1 + JAWS_MOUTH_OPEN.1));
        let aim = (player.1 - my).atan2(player.0 - mx).to_degrees();
        // right of the jaws (under 90 degrees), it sweeps right, toward that wall
        let dir = if aim < 90.0 { -1.0 } else { 1.0 };
        self.laser = Some(Laser { t: 0, x: mx, y: my, from: aim - dir * 6.0, to: aim + dir * (JAWS_LASER.2 - 6.0) });
        events.push(Event::LaserWarn);
    }

    fn lase(&mut self, events: &mut Vec<Event>) {
        if let Some(l) = &mut self.laser {
            l.t += 1;
            if l.t == JAWS_LASER.0 {
                events.push(Event::LaserFire);
            }
            if l.t >= JAWS_LASER.0 + JAWS_LASER.1 {
                self.laser = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(b: &mut Boss, frames: u32, hz: &mut Hazards, ev: &mut Vec<Event>) {
        for _ in 0..frames {
            b.step(hz, (135.0, 300.0), ev);
        }
    }

    #[test]
    fn the_phases_split_the_health_bar() {
        assert_eq!(floor(4), 0.0);
        assert!((floor(3) - JAWS_HP * 0.15).abs() < 1.0, "phase 4 starts at 15%: {}", floor(3));
        let shares: f32 = JAWS_PHASES.iter().map(|p| p.0).sum();
        assert!((shares - 1.0).abs() < 1e-4);
        let b = Boss::at_phase(0.0, 3);
        assert_eq!(b.bar(), [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn it_cant_be_hurt_until_the_intro_is_over() {
        let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
        let mut b = Boss::new(0.0);
        assert_eq!(b.hurt(0, 100.0, &mut ev), Hurt::Nothing);
        assert!(!b.hittable());
        run(&mut b, JAWS_INTRO[3], &mut hz, &mut ev);
        assert_eq!((b.act, b.phase), (Act::Fight, 1));
        assert!(ev.contains(&Event::BossPhase(1)));
        assert!(b.hittable());
        assert!(matches!(b.hurt(0, 100.0, &mut ev), Hurt::Hit));
        assert!(hz.bullets.len() == 0, "nothing fired in the intro");
    }

    #[test]
    fn a_phase_breaks_at_its_floor_cancels_and_moves_on() {
        let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
        let mut b = Boss::at_phase(0.0, 1);
        run(&mut b, 200, &mut hz, &mut ev);
        assert!(hz.bullets.len() > 0, "phase 1 fires");
        assert_eq!(b.hurt(0, JAWS_HP, &mut ev), Hurt::Broke, "one huge hit only empties this phase's share");
        assert_eq!(b.hp, floor(1));
        assert_eq!(b.act, Act::Break);
        assert!(ev.iter().any(|e| matches!(e, Event::BossBreak { timeout: false, .. })));
        assert_eq!(b.outcomes[0], Some(Outcome::Broken));
        assert_eq!(b.hurt(0, 100.0, &mut ev), Hurt::Nothing, "no damage between phases");
        run(&mut b, JAWS_BREAK, &mut hz, &mut ev);
        assert_eq!((b.act, b.phase), (Act::Fight, 2));
    }

    #[test]
    fn a_phase_times_out_and_the_last_one_falls_into_the_hole() {
        let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
        let mut b = Boss::at_phase(0.0, 1);
        run(&mut b, JAWS_PHASES[0].1 * TICK_HZ, &mut hz, &mut ev);
        assert_eq!(b.outcomes[0], Some(Outcome::TimedOut));
        assert!(ev.iter().any(|e| matches!(e, Event::BossBreak { timeout: true, .. })));
        assert_eq!(b.hp, floor(1), "a timeout costs it that phase's share");
        let mut b = Boss::at_phase(0.0, 4);
        run(&mut b, JAWS_PHASES[3].1 * TICK_HZ, &mut hz, &mut ev);
        assert_eq!(b.act, Act::Falling);
        assert!(ev.contains(&Event::BossFall));
        run(&mut b, JAWS_FALL, &mut hz, &mut ev);
        assert_eq!(b.act, Act::Gone);
        assert!(ev.contains(&Event::BossGone));
        assert!(b.y > HOLE_Y + 40.0, "down under the disk");
    }

    #[test]
    fn breaking_a_fin_silences_its_spiral() {
        let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
        let mut b = Boss::at_phase(0.0, 2);
        assert_eq!(b.targets().collect::<Vec<_>>(), vec![1, 2, 0], "the fins before the body");
        assert_eq!(b.hurt(1, JAWS_FIN_HP, &mut ev), Hurt::Broke);
        assert!(ev.iter().any(|e| matches!(e, Event::FinBreak { fin: 0, .. })));
        assert_eq!(b.targets().collect::<Vec<_>>(), vec![2, 0]);
        let fin = b.muzzle(Muzzle::Pectoral);
        run(&mut b, 240, &mut hz, &mut ev);
        let near = (0..hz.bullets.len()).filter(|&i| hz.bullets.age[i] < 3 && (hz.bullets.x[i] - fin.0).hypot(hz.bullets.y[i] - fin.1) < 6.0).count();
        assert_eq!(near, 0, "no fresh bullets from the broken fin");
        assert!(hz.bullets.len() > 0, "the other fin still fires");
    }

    #[test]
    fn breaking_both_fins_costs_it_and_enrages_it() {
        let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
        let mut b = Boss::at_phase(0.0, 2);
        let full = b.hp;
        b.hurt(1, JAWS_FIN_HP, &mut ev);
        let seg = floor(1) - floor(2);
        assert!((full - b.hp - seg * JAWS_FIN_BLOW).abs() < 1.0, "a quarter of the phase");
        let windows = |b: &Boss| b.muzzle(Muzzle::Window(0));
        run(&mut b, 200, &mut hz, &mut ev);
        let w0 = windows(&b);
        let fresh = |hz: &Hazards, at: (f32, f32)| (0..hz.bullets.len()).any(|i| hz.bullets.age[i] < 2 && (hz.bullets.x[i] - at.0).hypot(hz.bullets.y[i] - at.1) < 4.0);
        assert!(!fresh(&hz, w0) && b.charges().windows[0] == 0.0, "the windows wait while a fin stands");
        b.hurt(2, JAWS_FIN_HP, &mut ev);
        let mut opened = false;
        for _ in 0..130 {
            b.step(&mut hz, (135.0, 300.0), &mut ev);
            opened |= fresh(&hz, windows(&b));
        }
        assert!(opened, "both fins gone: the windows open up");
    }

    /// Phase 2's turrets hang below and behind the body, so a shot from below reaches
    /// them before it reaches the body.
    #[test]
    fn shots_from_below_can_reach_both_turrets() {
        let mut b = Boss::at_phase(0.0, 2);
        for facing in [1.0, -1.0] {
            b.facing = facing;
            let body = b.rect(0);
            for p in 1..=2 {
                let r = b.rect(p);
                let x = (r.x0 + r.x1) * 0.5;
                let clear = x < body.x0 || x > body.x1 || r.y1 > body.y1;
                assert!(clear, "turret {p} is hidden behind the body (facing {facing})");
            }
        }
    }

    #[test]
    fn the_laser_warns_then_sweeps_and_only_the_beam_hurts() {
        let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
        let mut b = Boss::at_phase(0.0, 3);
        let mut warned = None;
        for f in 0..600u32 {
            b.step(&mut hz, (135.0, 300.0), &mut ev);
            if warned.is_none() && ev.contains(&Event::LaserWarn) {
                warned = Some(f);
                let l = b.laser.unwrap();
                assert!(!l.firing() && !l.touches(l.x + 100.0 * l.from.to_radians().cos(), l.y + 100.0 * l.from.to_radians().sin(), 2.0), "the warning is harmless");
            }
            if let Some(w) = warned {
                if f == w + JAWS_LASER.0 {
                    let l = b.laser.unwrap();
                    assert!(l.firing() && ev.contains(&Event::LaserFire));
                    let (s, c) = l.angle().to_radians().sin_cos();
                    assert!(l.touches(l.x + 150.0 * c, l.y + 150.0 * s, 2.0), "on the beam");
                    assert!(!l.touches(l.x + 150.0 * c - 20.0 * s, l.y + 150.0 * s + 20.0 * c, 2.0), "beside it");
                    assert!(!l.touches(l.x - 50.0 * c, l.y - 50.0 * s, 2.0), "behind the jaws");
                    let start = l.angle();
                    for _ in 0..JAWS_LASER.1 - 1 {
                        b.step(&mut hz, (135.0, 300.0), &mut ev);
                    }
                    let swept = (b.laser.unwrap().angle() - start).abs();
                    assert!((swept - JAWS_LASER.2).abs() < 3.0, "it sweeps about {}: {swept}", JAWS_LASER.2);
                    return;
                }
            }
        }
        panic!("no laser in 10 s");
    }

    #[test]
    fn it_dies_in_a_chain_of_blasts() {
        let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
        let mut b = Boss::at_phase(0.0, 4);
        assert_eq!(b.hurt(0, JAWS_HP, &mut ev), Hurt::Died);
        assert!(ev.iter().any(|e| matches!(e, Event::BossDown { .. })));
        run(&mut b, JAWS_DEATH, &mut hz, &mut ev);
        let blasts = ev.iter().filter(|e| matches!(e, Event::BossBlast { .. })).count();
        assert!((8..=12).contains(&blasts), "{blasts} blasts");
        assert_eq!(ev.iter().filter(|e| matches!(e, Event::BossBlast { last: true, .. })).count(), 1);
        assert_eq!(b.act, Act::Gone);
        assert_eq!(hz.bullets.len(), 0, "it fires nothing while it dies");
    }

    /// Its whole body swims on the tail's beat: its snout sways a little, where the
    /// tail joins swings more, and the tail hangs off the body wherever it's bent.
    #[test]
    fn it_swims_its_whole_body() {
        let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
        let mut b = Boss::at_phase(0.0, 1);
        let (snout, root) = (JAWS_MOUTH.0 + 22.0, JAWS_TAIL_ROOT.0);
        let (mut s, mut r) = ((f32::MAX, f32::MIN), (f32::MAX, f32::MIN));
        for _ in 0..200 {
            b.step(&mut hz, (135.0, 300.0), &mut ev);
            let (bs, br) = (b.bend(snout), b.bend(root));
            (s, r) = ((s.0.min(bs), s.1.max(bs)), (r.0.min(br), r.1.max(br)));
            let j = b.tail()[0];
            assert!((j.1 - (b.y + JAWS_TAIL_ROOT.1 + br)).abs() < 1e-3, "the tail hangs off the bent body");
        }
        let (head, tail) = (s.1 - s.0, r.1 - r.0);
        assert!(head > 1.5 && head <= 2.0 * JAWS_BODY_WAVE.0 + 0.01, "the snout sways {head} px");
        assert!(tail > 4.0 && tail > head * 1.8, "where the tail joins swings {tail} px");
    }

    /// Between attacks it lives: it chomps while its mouth is quiet (never while
    /// it's about to fire), and now and then it draws back and lunges, biting.
    #[test]
    fn it_chomps_and_lunges_between_attacks() {
        let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
        let mut b = Boss::at_phase(0.0, 3);
        let (mut chomps, mut lunges) = (0, 0);
        let (mut was, mut start_x, mut far) = (0.0f32, 0.0f32, 0.0f32);
        for _ in 0..JAWS_PHASES[2].1 * TICK_HZ - 1 {
            b.step(&mut hz, (135.0, 300.0), &mut ev);
            if b.lunge == 0 && b.mouth_due() > JAWS_JAW_LEAD + 12 && was < 0.25 && b.jaw >= 0.25 {
                chomps += 1;
            }
            was = b.jaw;
            if b.lunge == 1 {
                start_x = b.x;
                lunges += 1;
                assert!(matches!(b.attack_now(), Some((_, Beat::Rest))), "it lunges between attacks");
            }
            if b.lunge > 0 {
                far = far.max(b.facing * (b.x - start_x));
                assert!(b.laser.is_none() && b.turn == 0, "never mid-laser or mid-turn");
            }
        }
        assert!(chomps >= 6, "{chomps} chomps");
        assert!(lunges >= 3, "{lunges} lunges");
        assert!(far > JAWS_LUNGE.2 * 0.6, "it lunged {far} px forward");
        assert!(ev.iter().any(|e| matches!(e, Event::BossBite { .. })), "and bit");
    }

    /// It attacks one at a time, in its phase's order: each winds up (its tell glowing,
    /// no bullets), fires, then rests (no new bullets) before the next.
    #[test]
    fn it_attacks_one_at_a_time() {
        for phase in 1..=4u8 {
            let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
            let mut b = Boss::at_phase(0.0, phase);
            let cycle = JAWS_ATTACKS[phase as usize - 1];
            let (mut order, mut shots, mut told) = (vec![], vec![0usize; cycle.len()], vec![false; cycle.len()]);
            let frames: u32 = cycle.iter().map(|a| a.wind + a.fire + a.rest).sum::<u32>() + 2;
            for _ in 0..frames {
                let n = hz.bullets.len();
                b.step(&mut hz, (180.0, 300.0), &mut ev);
                let (a, beat) = b.attack_now().unwrap();
                if order.last() != Some(&b.attack) {
                    order.push(b.attack);
                }
                let new = hz.bullets.len() - n;
                match beat {
                    Beat::Wind(w) => {
                        assert_eq!(new, 0, "phase {phase} attack {}: no bullets as it winds up", b.attack);
                        let c = b.charges();
                        told[b.attack] |= w > 0.9 && (c.mouth > 0.8 || c.windows.iter().chain(&c.others).any(|&k| k > 0.8));
                    }
                    Beat::Fire(_) => shots[b.attack] += new,
                    Beat::Rest => assert_eq!(new, 0, "phase {phase} attack {}: no new bullets as it rests", b.attack),
                }
                assert!(a.laser || b.laser.is_none(), "the laser is its own attack");
            }
            assert_eq!(order, (0..cycle.len()).chain([0]).collect::<Vec<_>>(), "phase {phase}: in order, then round again");
            assert!(told.iter().all(|&t| t), "phase {phase}: every attack is telegraphed: {told:?}");
            for (i, a) in cycle.iter().enumerate() {
                assert!(a.laser || shots[i] > 0, "phase {phase} attack {i} fires");
            }
        }
    }

    /// Between attacks it swims somewhere else, so one safe spot doesn't do for them
    /// all; and past half the phase's health it rests less.
    #[test]
    fn it_moves_between_attacks_and_hurries_when_hurt() {
        let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
        let mut b = Boss::at_phase(0.0, 1);
        let mut spots = vec![];
        for _ in 0..1200 {
            b.step(&mut hz, (180.0, 300.0), &mut ev);
            if let Some((a, Beat::Fire(f))) = b.attack_now() {
                if f == a.fire / 2 && b.turn == 0 {
                    spots.push((b.attack, b.x, b.y));
                }
            }
        }
        let moved = spots.windows(2).filter(|w| w[0].0 != w[1].0).all(|w| (w[0].1 - w[1].1).hypot(w[0].2 - w[1].2) > 12.0);
        assert!(spots.len() >= 3 && moved, "{spots:?}");
        let a = &JAWS_ATTACKS[0][0];
        assert_eq!(b.rest_of(a), a.rest);
        b.hp = floor(1) + (JAWS_HP - floor(1)) * 0.4;
        assert!(b.rest_of(a) < a.rest, "hurt, it rests less");
    }

    /// Hit hard (a bomb, a fin snapping off) it flinches: it recoils and its jaw drops.
    #[test]
    fn a_hard_hit_makes_it_flinch() {
        let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
        let mut b = Boss::at_phase(0.0, 1);
        run(&mut b, 30, &mut hz, &mut ev);
        assert_eq!(b.flinch, 0);
        b.hurt(0, 5.0, &mut ev);
        b.step(&mut hz, (135.0, 300.0), &mut ev);
        assert_eq!(b.flinch, 0, "a shot doesn't");
        let (x, y) = (b.x, b.y);
        b.hurt(0, BOMB_DAMAGE, &mut ev);
        b.step(&mut hz, (135.0, 300.0), &mut ev);
        assert!(b.flinch > 0 && b.facing * (b.x - x) < -2.0 && b.y < y, "a bomb does: it recoils");
        run(&mut b, 8, &mut hz, &mut ev);
        assert!(b.jaw > 0.5, "its jaw drops");
        // a fin snapping off
        let mut b = Boss::at_phase(0.0, 2);
        run(&mut b, 90, &mut hz, &mut ev);
        b.hurt(1, JAWS_FIN_HP, &mut ev);
        b.step(&mut hz, (135.0, 300.0), &mut ev);
        assert!(b.flinch > 0, "a fin breaking does");
    }

    #[test]
    fn it_stays_on_the_screen_and_turns_round() {
        let (mut hz, mut ev) = (Hazards::new(0, 1), vec![]);
        let mut b = Boss::at_phase(0.0, 1);
        let mut facings = vec![b.facing];
        for _ in 0..JAWS_PHASES[0].1 * TICK_HZ - 1 {
            b.step(&mut hz, (135.0, 300.0), &mut ev);
            if *facings.last().unwrap() != b.facing {
                facings.push(b.facing);
            }
            let j = b.tail();
            let (snout, fin) = (b.at((72.0, 0.0)).0, j[5].0 - 24.0 * b.facing);
            for x in [snout, fin] {
                assert!((-4.0..=FIELD_W as f32 + 4.0).contains(&x), "x {x} off the screen (facing {})", b.facing);
            }
        }
        assert!(facings.len() >= 2, "it turned: {facings:?}");
        assert!(ev.contains(&Event::BossTurn));
    }
}
