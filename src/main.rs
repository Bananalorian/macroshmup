// No console window behind the game on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! PILGRIM
//! A vertical bullet-hell shmup. Somewhere beyond the event horizon, a small ship,
//! a long way from home.
//!
//! Everything you see and hear is generated in code: the sprite atlas and the
//! Pilgrim (render/art.rs, render/ship.rs), the font (render/font.rs), the
//! starfield (render/bg.rs) and every sound (synth.rs). The game runs a fixed
//! 60 Hz simulation (sim/) and renders every display frame.

mod audio;
mod clock;
mod debug;
mod input;
mod menu;
mod pad;
mod render;
#[cfg(not(target_arch = "wasm32"))]
mod sheet;
mod sim;
mod storage;
mod synth;
mod tune;

use audio::Audio;
use debug::SceneReq;
use input::{Input, Mode, Ui};
use macroquad::miniquad;
use macroquad::prelude::*;
use menu::{ITEMS, Item};
use render::explode::{Size, explode};
use render::layout::IRect;
use render::{PostParams, Renderer};
use render::hud::{self, HudView};
use render::layout::Layout;
use sim::enemy::EnemyKind;
use sim::player::Controls;
use sim::{Event, Life, World};

const TEAL_C: Color = render::palette::TEAL;
const SIGNAL_C: Color = render::palette::SIGNAL;
use storage::Settings;
use synth::Id;

enum Scene {
    /// The title, `t` ticks after it appeared, with `sel` chosen; untouched for
    /// `idle` ticks (the demo starts after a while).
    Title { t: u64, sel: usize, idle: u64 },
    /// The title's options.
    Options { t: u64, sel: usize },
    /// A run: Stage 1, or the pattern sandbox. `menu` is the pause menu's selected
    /// row while paused; `over` the end menu's once the last ship is gone or the
    /// stage is over.
    Stage { world: Box<World>, menu: Option<usize>, over: Option<usize> },
}

/// The derelict's death: blasts rippling across the wreck, then the big one.
struct Chain {
    x: f32,
    y: f32,
    t: u32,
}

impl Chain {
    const EVERY: u32 = 6;
    const BLASTS: u32 = 9;
    /// The last blast, frames in.
    fn end() -> u32 {
        Self::EVERY * Self::BLASTS + 10
    }
}

/// What kind of run the stage scene is playing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Run {
    /// Stage 1 from the top: it counts for the hi-score, and reaching the boss
    /// unlocks Boss Practice.
    Stage,
    /// From partway in: the boss's arrival (Boss Practice), or a debug start.
    Practice,
    Sandbox,
    /// The title's attract demo: the bot plays.
    Demo,
}

struct Game {
    scene: Scene,
    run: Run,
    /// Ticks since the run ended (the tally counts up on them), and whether it set a
    /// new hi-score.
    end_t: u32,
    record: bool,
    /// Ticks fire has been held (the HOLD TO BEAM scheme).
    hold_t: u32,
    seed: u64,
    settings: Settings,
    toast: Option<(String, u32)>,
    /// Debug: extra shots kept alive in the stage (PG_SCENE=shots:<n>).
    fill_shots: usize,
    /// BOOM TEST asked for an explosion of this size.
    boom: Option<Size>,
    chains: Vec<Chain>,
    /// Hit-stop: ticks the stage holds still for (a big kill just landed).
    stop: u32,
}

/// What a frame of menu input asked for.
#[derive(Debug, PartialEq)]
enum MenuAct {
    Nothing,
    Changed,
    Resume,
    Quit,
    /// BOOM TEST: resume and set off an explosion.
    Boom,
}

/// Apply one frame of input to a menu of `items` (the pause menu, the options).
fn menu_input(sel: &mut usize, ui: &Ui, lay: &Layout, s: &mut Settings, items: &[Item]) -> MenuAct {
    if ui.back || ui.pause {
        return MenuAct::Resume;
    }
    let n = items.len() as i32;
    *sel = (*sel as i32 + ui.nav.1).rem_euclid(n) as usize;
    let activate = |sel: usize, s: &mut Settings| match items[sel] {
        Item::Resume | Item::Back => MenuAct::Resume,
        Item::Quit => MenuAct::Quit,
        Item::Boom => MenuAct::Boom,
        item => {
            menu::adjust(s, item, 1);
            MenuAct::Changed
        }
    };
    if ui.nav.0 != 0 && items[*sel].adjustable() {
        menu::adjust(s, items[*sel], ui.nav.0.signum());
        return MenuAct::Changed;
    }
    if ui.confirm {
        return activate(*sel, s);
    }
    if let Some((x, y)) = ui.tap {
        if let Some((row, step)) = hud::menu_tap(lay, items, x, y) {
            *sel = row;
            if step != 0 {
                menu::adjust(s, items[row], step);
                return MenuAct::Changed;
            }
            return activate(row, s);
        }
    }
    MenuAct::Nothing
}

impl Game {
    fn new(seed: u64, req: Option<SceneReq>, settings: Settings) -> Game {
        let mut g = Game {
            scene: Scene::Title { t: 0, sel: 0, idle: 0 },
            run: Run::Sandbox,
            end_t: 0,
            record: false,
            hold_t: 0,
            seed,
            settings,
            toast: None,
            fill_shots: 0,
            boom: None,
            chains: vec![],
            stop: 0,
        };
        match req {
            None | Some(SceneReq::Title) => {}
            Some(SceneReq::Options) => g.scene = Scene::Options { t: 0, sel: 0 },
            Some(SceneReq::Stage(secs)) => g.start_stage(secs),
            Some(SceneReq::Boss(phase)) => g.start_boss(phase),
            Some(SceneReq::Shots(n)) => {
                g.fill_shots = n;
                g.start_sandbox();
            }
            Some(SceneReq::Sandbox(i)) => {
                g.settings.view.pattern = i;
                g.start_sandbox();
            }
        }
        g
    }

    /// Play the same run again from the top.
    fn retry(&mut self) {
        match self.run {
            Run::Stage | Run::Demo => self.start_stage(0.0),
            Run::Practice => self.start_boss(0),
            Run::Sandbox => self.start_sandbox(),
        }
    }

    /// A new run in `w`.
    fn begin(&mut self, mut w: World, run: Run) {
        w.invincible = self.settings.view.invincible && run != Run::Demo;
        self.chains.clear();
        (self.run, self.end_t, self.record, self.hold_t) = (run, 0, false, 0);
        self.scene = Scene::Stage { world: Box::new(w), menu: None, over: None };
    }

    /// Stage 1, `secs` seconds in (only a run from the top counts for the hi-score).
    fn start_stage(&mut self, secs: f32) {
        let w = World::stage_at(self.seed, self.settings.tuning, secs);
        self.begin(w, if secs == 0.0 { Run::Stage } else { Run::Practice });
    }

    /// Boss Practice: from its arrival (phase 0), or straight into a phase.
    fn start_boss(&mut self, phase: u8) {
        let w = World::boss_at(self.seed, self.settings.tuning, phase);
        self.begin(w, Run::Practice);
    }

    /// The title's demo: the bot plays Stage 1.
    fn start_demo(&mut self) {
        let w = World::stage(self.seed, self.settings.tuning);
        self.begin(w, Run::Demo);
    }

    fn start_sandbox(&mut self) {
        let mut w = World::new(self.seed, self.settings.tuning);
        w.fill_shots = self.fill_shots;
        w.set_pattern(self.settings.view.pattern);
        self.begin(w, Run::Sandbox);
    }

    /// Back to the title.
    fn to_title(&mut self, sel: usize) {
        self.scene = Scene::Title { t: 0, sel, idle: 0 };
    }

    /// The title, left alone, plays a demo; the demo ends on any button (`touched`),
    /// at its end, or after a minute. True when a demo just ended.
    fn attract(&mut self, touched: bool) -> bool {
        match &self.scene {
            Scene::Title { idle, .. } if *idle >= tune::ATTRACT_AFTER => {
                self.start_demo();
                false
            }
            Scene::Stage { world, over, .. } if self.run == Run::Demo && (touched || over.is_some() || world.frame >= tune::DEMO_LEN) => {
                self.to_title(0);
                true
            }
            _ => false,
        }
    }


    /// The sandbox switches changed: apply them to the running game.
    fn sync_view(&mut self) {
        let v = self.settings.view;
        if let Scene::Stage { world, .. } = &mut self.scene {
            if world.stage.is_none() && world.hazards.preset != v.pattern {
                world.set_pattern(v.pattern);
            }
            world.invincible = v.invincible;
        }
    }

    /// Step the sandbox pattern from a key, LB or the NEXT button.
    fn step_pattern(&mut self, dir: i32) {
        if dir == 0 || !matches!(&self.scene, Scene::Stage { menu: None, over: None, world } if world.stage.is_none()) {
            return;
        }
        menu::adjust(&mut self.settings, Item::Pattern, dir.signum());
        self.sync_view();
        let n = tune::SANDBOX.len();
        let msg = format!("{}  {}/{}", menu::value(&self.settings, Item::Pattern), self.settings.view.pattern + 1, n);
        self.say(msg);
    }

    fn say(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), tune::TOAST_FRAMES));
    }

    /// The settings changed: apply them everywhere and save.
    fn settings_changed(&mut self, input: &mut Input) {
        if let Scene::Stage { world, .. } = &mut self.scene {
            world.tuning = self.settings.tuning;
        }
        input.analog = self.settings.analog;
        self.settings.save();
    }

    fn sim_frame(&self) -> u64 {
        match &self.scene {
            Scene::Title { t, .. } | Scene::Options { t, .. } => *t,
            Scene::Stage { world, .. } => world.frame,
        }
    }

    fn paused(&self) -> bool {
        matches!(self.scene, Scene::Stage { menu: Some(_), .. })
    }

    /// Menus, pausing, starting and quitting: everything that isn't flying.
    fn handle_ui(&mut self, ui: &Ui, lay: &Layout, input: &mut Input, audio: &mut Audio) {
        if ui.mute {
            self.settings.muted = !self.settings.muted;
            audio.muted = self.settings.muted;
            self.settings.save();
            self.say(if self.settings.muted { "SOUND OFF" } else { "SOUND ON" });
        }
        let items = hud::title_items(self.settings.boss_unlocked);
        match &mut self.scene {
            Scene::Title { sel, idle, .. } => {
                if ui.confirm || ui.back || ui.pause || ui.tap.is_some() || ui.nav != (0, 0) {
                    *idle = 0;
                }
                let n = items.len() as i32;
                let before = *sel;
                *sel = (*sel as i32 + ui.nav.1).rem_euclid(n) as usize;
                let mut choice = ui.confirm.then_some(*sel);
                if let Some((x, y)) = ui.tap {
                    // a tap on a row picks it; anywhere else starts what's selected
                    choice = Some(hud::title_tap(lay, items.len(), x, y).unwrap_or(*sel));
                }
                if *sel != before {
                    audio.play(Id::Select, 0.5);
                }
                if let Some(i) = choice {
                    audio.play(Id::Select, 1.0);
                    input.release();
                    match items[i] {
                        hud::TitleItem::Stage => self.start_stage(0.0),
                        hud::TitleItem::Practice => self.start_boss(0),
                        hud::TitleItem::Sandbox => self.start_sandbox(),
                        hud::TitleItem::Options => self.scene = Scene::Options { t: 0, sel: 0 },
                    }
                }
            }
            Scene::Options { sel, .. } => {
                let before = *sel;
                let act = menu_input(sel, ui, lay, &mut self.settings, &menu::OPTIONS);
                if *sel != before {
                    audio.play(Id::Select, 0.5);
                }
                match act {
                    MenuAct::Changed => {
                        self.settings_changed(input);
                        audio.volumes(self.settings.music_vol, self.settings.sound_vol);
                        audio.play(Id::Select, 0.7);
                    }
                    MenuAct::Nothing => {}
                    _ => {
                        input.release();
                        let back = items.iter().position(|i| *i == hud::TitleItem::Options).unwrap_or(0);
                        self.to_title(back);
                    }
                }
            }
            // the tally plays out first: any button skips to its end
            Scene::Stage { over: Some(_), world, .. } if self.end_t < if world.stage.as_ref().is_some_and(|s| s.over) { tune::TALLY_DONE } else { 0 } => {
                if ui.confirm || ui.back || ui.tap.is_some() {
                    self.end_t = tune::TALLY_DONE;
                    input.release();
                }
            }
            Scene::Stage { over: Some(sel), world, .. } => {
                let n = hud::OVER_ITEMS.len() as i32;
                let before = *sel;
                let cleared = world.stage.as_ref().is_some_and(|s| s.over);
                let staged = self.run != Run::Sandbox;
                *sel = (*sel as i32 + ui.nav.1).rem_euclid(n) as usize;
                let mut choice = ui.confirm.then_some(*sel);
                if let Some((x, y)) = ui.tap {
                    if let Some(i) = hud::ended_tap(lay, cleared, x, y) {
                        *sel = i;
                        choice = Some(i);
                    }
                }
                if *sel != before {
                    audio.play(Id::Select, 0.5);
                }
                if ui.back {
                    choice = Some(1);
                }
                match choice {
                    Some(0) => {
                        audio.play(Id::Select, 1.0);
                        audio.silence();
                        input.release();
                        self.retry();
                    }
                    Some(_) => {
                        audio.silence();
                        input.release();
                        let back = if staged { 0 } else { items.iter().position(|i| *i == hud::TitleItem::Sandbox).unwrap_or(0) };
                        self.to_title(back);
                    }
                    None => {}
                }
            }
            Scene::Stage { menu: menu @ None, .. } => {
                if ui.pause {
                    *menu = Some(0);
                    audio.play(Id::Select, 1.0);
                    input.release();
                } else if let Some((knob, dir)) = ui.tune {
                    let item = [Item::Speed, Item::Focus, Item::Blend][knob];
                    menu::adjust(&mut self.settings, item, dir);
                    let msg = format!("{} {}", item.label(), menu::value(&self.settings, item));
                    self.settings_changed(input);
                    self.say(msg);
                }
            }
            Scene::Stage { menu: Some(sel), .. } => {
                let before = *sel;
                let act = menu_input(sel, ui, lay, &mut self.settings, &ITEMS);
                if *sel != before {
                    audio.play(Id::Select, 0.5);
                }
                match act {
                    MenuAct::Nothing => {}
                    MenuAct::Changed => {
                        audio.play(Id::Select, 0.7);
                        self.settings_changed(input);
                        self.sync_view();
                    }
                    MenuAct::Resume => {
                        if let Scene::Stage { menu, .. } = &mut self.scene {
                            *menu = None;
                        }
                        input.release();
                    }
                    MenuAct::Quit => {
                        audio.silence();
                        input.release();
                        let back = if self.run == Run::Sandbox { items.iter().position(|i| *i == hud::TitleItem::Sandbox).unwrap_or(0) } else { 0 };
                        self.to_title(back);
                    }
                    MenuAct::Boom => {
                        if let Scene::Stage { menu, .. } = &mut self.scene {
                            *menu = None;
                        }
                        input.release();
                        self.boom = Some(Size::ALL[self.settings.view.boom % 4]);
                    }
                }
            }
        }
    }
}

/// The game opens full screen unless the player last chose a window (F11). A
/// browser only goes full screen on a click, and debug runs keep their window.
fn start_fullscreen() -> bool {
    !cfg!(target_arch = "wasm32") && !debug::scripted() && Settings::load().fullscreen
}

fn window_conf() -> Conf {
    let (w, h) = debug::window_size().unwrap_or((1280, 720));
    Conf {
        window_title: "PILGRIM".to_owned(),
        window_width: w,
        window_height: h,
        fullscreen: start_fullscreen(),
        // physical pixels, so the integer scale is exact on high-DPI screens and phones
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

/// The enemies' engines, smoke from the damaged ones, flakes off the rising derelict.
fn foe_effects(w: &World, r: &mut Renderer) {
    for e in w.foes.list.iter().filter(|e| e.alive()) {
        if w.frame % 3 == 0 {
            for &(dx, dy) in render::foes::engines(e.kind) {
                r.parts.exhaust(e.x + dx, e.y + dy);
            }
        }
        let q = &e.parts[0];
        let (_, bw, bh) = e.kind.stats();
        if !e.kind.fodder() && q.hp < q.max_hp * 0.5 && w.frame % 5 == 0 {
            let (x, y) = r.parts.somewhere(e.x, e.y, bw * 0.6, bh * 0.6);
            r.parts.wound(x, y);
        }
        // a rushing lancer throws a wake of grit behind it
        if let sim::enemy::Path::Rush { enter, wait, dir, .. } = e.path {
            if e.t >= enter + wait {
                r.parts.wake(e.x - dir.0 * 9.0, e.y - dir.1 * 9.0, dir);
            }
        }
        if e.path.entering(e.t) && e.kind == EnemyKind::Hull && w.frame % 2 == 0 {
            let (x, y) = r.parts.somewhere(e.x, e.y, bw, bh);
            r.parts.flake(x, y);
        }
    }
}

/// JAWSTHIC's running effects: glitch smears streaming off its tail fin and plates
/// the way it's leaving, threads dripping off its belly.
fn boss_effects(w: &World, r: &mut Renderer) {
    use sim::boss::Act;
    let Some(b) = w.boss.as_ref().filter(|b| b.act != Act::Gone) else { return };
    let back = -b.facing;
    let surface = b.surface();
    let above = |y: f32| surface.is_none_or(|s| y < s - 1.0);
    // the fin cutting through the disk throws up a wake
    if b.act == Act::Intro && b.t < tune::JAWS_INTRO[0] {
        let (fx, _) = b.at(tune::JAWS_DORSAL_AT);
        r.parts.splash(fx - 6.0 * b.facing, tune::HOLE_Y - 1.0, 14.0, 2);
    }
    let wild = b.turn > 0 || matches!(b.act, Act::Dying | Act::Falling);
    let hot = wild || b.phase >= 3;
    let j = b.tail();
    let (tx, ty, _) = j[5];
    let rate = if wild { 6 } else { [1, 1, 2, 2, 3][b.phase as usize] };
    for _ in 0..rate {
        let (x, y) = r.parts.somewhere(tx + back * 14.0, ty - 4.0, 18.0, 52.0);
        if above(y) {
            r.parts.smear(x, y, back, hot);
        }
    }
    if w.frame % 3 == 0 {
        let i = (w.frame / 3 % 5) as usize;
        let (x, y) = r.parts.somewhere(j[i].0, j[i].1, 8.0, 18.0);
        if above(y) {
            r.parts.smear(x, y, back, hot);
        }
    }
    if wild {
        let (cx, cy) = b.at((10.0, -2.0));
        for _ in 0..3 {
            let (x, y) = r.parts.somewhere(cx, cy, 110.0, 50.0);
            if above(y) {
                r.parts.smear(x, y, back, true);
            }
        }
    }
    if w.frame % if hot { 6 } else { 17 } == 0 {
        let (cx, cy) = b.at((12.0, 21.0));
        let (x, y) = r.parts.somewhere(cx, cy, 70.0, 2.0);
        if above(y) {
            r.parts.drip(x, y);
        }
    }
}

/// How big a kill's blast is.
fn kill_size(kind: EnemyKind) -> Size {
    match kind {
        EnemyKind::Drone | EnemyKind::Courier | EnemyKind::Usher | EnemyKind::Lancer | EnemyKind::Remora | EnemyKind::Debris => Size::Small,
        EnemyKind::Gunship | EnemyKind::Buoy | EnemyKind::Satellite | EnemyKind::Monolith => Size::Medium,
        EnemyKind::Heavy | EnemyKind::Hull => Size::Large,
    }
}

/// Hit-stop: how many ticks the stage holds still after this tick's events. Big
/// kills land with a moment's freeze; fodder never stops the flow; a boss phase
/// breaking, a turret or a weak point snapping off, hold longest. Never while a bomb
/// sweeps the screen: it plays as it always has (a stop would freeze its ring).
fn hitstop(events: &[Event], bombing: bool) -> u32 {
    if bombing {
        return 0;
    }
    events
        .iter()
        .map(|e| match *e {
            Event::Kill { kind, .. } => tune::HITSTOP[kill_size(kind) as usize],
            Event::Break { big, .. } => tune::HITSTOP[if big { 2 } else { 1 }],
            Event::PartBreak { .. } | Event::FinBreak { .. } => tune::HITSTOP[2],
            Event::BossBreak { .. } => tune::HITSTOP_BOSS,
            Event::Slam { .. } => tune::HITSTOP_SLAM,
            Event::ShieldBreak { .. } => tune::HITSTOP_SHIELD,
            _ => 0,
        })
        .max()
        .unwrap_or(0)
}

/// JAWSTHIC's moments: the breach, the roar, phases breaking, turns, fins snapping
/// off, the laser, and its death in a chain of blasts ending in a white-out.
fn boss_moments(w: &World, r: &mut Renderer, s: &Settings, audio: &mut Audio) {
    let me = (w.player.x, w.player.y);
    let Some(b) = &w.boss else { return };
    let mirror = |pixels: &[(i8, i8, Color)]| -> Vec<(i8, i8, Color)> { pixels.iter().map(|&(x, y, c)| (if b.facing < 0.0 { -x } else { x }, y, c)).collect() };
    let surge = |r: &mut Renderer, px: f32, frames: u32| r.fx.surge(if s.reduce_flash { px * tune::REDUCED_CA } else { px }, frames);
    for e in &w.events {
        match *e {
            Event::BossBreach { x, y } => {
                r.parts.splash(x, y, 110.0, 90);
                r.fx.shock(x, y, 140.0, 3.0, 34);
                audio.blast(Size::Large);
            }
            Event::BossRoar { x, y } => {
                let (mx, my) = b.muzzle(sim::boss::Muzzle::Mouth);
                r.parts.ring(mx, my, 5.0, 26, SIGNAL_C);
                r.fx.shock(x, y, 200.0, 4.0, 40);
                surge(r, 2.4, 14);
                r.fx.shake(tune::SHAKE_BOMB, w.hazards.bullets.len(), s.shake);
            }
            Event::BossPhase(_) => {
                let (cx, cy) = b.at((10.0, -2.0));
                r.parts.ring(cx, cy, 4.0, 30, SIGNAL_C);
            }
            Event::BossBreak { timeout, .. } => {
                let (cx, cy) = b.at((10.0, -2.0));
                explode(&mut r.fx, &mut r.parts, cx, cy, if timeout { Size::Medium } else { Size::Large }, me, s.reduce_flash, w.frame);
                r.parts.ring(cx, cy, 6.0, 30, render::palette::CREAM);
                surge(r, 1.6, 10);
                for _ in 0..40 {
                    let (x, y) = r.parts.somewhere(cx, cy, 150.0, 60.0);
                    r.parts.smear(x, y, -b.facing, true);
                }
                audio.blast(Size::Large);
            }
            Event::BossTurn => {
                let (cx, cy) = b.at((10.0, -2.0));
                for _ in 0..24 {
                    let (x, y) = r.parts.somewhere(cx, cy, 160.0, 60.0);
                    r.parts.smear(x, y, -b.facing, true);
                }
            }
            Event::FinBreak { x, y, fin } => {
                let pieces = if fin == 0 { &r.jaws.fin_shards } else { &r.jaws.tail_shards[5] };
                r.parts.shatter(x, y, &mirror(pieces));
                explode(&mut r.fx, &mut r.parts, x, y, Size::Medium, me, s.reduce_flash, w.frame);
                r.parts.smoke(x, y, 8, 20.0);
                audio.blast(Size::Medium);
            }
            Event::BossBite { x, y } => {
                // the fangs meeting: chips of stone and sparks off the jaws
                for _ in 0..3 {
                    r.parts.hit(x, y, false);
                }
                r.parts.smoke(x, y, 3, 10.0);
            }
            Event::LaserFire => {
                if let Some(l) = b.laser {
                    for _ in 0..3 {
                        r.parts.hit(l.x, l.y, false);
                    }
                    r.fx.aberrate(if s.reduce_flash { 0.4 } else { 1.2 });
                }
            }
            Event::BossDown { x, y } => {
                explode(&mut r.fx, &mut r.parts, x, y, Size::Large, me, s.reduce_flash, w.frame);
                r.parts.ring(x, y, 6.0, 40, SIGNAL_C);
                surge(r, 3.0, 16);
                audio.blast(Size::Boss);
            }
            Event::BossBlast { x, y, last } => {
                if last {
                    let (cx, cy) = (b.x, b.y);
                    explode(&mut r.fx, &mut r.parts, cx, cy, Size::Boss, me, s.reduce_flash, w.frame);
                    r.parts.shatter(cx, cy, &mirror(&r.jaws.shards));
                    r.parts.smoke(cx, cy, 40, 120.0);
                    r.parts.glow(cx, cy, 8.0, 30);
                    r.parts.ring(cx, cy, 7.0, 36, render::palette::CREAM);
                    r.fx.whiteout(70, if s.reduce_flash { 0.45 } else { 0.9 });
                    surge(r, tune::BOMB_SURGE.0, tune::BOMB_SURGE.1);
                    r.fx.shake(tune::SHAKE_BOMB * 1.4, w.hazards.bullets.len(), s.shake);
                    audio.blast(Size::Boss);
                } else {
                    // the piece it hit blows off in its own pixels: the tail fin, the
                    // plates from the back, the fins
                    let k = b.blown().saturating_sub(1);
                    let piece = match k {
                        0 => Some(&r.jaws.tail_shards[5]),
                        1..=5 => Some(&r.jaws.tail_shards[5 - k]),
                        6 => Some(&r.jaws.dorsal_shards),
                        7 => Some(&r.jaws.fin_shards),
                        _ => None,
                    };
                    if let Some(px) = piece {
                        let px = mirror(px);
                        r.parts.shatter(x, y, &px);
                    }
                    explode(&mut r.fx, &mut r.parts, x, y, Size::Medium, me, s.reduce_flash, w.frame);
                    r.fx.shock(x, y, 70.0, 2.5, 22);
                    r.parts.smoke(x, y, 6, 20.0);
                    for _ in 0..10 {
                        let (sx, sy) = r.parts.somewhere(x, y, 30.0, 24.0);
                        r.parts.smear(sx, sy, -b.facing, true);
                    }
                    r.fx.shake(tune::SHAKE_DEATH * 0.6, w.hazards.bullets.len(), s.shake);
                    audio.blast(Size::Large);
                }
            }
            Event::BossFall => {
                r.parts.splash(b.x, tune::HOLE_Y, 160.0, 120);
                surge(r, 2.0, 30);
            }
            _ => {}
        }
    }
}

/// The derelict's chained death, a blast every few frames across the wreck.
fn chain_effects(chains: &mut Vec<Chain>, w: &World, r: &mut Renderer, s: &Settings, audio: &mut Audio) {
    let me = (w.player.x, w.player.y);
    for c in chains.iter_mut() {
        c.t += 1;
        let (_, bw, bh) = EnemyKind::Hull.stats();
        if c.t % Chain::EVERY == 0 && c.t <= Chain::EVERY * Chain::BLASTS {
            let (x, y) = r.parts.somewhere(c.x, c.y, bw, bh);
            explode(&mut r.fx, &mut r.parts, x, y, Size::Medium, me, s.reduce_flash, w.frame);
            r.parts.smoke(x, y, 4, 14.0);
            audio.blast(Size::Medium);
        }
        if c.t == Chain::end() {
            explode(&mut r.fx, &mut r.parts, c.x, c.y, Size::Boss, me, s.reduce_flash, w.frame);
            r.parts.shatter(c.x, c.y, &r.atlas.r.foes.hull.pixels);
            r.parts.smoke(c.x, c.y, 30, 90.0);
            r.parts.glow(c.x, c.y, 6.0, 24);
            r.parts.ring(c.x, c.y, 6.0, 30, render::palette::CREAM);
            let (px, frames) = tune::BOMB_SURGE;
            r.fx.surge(if s.reduce_flash { px * tune::REDUCED_CA * 0.6 } else { px * 0.6 }, frames);
            r.fx.shake(tune::SHAKE_BOMB, w.hazards.bullets.len(), s.shake);
            audio.blast(Size::Boss);
        }
    }
    chains.retain(|c| c.t < Chain::end());
}

/// Focusing this tick: the focus button, or (HOLD TO BEAM) fire held a moment.
fn focused(button: bool, hold_beam: bool, fire_held: u32) -> bool {
    button || (hold_beam && fire_held > tune::HOLD_BEAM)
}

/// A run ended with `points`: a new best (Stage 1 from the top only) is kept and
/// saved. True when it's a new record.
fn new_record(s: &mut Settings, run: Run, points: u64) -> bool {
    let record = run == Run::Stage && points > s.hiscore;
    if record {
        s.hiscore = points;
        s.save();
    }
    record
}

/// The tally's sounds, `e` ticks after the stage ended: a tick as each line comes
/// in, rapid ticks while the score counts up, the rank's slam, and a jingle for a
/// new record.
fn tally_cue(e: u32, record: bool) -> Option<(Id, f32)> {
    use tune::{TALLY_COUNT, TALLY_RANK, TALLY_SCORE, TALLY_STEP};
    if (1..=5).any(|i| e == TALLY_STEP * i) {
        Some((Id::Tick, 1.0))
    } else if (TALLY_SCORE..TALLY_SCORE + TALLY_COUNT).contains(&e) && e % 3 == 0 {
        Some((Id::Tick, 0.6))
    } else if e == TALLY_RANK {
        Some((Id::Slam, 1.0))
    } else if record && e == TALLY_RANK + 24 {
        Some((Id::Extend, 1.0))
    } else {
        None
    }
}

/// Feed one tick's events to the effects and the blasts' sounds.
fn effects(w: &World, r: &mut Renderer, s: &Settings, audio: &mut Audio, chains: &mut Vec<Chain>) {
    let p = &w.player;
    let me = (p.x, p.y);
    foe_effects(w, r);
    let flying = matches!(w.life, Life::Alive | Life::Respawning { .. });
    let ship = r.atlas.r.ship.frame(p.blend(&w.tuning), p.bank);
    if flying && w.frame % 2 == 0 {
        let (jx, jy) = ship.nozzles[2];
        r.parts.trail(p.x + jx as f32, p.y + jy as f32 + 2.0, p.thrust() || !w.alive());
    }
    if flying && w.frame % 4 == 0 {
        for &(nx, ny) in &ship.nozzles[..2] {
            r.parts.trail(p.x + nx as f32, p.y + ny as f32 + 3.0, false);
        }
    }
    // the augments' jets
    if flying && w.frame % 4 == 2 {
        for &(ax, ay) in &w.kit.aug[..w.kit.augments()] {
            r.parts.trail(ax, ay + 8.0, false);
        }
    }
    // the blades lock into the laser form: sparks off the crossbar, a glint at the tip
    if flying && p.locked == 1 {
        r.parts.lock(p.x, p.y + render::ship::BAR as f32, p.y + render::ship::TIP as f32);
    }
    // a bomb's cancelled bullets become teal shards that fly to you (a sample on a crowded screen)
    if w.cancel_shards {
        let step = (w.cancelled.len() / 500).max(1);
        for &(cx, cy) in w.cancelled.iter().step_by(step) {
            r.parts.shard_to_you(cx, cy);
        }
    }
    for e in &w.events {
        match *e {
            Event::SlamDash { x, y, .. } => {
                // off it goes: teal sparks kicked back where it launched
                for _ in 0..3 {
                    r.parts.graze(x, y + 6.0, x, y);
                }
            }
            Event::Slam { x, y, link } => {
                // the sword strikes: a ring, sparks, and a jolt of aberration, bigger
                // down the chain
                let k = link as f32;
                r.fx.shock(x, y, 40.0 + 14.0 * k, 2.0, 18);
                r.parts.ring(x, y, 3.0 + k, 16, render::palette::TEAL);
                for _ in 0..4 + 2 * link {
                    r.parts.hit(x, y, false);
                }
                r.fx.aberrate(if s.reduce_flash { 0.3 } else { 0.8 + 0.3 * k });
            }
            Event::Hit { x, y, beam } => {
                if !beam || w.frame % 3 == 0 {
                    r.parts.hit(x, y, beam);
                }
                if beam {
                    r.parts.beam_spray(x, y);
                }
            }
            Event::Break { x, y, big } => {
                // much bigger than what died
                let size = if big { Size::Large } else { Size::Medium };
                explode(&mut r.fx, &mut r.parts, x, y, size, me, s.reduce_flash, w.frame);
                audio.blast(size);
            }
            Event::Graze { x, y } => r.parts.graze(x, y, p.x, p.y),
            Event::Bomb { x, y } => {
                // Shinjuku's grenade blast, scaled up to fill the screen: the boss-sized
                // blast, smoke, a hot glow (its temporary light), a screen-wide shock ring
                // and the RGB surge
                explode(&mut r.fx, &mut r.parts, x, y, Size::Boss, me, s.reduce_flash, w.frame);
                r.parts.smoke(x, y, 36, 120.0);
                r.parts.glow(x, y, 7.0, 28);
                r.parts.ring(x, y, 7.0, 30, TEAL_C);
                r.fx.shock(x, y, 320.0, 6.0, 44);
                let (px, frames) = tune::BOMB_SURGE;
                r.fx.surge(if s.reduce_flash { px * tune::REDUCED_CA } else { px }, frames);
            }
            // the ring has swept the screen clean: now it may shake
            Event::BombSwept => {
                r.fx.shake(tune::SHAKE_BOMB, w.hazards.bullets.len(), s.shake);
            }
            Event::NoBomb => {}
            Event::Struck { x, y } => {
                r.parts.ring(x, y, 2.5, 10, SIGNAL_C);
                r.parts.hit(x, y, false);
                r.fx.aberrate(if s.reduce_flash { 0.6 } else { 1.6 });
            }
            Event::Kit { level, x, y } => {
                // a piece of the kit comes online: teal sparks gather where it'll sit,
                // a ring, a jolt of aberration and its word
                let (gx, gy) = if level <= 2 {
                    let (dx, dy) = sim::kit::Kit::post(level as usize - 1, p.blend(&w.tuning));
                    (x + dx, y + dy)
                } else {
                    (x, y)
                };
                r.parts.gather(gx, gy, if level >= 3 { 44.0 } else { 24.0 }, 20);
                r.parts.ring(gx, gy, 2.0, 18, TEAL_C);
                r.fx.aberrate(if s.reduce_flash { 0.3 } else { 0.8 });
                r.popups.kit(level, x, y);
            }
            Event::ShieldBreak { x, y } => {
                // the hexagon flies apart, the bullets round you pop, a shock ring
                r.parts.shield_shatter(x, y, &render::scene::shield_outline());
                for &(cx, cy) in &w.cancelled {
                    r.parts.cancel(cx, cy);
                }
                r.parts.ring(x, y, 4.0, 16, TEAL_C);
                r.fx.shock(x, y, 80.0, 2.5, 20);
                r.fx.surge(if s.reduce_flash { 0.5 } else { 1.4 }, 10);
            }
            Event::ShieldUp { x, y } => {
                r.parts.gather(x, y, 48.0, 28);
                r.parts.ring(x, y, 3.0, 14, TEAL_C);
            }
            Event::AugmentLost { x, y } => r.parts.shatter(x, y, &r.atlas.r.ship.augment_pixels),
            Event::Death { x, y } => {
                // the sheet's destroy frames: the hull shatters into its own pixels
                r.parts.shatter(x, y, &r.atlas.r.ship.pixels);
                explode(&mut r.fx, &mut r.parts, x, y, Size::Large, me, s.reduce_flash, w.frame);
                r.fx.surge(if s.reduce_flash { tune::DEATH_SURGE.0 * tune::REDUCED_CA } else { tune::DEATH_SURGE.0 }, tune::DEATH_SURGE.1);
            }
            Event::Cancel => {
                // every bullet pops (a sample of them on a crowded screen)
                let step = (w.cancelled.len() / 600).max(1);
                for &(cx, cy) in w.cancelled.iter().step_by(step) {
                    r.parts.cancel(cx, cy);
                }
                // the screen's been cancelled, so it may shake
                r.fx.shake(tune::SHAKE_DEATH, w.hazards.bullets.len(), s.shake);
            }
            Event::Respawn => r.parts.ring(tune::START_X, tune::FIELD_H as f32 - 4.0, 3.0, 24, TEAL_C),
            Event::GameOver => r.parts.ring(w.player.x, w.player.y, 4.0, 60, render::palette::CREAM),
            // much bigger than what died, and it shatters into its own pixels
            Event::Kill { x, y, kind } => {
                let art = &r.atlas.r.foes;
                let size = kill_size(kind);
                let pixels = if kind == EnemyKind::Hull { &art.pod.pixels } else { &art.sprite(kind, w.frame).pixels };
                // its impact frame: lit bone-bright a moment (not the derelict's hull,
                // and not in a bomb's sweep, which plays as it always has)
                if kind != EnemyKind::Hull && w.bomb.is_none() {
                    let sil = art.sprite(kind, w.frame).sil;
                    r.parts.impact(sil, ((x - sil.w * 0.5).round() as i32, (y - sil.h * 0.5).round() as i32), tune::IMPACT[size as usize]);
                }
                r.parts.shatter(x, y, pixels);
                explode(&mut r.fx, &mut r.parts, x, y, size, me, s.reduce_flash, w.frame);
                audio.kill(size);
                if !kind.fodder() {
                    r.parts.smoke(x, y, if kind.heavy() { 14 } else { 5 }, if kind.heavy() { 34.0 } else { 16.0 });
                }
                match kind {
                    EnemyKind::Heavy => {
                        r.parts.ring(x, y, 4.0, 24, render::palette::CREAM);
                        r.fx.shake(tune::SHAKE_DEATH * 0.5, w.hazards.bullets.len(), s.shake);
                    }
                    EnemyKind::Hull => chains.push(Chain { x, y, t: 0 }),
                    _ => {}
                }
            }
            Event::PartBreak { x, y } => {
                r.parts.shatter(x, y, &r.atlas.r.foes.pod.pixels);
                explode(&mut r.fx, &mut r.parts, x, y, Size::Medium, me, s.reduce_flash, w.frame);
                r.parts.smoke(x, y, 6, 16.0);
                audio.blast(Size::Medium);
            }
            Event::Points { x, y, n, how } => {
                r.popups.points(x, y, n, how);
                match how {
                    sim::score::Award::Blank => r.parts.ring(x, y, 2.0, 12, TEAL_C),
                    sim::score::Award::Grade(_, g) => {
                        let c = if g == sim::score::Grade::S { TEAL_C } else { render::palette::CREAM };
                        r.parts.ring(x, y, 5.0, 26, c);
                        if g == sim::score::Grade::S {
                            r.fx.aberrate(if s.reduce_flash { 0.4 } else { 1.0 });
                        }
                    }
                    _ => {}
                }
            }
            Event::Extend => {
                r.popups.extend();
                r.parts.ring(p.x, p.y, 3.0, 30, TEAL_C);
            }
            Event::Crack { x, y } => {
                explode(&mut r.fx, &mut r.parts, x, y - 6.0, Size::Large, me, s.reduce_flash, w.frame);
                r.parts.ring(x, y, 5.0, 26, SIGNAL_C);
                r.parts.smoke(x, y, 16, 60.0);
                r.fx.surge(if s.reduce_flash { 0.4 } else { 1.4 }, 10);
                audio.blast(Size::Large);
            }
            _ => {}
        }
    }
    // never shake while enemy bullets are live
    if w.hazards.bullets.len() > 0 {
        r.fx.calm();
    }
    // the stress scenes: thousands of particles and a blast every so often
    if w.hazards.preset().fill > 0 {
        r.parts.fill(tune::FX_STRESS_PARTICLES, tune::FIELD_W as f32, tune::FIELD_H as f32);
        if w.frame % tune::FX_STRESS_EVERY == 0 {
            let k = w.frame / tune::FX_STRESS_EVERY;
            let (x, y) = (40.0 + (k * 97 % 190) as f32, 40.0 + (k * 53 % 140) as f32);
            explode(&mut r.fx, &mut r.parts, x, y, Size::ALL[(k % 3) as usize + 1], me, s.reduce_flash, w.frame);
        }
    }
}

/// What the HUD needs to know about Stage 1.
fn stage_view(w: &World, s: &sim::stage1::Stage) -> hud::StageView {
    let boss = w.foes.list.iter().find(|e| e.kind == EnemyKind::Hull && e.alive()).map(|e| {
        let mut t = [0.0; 3];
        for (i, q) in e.parts[1..].iter().take(3).enumerate() {
            t[i] = q.hp / q.max_hp;
        }
        (e.parts[0].hp / e.parts[0].max_hp, t)
    });
    let warn_at = sim::stage1::frames(tune::SECTIONS[tune::WARNING_SECTION].0);
    use sim::boss::Act;
    let jaws = w.boss.as_ref().filter(|b| !matches!(b.act, Act::Gone | Act::Dying | Act::Falling)).map(|b| {
        let [_, _, peak, done] = tune::JAWS_INTRO;
        let fill = if b.act == Act::Intro { (b.t.saturating_sub(peak) as f32 / (done - peak) as f32).min(1.0) } else { 1.0 };
        let calm = (b.act == Act::Break).then(|| b.t as f32 / tune::JAWS_BREAK as f32);
        hud::JawsView { bar: b.bar(), phase: if b.act == Act::Intro { 0 } else { b.phase }, left: b.time_left(), fill, calm }
    });
    hud::StageView {
        t: s.t,
        section: s.section,
        killed: s.killed,
        spawned: s.spawned,
        over: s.over,
        boss,
        warned: (s.t >= warn_at).then(|| s.t - warn_at),
        jaws,
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(arg) = std::env::var("PG_SIM") {
        debug::sim_report(&arg);
        return;
    }
    let conf = macroquad::conf::Conf {
        miniquad_conf: window_conf(),
        default_filter_mode: FilterMode::Nearest,
        draw_call_vertex_capacity: tune::DRAW_CALL_VERTICES,
        draw_call_index_capacity: tune::DRAW_CALL_INDICES,
        ..Default::default()
    };
    macroquad::Window::from_config(conf, amain());
}

async fn amain() {
    let hooks = debug::hooks();
    #[cfg(not(target_arch = "wasm32"))]
    if hooks.wav {
        audio::dump();
        return;
    }
    let mut renderer = Renderer::new();
    if hooks.sheet {
        debug::export_sheet(&renderer);
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    if hooks.fx_sheet {
        sheet::export_fx(&mut renderer);
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    if hooks.mockup {
        sheet::export_mockup(&mut renderer);
        return;
    }

    // a frame to look at while the sounds are synthesized; it also draws enough to
    // create every batch buffer up front, so no explosion ever waits on one
    let lay = Renderer::layout();
    renderer.begin(&lay);
    renderer.prewarm();
    renderer.begin(&lay);
    renderer.post(&lay, &PostParams { field: IRect::new(0, 0, lay.cw, lay.ch), time: 0.0 });
    hud::text(&renderer.atlas, "P I L G R I M", lay.cw / 2 - 38, lay.ch / 2 - 4, 1, 1, render::palette::ASH);
    renderer.end(&lay);
    next_frame().await;

    let settings = Settings::load();
    let mut audio = Audio::load().await;
    audio.muted = settings.muted;
    audio.volumes(settings.music_vol, settings.sound_vol);
    if matches!(hooks.scene, Some(SceneReq::Boss(p)) if p > 0) {
        audio.music(true);
    }
    let seed = hooks.seed.unwrap_or_else(|| (miniquad::date::now() * 1000.0) as u64);
    let scripted = !hooks.shots.is_empty();
    let last_shot = hooks.shots.iter().copied().max().unwrap_or(0);

    let mut settings = settings;
    settings.view.autopilot = hooks.auto;
    settings.view.invincible = hooks.invincible;
    let mut game = Game::new(seed, hooks.scene, settings);
    let mut bot = sim::bot::Bot::new();
    if let (true, Scene::Stage { menu, .. }) = (hooks.menu, &mut game.scene) {
        *menu = ITEMS.iter().position(|i| *i == Item::Focus);
    }
    let mut input = Input::new();
    input.analog = game.settings.analog;
    let mut clock = clock::Clock::new();
    let mut perf = debug::Perf::new();
    let mut keys = debug::Keys::default();
    game.settings.view.fps |= hooks.perf;
    let mut fullscreen = start_fullscreen();
    let mut mode_known = false;
    let mut dip_skip = false;
    let mut slow_n = 0u32;
    let mut frame: u64 = 0;
    let mut ended_at: Option<u64> = None;
    let started = miniquad::date::now();
    let mut perf_at = started;
    // closing the window comes through the loop (so PG_PERF can report)
    prevent_quit();

    loop {
        if is_quit_requested() {
            break;
        }
        let t0 = miniquad::date::now();
        let alt = is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt);
        let fs_key = is_key_pressed(KeyCode::F11) || (alt && is_key_pressed(KeyCode::Enter));
        if fs_key {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
            game.settings.fullscreen = fullscreen;
            game.settings.save();
        }
        let f7 = keys.read(&mut game.settings.view);
        if std::mem::take(&mut keys.skip) {
            if let Scene::Stage { world, .. } = &mut game.scene {
                world.skip_section();
                game.chains.clear();
            }
        }
        game.sync_view();
        renderer.gray = game.settings.view.gray;
        if keys.sheet {
            keys.sheet = false;
            debug::export_sheet(&renderer);
        }

        let lay = Renderer::layout();
        let (held, mut ui) = input.poll(&lay, game.settings.touch_sens);
        if fs_key {
            ui.confirm = false; // Alt+Enter is for the window, not the menu
        }
        mode_known |= ui.confirm || ui.tap.is_some() || held.fire || held.mv != (0.0, 0.0) || input.mode != Mode::Keys;
        if input.connected {
            game.say("CONTROLLER CONNECTED");
        }
        game.handle_ui(&ui, &lay, &mut input, &mut audio);
        game.step_pattern(ui.pattern + f7);

        // the fixed-step sim: scripted screenshot runs advance exactly one tick per frame
        let dt = get_frame_time();
        let ticks = keys.filter(if scripted { 1 } else { clock.advance(dt as f64) }, game.settings.view.slowmo);
        for _ in 0..ticks {
            // the bomb's slow-motion dip: half the ticks run while it lasts; the boss's
            // death: a quarter
            if let Scene::Stage { menu: None, over: None, .. } = &game.scene {
                if game.stop > 0 {
                    game.stop -= 1;
                    continue;
                }
            }
            if let Scene::Stage { world, menu: None, .. } = &game.scene {
                if world.slow > 0 {
                    slow_n += 1;
                    if slow_n % 4 != 0 {
                        continue;
                    }
                } else if world.dip > 0 {
                    dip_skip = !dip_skip;
                    if dip_skip {
                        continue;
                    }
                }
            }
            match &mut game.scene {
                Scene::Title { t, idle, .. } => {
                    *t += 1;
                    *idle += 1;
                }
                Scene::Options { t, .. } => *t += 1,
                // the stage is over: everything holds still behind the panel (the
                // tally counts up), but the smoke and light of whatever just happened
                // play out
                Scene::Stage { world, over: Some(_), .. } if world.stage.as_ref().is_some_and(|s| s.over) => {
                    renderer.parts.tick((world.player.x, world.player.y));
                    renderer.popups.tick();
                    renderer.fx.tick();
                    game.end_t += 1;
                    if let Some((id, vol)) = tally_cue(game.end_t, game.record) {
                        audio.play(id, vol);
                    }
                }
                Scene::Stage { menu: None, over: Some(_), .. } => game.end_t += 1,
                Scene::Stage { world, menu: None, over } => {
                    let (drag, bomb, slam) = input.take();
                    // HOLD TO BEAM: fire held a moment becomes the beam
                    game.hold_t = if held.fire { game.hold_t + 1 } else { 0 };
                    let focus = focused(held.focus, game.settings.hold_beam, game.hold_t);
                    let bot_flies = game.settings.view.autopilot || game.run == Run::Demo;
                    let mut c = if bot_flies { bot.decide(world) } else { Controls { mv: held.mv, drag, fire: held.fire, focus, bomb, slam } };
                    if let Some(n) = hooks.kit.filter(|&n| world.kit.level < n && world.alive()) {
                        world.give_kit(n);
                    }
                    if let Some(n) = hooks.rush {
                        world.rush = world.rush.max(n);
                    }
                    if hooks.bomb_at == Some(world.frame + 1) {
                        c.bomb = true;
                    }
                    if hooks.die_at == Some(world.frame + 1) {
                        world.debug_strike();
                    }
                    // (counted in screen frames, like PG_SHOTS: the stage's clock starts late in boss practice)
                    if hooks.kill_at == Some(frame) {
                        world.skip_section();
                    }
                    world.tick(c);
                    if hooks.boom.is_some() && world.frame == 30 {
                        game.boom = hooks.boom.map(|i| Size::ALL[i]);
                    }
                    renderer.parts.tick((world.player.x, world.player.y));
                    renderer.popups.tick();
                    renderer.fx.tick();
                    effects(world, &mut renderer, &game.settings, &mut audio, &mut game.chains);
                    boss_effects(world, &mut renderer);
                    boss_moments(world, &mut renderer, &game.settings, &mut audio);
                    chain_effects(&mut game.chains, world, &mut renderer, &game.settings, &mut audio);
                    let cleared = world.stage.as_ref().is_some_and(|s| s.over);
                    if (world.life == Life::GameOver || cleared) && over.is_none() {
                        *over = Some(0);
                        ended_at = Some(frame);
                        // a new best is saved the moment the run ends
                        game.end_t = 0;
                        game.record = new_record(&mut game.settings, game.run, world.score.points);
                    }
                    // reaching the boss unlocks Boss Practice
                    if game.run == Run::Stage && !game.settings.boss_unlocked && world.events.contains(&Event::BossArrive) {
                        game.settings.boss_unlocked = true;
                        game.settings.save();
                    }
                    if let Some(size) = game.boom.take() {
                        let me = (world.player.x, world.player.y);
                        explode(&mut renderer.fx, &mut renderer.parts, tune::FIELD_W as f32 * 0.5, 150.0, size, me, game.settings.reduce_flash, world.frame);
                        audio.blast(size);
                    }
                    audio.handle(world);
                    game.stop = game.stop.max(hitstop(&world.events, world.bomb.is_some()));
                    if world.events.contains(&Event::NoBomb) {
                        game.toast = Some(("NO BOMBS LEFT".into(), tune::TOAST_FRAMES / 2));
                    }
                    world.events.clear();
                }
                Scene::Stage { .. } => {}
            }
            if let Some((_, left)) = &mut game.toast {
                *left = left.saturating_sub(1);
            }
        }
        let touched = ui.confirm || ui.back || ui.pause || ui.tap.is_some() || held.fire || held.mv != (0.0, 0.0);
        if game.attract(touched) {
            audio.silence();
            input.release();
        }
        audio.update(dt, game.paused());

        // the scene, then the post pass, then the HUD on top
        renderer.begin(&lay);
        let time = game.sim_frame() as f32 / tune::TICK_HZ as f32;
        match &game.scene {
            Scene::Title { t, sel, .. } => {
                hud::title_bg(&renderer, &lay, *t);
                renderer.post(&lay, &PostParams { field: IRect::new(0, 0, lay.cw, lay.ch), time });
                let items = hud::title_items(game.settings.boss_unlocked);
                hud::title(&renderer, &lay, *t, mode_known.then_some(input.mode), &items, *sel, game.settings.hiscore);
            }
            Scene::Options { t, sel } => {
                hud::title_bg(&renderer, &lay, *t);
                renderer.post(&lay, &PostParams { field: IRect::new(0, 0, lay.cw, lay.ch), time });
                hud::options(&renderer, &lay, &game.settings, input.mode, *sel);
            }
            Scene::Stage { world, menu, over } => {
                let wrecks: Vec<(f32, f32, f32)> = game.chains.iter().map(|c| (c.x, c.y, c.t as f32 / Chain::end() as f32)).collect();
                render::scene::draw(&renderer, &lay, world, game.settings.view.hitboxes, game.settings.hitbox_always, &wrecks);
                renderer.post(&lay, &PostParams { field: lay.field, time });
                let view = HudView {
                    settings: &game.settings,
                    mode: input.mode,
                    menu: *menu,
                    toast: game.toast.as_ref().map(|(m, l)| (m.as_str(), *l)),
                    bomb_flash: input.bomb_flash,
                    slam_flash: input.slam_flash,
                    lives: world.lives,
                    bombs: world.bombs,
                    rush: world.rush,
                    kit: hud::KitView { level: world.kit.level, charge: world.kit.charge() },
                    style: world.stage.as_ref().map(|_| hud::StyleView { rank: world.style.rank(), fill: world.style.fill(), mult: world.style.mult(), since: world.style.since, rose: world.style.rose }),
                    over: *over,
                    pattern: world.hazards.preset,
                    bullets: world.hazards.bullets.len(),
                    hits: world.deaths,
                    grazes: world.grazes,
                    stage: world.stage.as_ref().map(|s| stage_view(world, s)),
                    score: world.score.points,
                    hiscore: game.settings.hiscore,
                    end_t: game.end_t,
                    record: game.record,
                    tally: world.score.tally,
                    demo: game.run == Run::Demo,
                    ranked: game.run == Run::Stage,
                };
                hud::stage(&renderer, &lay, &view);
            }
        }
        if game.settings.view.fps {
            let (shots, bullets, sim_seed) = match &game.scene {
                Scene::Stage { world, .. } => (world.shots.len(), world.hazards.bullets.len(), world.seed),
                _ => (0, 0, game.seed),
            };
            perf.draw(&renderer.atlas, &lay, game.sim_frame(), sim_seed, shots, bullets, renderer.parts.len());
        }
        renderer.end(&lay);
        perf.record(dt, (miniquad::date::now() - t0) as f32, ticks);
        // long runs with PG_PERF: a line every 30 s
        if hooks.perf && !scripted && miniquad::date::now() - perf_at >= 30.0 {
            perf_at = miniquad::date::now();
            let (bullets, foes) = match &game.scene {
                Scene::Stage { world, .. } => (world.hazards.bullets.len(), world.foes.list.len()),
                _ => (0, 0),
            };
            println!("{:>5.0} s  {}  particles {}  bullets {}  enemies {}", perf_at - started, perf.interval(), renderer.parts.len(), bullets, foes);
        }
        // the bot plays on forever (PG_AUTO): two seconds after a game over or the stage's
        // tally, RETRY
        if hooks.auto && ended_at.is_some_and(|f| frame >= f + 120 + tune::TALLY_DONE as u64) {
            ended_at = None;
            game.retry();
        }

        frame += 1;
        if hooks.shots.contains(&frame) {
            debug::save_shot(frame, hooks.gray);
        }
        if scripted && frame >= last_shot {
            if hooks.perf {
                let bullets = match &game.scene {
                    Scene::Stage { world, .. } => world.hazards.bullets.len(),
                    _ => 0,
                };
                println!("{}  bullets {}", perf.summary(), bullets);
            }
            break;
        }
        next_frame().await;
    }
    if hooks.perf && !scripted {
        println!("{}", perf.summary());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Big kills land with a moment's freeze, fodder never stops the flow, many kills
    /// at once stop once (as long as the biggest), no stop feels like lag, and a
    /// bomb's sweep never stops.
    #[test]
    fn big_kills_land_with_a_hitstop() {
        let kill = |kind| Event::Kill { x: 0.0, y: 0.0, kind };
        assert_eq!(hitstop(&[kill(EnemyKind::Drone), kill(EnemyKind::Courier), kill(EnemyKind::Usher)], false), 0);
        assert!(hitstop(&[kill(EnemyKind::Gunship)], false) > 0);
        assert!(hitstop(&[kill(EnemyKind::Heavy)], false) > hitstop(&[kill(EnemyKind::Gunship)], false));
        assert_eq!(hitstop(&[kill(EnemyKind::Drone), kill(EnemyKind::Heavy), kill(EnemyKind::Gunship)], false), hitstop(&[kill(EnemyKind::Heavy)], false));
        assert!(hitstop(&[Event::BossBreak { x: 0.0, y: 0.0, timeout: false }], false) >= hitstop(&[kill(EnemyKind::Heavy)], false));
        assert_eq!(hitstop(&[Event::Graze { x: 0.0, y: 0.0 }], false), 0);
        assert_eq!(hitstop(&[kill(EnemyKind::Lancer), kill(EnemyKind::Remora), kill(EnemyKind::Debris)], false), 0, "the Swarm's fodder never stops it");
        // a slam strike lands with a short pause; its aim plays at full speed (no stop)
        assert_eq!(hitstop(&[Event::Slam { x: 0.0, y: 0.0, link: 1 }], false), tune::HITSTOP_SLAM);
        assert!(tune::HITSTOP_SLAM > 0 && tune::HITSTOP_SLAM <= 8);
        assert!(tune::HITSTOP.iter().chain([&tune::HITSTOP_BOSS]).all(|&f| f <= 8));
        // a bomb's sweep never stops: its ring would freeze on the screen
        assert_eq!(hitstop(&[kill(EnemyKind::Heavy), Event::BossBreak { x: 0.0, y: 0.0, timeout: false }], true), 0);
    }

    fn ui() -> Ui {
        Ui::default()
    }

    #[test]
    fn pg_scene_starts_the_stage_partway_in() {
        let g = Game::new(1, Some(SceneReq::Stage(2.5)), Settings::default());
        assert_eq!(g.sim_frame(), 149, "the next tick is 2.5 s in");
        assert!(matches!(&g.scene, Scene::Stage { world, .. } if world.stage.is_some()));
        let g = Game::new(1, Some(SceneReq::Shots(500)), Settings::default());
        assert!(matches!(&g.scene, Scene::Stage { world, .. } if world.fill_shots == 500 && world.stage.is_none()));
    }

    #[test]
    fn the_title_starts_each_kind_of_run() {
        use hud::TitleItem::*;
        let lay = Layout::fit(1280, 720);
        assert_eq!(hud::title_items(false), vec![Stage, Sandbox, Options]);
        assert_eq!(hud::title_items(true), vec![Stage, Practice, Sandbox, Options], "Boss Practice once unlocked");
        let mut g = Game::new(1, None, Settings::default());
        g.start_stage(0.0);
        assert!(g.run == Run::Stage && matches!(&g.scene, Scene::Stage { world, .. } if world.stage.is_some() && world.frame == 0));
        g.start_boss(0);
        assert!(g.run == Run::Practice && matches!(&g.scene, Scene::Stage { world, .. } if world.clock() >= sim::stage1::frames(tune::JAWS_AT) as u64 - 1));
        g.retry();
        assert_eq!(g.run, Run::Practice, "a retry plays the same kind of run");
        g.start_sandbox();
        assert!(g.run == Run::Sandbox && matches!(&g.scene, Scene::Stage { world, .. } if world.stage.is_none()));
        g.start_stage(40.0);
        assert_eq!(g.run, Run::Practice, "a start partway in doesn't count for the hi-score");
        let (cx, y0, rh) = hud::title_rows(&lay);
        assert_eq!(hud::title_tap(&lay, 4, cx as f32, (y0 + rh + 2) as f32), Some(1));
        assert_eq!(hud::title_tap(&lay, 3, cx as f32, (y0 + 3 * rh + 2) as f32), None);
    }

    #[test]
    fn the_title_plays_a_demo_when_left_alone_and_any_button_ends_it() {
        let mut g = Game::new(1, None, Settings::default());
        if let Scene::Title { idle, .. } = &mut g.scene {
            *idle = tune::ATTRACT_AFTER - 1;
        }
        assert!(!g.attract(false));
        assert!(matches!(g.scene, Scene::Title { .. }), "not yet");
        if let Scene::Title { idle, .. } = &mut g.scene {
            *idle = tune::ATTRACT_AFTER;
        }
        assert!(!g.attract(false));
        assert!(g.run == Run::Demo && matches!(&g.scene, Scene::Stage { world, .. } if world.stage.is_some()), "the demo plays Stage 1");
        assert!(!g.attract(false), "it plays on");
        assert!(g.attract(true), "a button ends it");
        assert!(matches!(g.scene, Scene::Title { idle: 0, .. }));
        g.start_demo();
        if let Scene::Stage { world, .. } = &mut g.scene {
            world.frame = tune::DEMO_LEN;
        }
        assert!(g.attract(false), "and it ends by itself after a while");
    }

    #[test]
    fn hold_to_beam_turns_a_held_fire_into_the_beam() {
        assert!(!focused(false, true, tune::HOLD_BEAM), "a tap is the wide shot");
        assert!(focused(false, true, tune::HOLD_BEAM + 1), "held, the beam");
        assert!(!focused(false, false, 1000), "not with the focus button scheme");
        assert!(focused(true, false, 0) && focused(true, true, 0), "the focus button always works");
    }

    #[test]
    fn only_a_full_stage_run_sets_the_hi_score() {
        let mut s = Settings::default();
        assert!(new_record(&mut s, Run::Stage, 1_000));
        assert_eq!(s.hiscore, 1_000);
        assert!(!new_record(&mut s, Run::Stage, 999), "not a record");
        assert!(!new_record(&mut s, Run::Stage, 1_000), "a tie isn't either");
        for run in [Run::Practice, Run::Sandbox, Run::Demo] {
            assert!(!new_record(&mut s, run, 5_000), "{run:?} doesn't count");
        }
        assert_eq!(s.hiscore, 1_000);
    }

    #[test]
    fn the_tally_ticks_slams_and_sings() {
        let cues: Vec<(u32, Id)> = (0..tune::TALLY_DONE + 60).filter_map(|e| tally_cue(e, true).map(|c| (e, c.0))).collect();
        assert_eq!(cues.iter().filter(|c| c.1 == Id::Slam).count(), 1, "one slam, for the rank");
        assert!(cues.iter().any(|c| c == &(tune::TALLY_RANK, Id::Slam)));
        assert!(cues.iter().any(|c| c.1 == Id::Extend), "a record sings");
        assert!(cues.iter().filter(|c| c.1 == Id::Tick).count() >= 5, "a tick per line, and the count-up");
        assert!(!(0..tune::TALLY_DONE + 60).any(|e| tally_cue(e, false).is_some_and(|c| c.0 == Id::Extend)), "no record, no jingle");
    }

    #[test]
    fn the_stage_ignores_the_sandbox_switches() {
        let mut g = Game::new(1, Some(SceneReq::Stage(0.0)), Settings::default());
        g.step_pattern(1);
        g.settings.view.pattern = 3;
        g.sync_view();
        assert!(matches!(&g.scene, Scene::Stage { world, .. } if world.hazards.preset == 0));
    }

    #[test]
    fn the_menu_moves_changes_and_closes() {
        let lay = Layout::fit(1280, 720);
        let mut s = Settings::default();
        let mut sel = 0;
        assert_eq!(menu_input(&mut sel, &Ui { nav: (0, -1), ..ui() }, &lay, &mut s, &ITEMS), MenuAct::Nothing);
        assert_eq!(ITEMS[sel], Item::Quit, "wraps around");
        sel = ITEMS.iter().position(|i| *i == Item::Focus).unwrap();
        assert_eq!(menu_input(&mut sel, &Ui { nav: (1, 0), ..ui() }, &lay, &mut s, &ITEMS), MenuAct::Changed);
        assert_eq!(s.tuning.focus_ratio, 0.60);
        assert_eq!(menu_input(&mut sel, &Ui { back: true, ..ui() }, &lay, &mut s, &ITEMS), MenuAct::Resume);
        sel = 0;
        assert_eq!(menu_input(&mut sel, &Ui { confirm: true, ..ui() }, &lay, &mut s, &ITEMS), MenuAct::Resume);
        sel = ITEMS.len() - 1;
        assert_eq!(menu_input(&mut sel, &Ui { confirm: true, ..ui() }, &lay, &mut s, &ITEMS), MenuAct::Quit);
        // the options: BACK leaves, values step
        let mut sel = menu::OPTIONS.len() - 1;
        assert_eq!(menu_input(&mut sel, &Ui { confirm: true, ..ui() }, &lay, &mut s, &menu::OPTIONS), MenuAct::Resume);
        sel = 0;
        assert_eq!(menu_input(&mut sel, &Ui { nav: (-1, 0), ..ui() }, &lay, &mut s, &menu::OPTIONS), MenuAct::Changed);
        assert_eq!(s.music_vol, tune::VOLUME_DEFAULT - 1);
    }

    #[test]
    fn the_sandbox_switches_reach_the_running_stage() {
        let mut g = Game::new(1, Some(SceneReq::Sandbox(3)), Settings::default());
        assert!(matches!(&g.scene, Scene::Stage { world, .. } if world.hazards.preset == 3));
        g.step_pattern(1);
        assert!(matches!(&g.scene, Scene::Stage { world, .. } if world.hazards.preset == 4));
        g.settings.view.invincible = true;
        g.sync_view();
        assert!(matches!(&g.scene, Scene::Stage { world, .. } if world.invincible));
        g.step_pattern(-5);
        assert_eq!(g.settings.view.pattern, 3, "one step at a time");
        g.settings.view.pattern = 0;
        g.step_pattern(-1);
        assert_eq!(g.settings.view.pattern, tune::SANDBOX.len() - 1, "wraps around");
    }

    #[test]
    fn tapping_a_value_steps_it() {
        let lay = Layout::fit(390, 844);
        let m = hud::menu_box(&lay, ITEMS.len());
        let row = ITEMS.iter().position(|i| *i == Item::Speed).unwrap();
        let y = (m.row0 + row as i32 * m.row_h + 3) as f32;
        let mut s = Settings::default();
        let mut sel = 0;
        let tap = Ui { tap: Some(((m.val1 - 2) as f32, y)), ..ui() };
        assert_eq!(menu_input(&mut sel, &tap, &lay, &mut s, &ITEMS), MenuAct::Changed);
        assert_eq!((sel, s.tuning.speed), (row, tune::MOVE_SPEED + tune::SPEED_RANGE.2));
    }
}
