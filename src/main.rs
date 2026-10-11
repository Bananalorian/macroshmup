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
mod rumble;
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
    /// The title, `t` ticks after it appeared, with `sel` chosen.
    Title { t: u64, sel: usize },
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

struct Game {
    scene: Scene,
    seed: u64,
    settings: Settings,
    toast: Option<(String, u32)>,
    /// Debug: extra shots kept alive in the stage (PG_SCENE=shots:<n>).
    fill_shots: usize,
    /// BOOM TEST asked for an explosion of this size.
    boom: Option<Size>,
    chains: Vec<Chain>,
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

/// Apply one frame of input to the pause menu.
fn menu_input(sel: &mut usize, ui: &Ui, lay: &Layout, s: &mut Settings) -> MenuAct {
    if ui.back || ui.pause {
        return MenuAct::Resume;
    }
    let n = ITEMS.len() as i32;
    *sel = (*sel as i32 + ui.nav.1).rem_euclid(n) as usize;
    let activate = |sel: usize, s: &mut Settings| match ITEMS[sel] {
        Item::Resume => MenuAct::Resume,
        Item::Quit => MenuAct::Quit,
        Item::Boom => MenuAct::Boom,
        item => {
            menu::adjust(s, item, 1);
            MenuAct::Changed
        }
    };
    if ui.nav.0 != 0 && ITEMS[*sel].adjustable() {
        menu::adjust(s, ITEMS[*sel], ui.nav.0.signum());
        return MenuAct::Changed;
    }
    if ui.confirm {
        return activate(*sel, s);
    }
    if let Some((x, y)) = ui.tap {
        if let Some((row, step)) = hud::menu_tap(lay, x, y) {
            *sel = row;
            if step != 0 {
                menu::adjust(s, ITEMS[row], step);
                return MenuAct::Changed;
            }
            return activate(row, s);
        }
    }
    MenuAct::Nothing
}

impl Game {
    fn new(seed: u64, req: Option<SceneReq>, settings: Settings) -> Game {
        let mut g = Game { scene: Scene::Title { t: 0, sel: 0 }, seed, settings, toast: None, fill_shots: 0, boom: None, chains: vec![] };
        match req {
            None | Some(SceneReq::Title) => {}
            Some(SceneReq::Stage(secs)) => g.start_stage(secs),
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

    /// Stage 1, `secs` seconds in.
    fn start_stage(&mut self, secs: f32) {
        let mut w = World::stage_at(self.seed, self.settings.tuning, secs);
        w.invincible = self.settings.view.invincible;
        self.chains.clear();
        self.scene = Scene::Stage { world: Box::new(w), menu: None, over: None };
    }

    fn start_sandbox(&mut self) {
        let mut w = World::new(self.seed, self.settings.tuning);
        w.fill_shots = self.fill_shots;
        w.set_pattern(self.settings.view.pattern);
        w.invincible = self.settings.view.invincible;
        self.chains.clear();
        self.scene = Scene::Stage { world: Box::new(w), menu: None, over: None };
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
            Scene::Title { t, .. } => *t,
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
        match &mut self.scene {
            Scene::Title { sel, .. } => {
                let n = hud::TITLE_ITEMS.len() as i32;
                let before = *sel;
                *sel = (*sel as i32 + ui.nav.1).rem_euclid(n) as usize;
                let mut choice = ui.confirm.then_some(*sel);
                if let Some((x, y)) = ui.tap {
                    // a tap on a row picks it; anywhere else starts what's selected
                    choice = Some(hud::title_tap(lay, x, y).unwrap_or(*sel));
                }
                if *sel != before {
                    audio.play(Id::Select, 0.5);
                }
                if let Some(i) = choice {
                    audio.play(Id::Select, 1.0);
                    input.release();
                    if i == 0 {
                        self.start_stage(0.0);
                    } else {
                        self.start_sandbox();
                    }
                }
            }
            Scene::Stage { over: Some(sel), world, .. } => {
                let n = hud::OVER_ITEMS.len() as i32;
                let before = *sel;
                let cleared = world.stage.as_ref().is_some_and(|s| s.over);
                let staged = world.stage.is_some();
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
                        if staged {
                            self.start_stage(0.0);
                        } else {
                            self.start_sandbox();
                        }
                    }
                    Some(_) => {
                        audio.silence();
                        input.release();
                        self.scene = Scene::Title { t: 0, sel: if staged { 0 } else { 1 } };
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
                let act = menu_input(sel, ui, lay, &mut self.settings);
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
                        let staged = matches!(&self.scene, Scene::Stage { world, .. } if world.stage.is_some());
                        self.scene = Scene::Title { t: 0, sel: if staged { 0 } else { 1 } };
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

fn window_conf() -> Conf {
    let (w, h) = debug::window_size().unwrap_or((1280, 720));
    Conf {
        window_title: "PILGRIM".to_owned(),
        window_width: w,
        window_height: h,
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
        if e.kind != EnemyKind::Drone && q.hp < q.max_hp * 0.5 && w.frame % 5 == 0 {
            let (x, y) = r.parts.somewhere(e.x, e.y, bw * 0.6, bh * 0.6);
            r.parts.wound(x, y);
        }
        if e.path.entering(e.t) && e.kind == EnemyKind::Hull && w.frame % 2 == 0 {
            let (x, y) = r.parts.somewhere(e.x, e.y, bw, bh);
            r.parts.flake(x, y);
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

/// Feed one tick's events to the effects, the blasts' sounds, the rumble and the phone buzz.
fn effects(w: &World, r: &mut Renderer, input: &mut Input, s: &Settings, audio: &mut Audio, chains: &mut Vec<Chain>) {
    let p = &w.player;
    let me = (p.x, p.y);
    foe_effects(w, r);
    let flying = matches!(w.life, Life::Alive | Life::Respawning { .. });
    if flying && w.frame % 2 == 0 {
        r.parts.trail(p.x, p.y + 23.0, p.thrust() || !w.alive());
    }
    if flying && w.frame % 4 == 0 {
        for (gx, gy) in p.pod_guns() {
            r.parts.trail(gx, gy + 13.0, false);
        }
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
            Event::Hit { x, y, beam } if !beam || w.frame % 3 == 0 => r.parts.hit(x, y, beam),
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
                let (size, pixels) = match kind {
                    EnemyKind::Drone => (Size::Small, &art.drone.pixels),
                    EnemyKind::Gunship => (Size::Medium, &art.gunship.pixels),
                    EnemyKind::Buoy => (Size::Medium, &art.buoy[0].pixels),
                    EnemyKind::Heavy => (Size::Large, &art.heavy.pixels),
                    EnemyKind::Hull => (Size::Large, &art.turret.pixels),
                };
                r.parts.shatter(x, y, pixels);
                explode(&mut r.fx, &mut r.parts, x, y, size, me, s.reduce_flash, w.frame);
                audio.blast(size);
                if kind != EnemyKind::Drone {
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
                r.parts.shatter(x, y, &r.atlas.r.foes.turret.pixels);
                explode(&mut r.fx, &mut r.parts, x, y, Size::Medium, me, s.reduce_flash, w.frame);
                r.parts.smoke(x, y, 6, 16.0);
                audio.blast(Size::Medium);
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
        if let Some(k) = rumble::for_event(e).and_then(|k| rumble::scaled(k, s.rumble)) {
            input.pads.rumble(k.0, k.1, k.2);
        }
        if input.mode == Mode::Touch && s.rumble != storage::Rumble::Off {
            if let Some(ms) = rumble::buzz(e) {
                input.pads.vibrate(ms);
            }
        }
    }
    if let Some(k) = rumble::held(w).and_then(|k| rumble::scaled(k, s.rumble)) {
        input.pads.rumble(k.0, k.1, k.2);
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
    let warn_at = sim::stage1::frames(tune::SECTIONS[tune::SECTIONS.len() - 1].0);
    hud::StageView {
        t: s.t,
        section: s.section,
        killed: s.killed,
        spawned: s.spawned,
        bombs_used: w.bombs_used,
        over: s.over,
        boss,
        warned: (s.t >= warn_at).then(|| s.t - warn_at),
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
    let seed = hooks.seed.unwrap_or_else(|| (miniquad::date::now() * 1000.0) as u64);
    let scripted = !hooks.shots.is_empty();
    let last_shot = hooks.shots.iter().copied().max().unwrap_or(0);

    let mut settings = settings;
    // automated runs (screenshots, perf) never touch a real controller's motors
    if scripted {
        settings.rumble = storage::Rumble::Off;
    }
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
    let mut fullscreen = false;
    let mut mode_known = false;
    let mut dip_skip = false;
    let mut frame: u64 = 0;
    // closing the window comes through the loop, so the rumble can be stopped first
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
            if let Some(k) = rumble::scaled(tune::RUMBLE_CONNECT, game.settings.rumble) {
                input.pads.rumble(k.0, k.1, k.2);
            }
        }
        game.handle_ui(&ui, &lay, &mut input, &mut audio);
        game.step_pattern(ui.pattern + f7);

        // the fixed-step sim: scripted screenshot runs advance exactly one tick per frame
        let dt = get_frame_time();
        let ticks = keys.filter(if scripted { 1 } else { clock.advance(dt as f64) }, game.settings.view.slowmo);
        for _ in 0..ticks {
            // the bomb's slow-motion dip: half the ticks run while it lasts
            if let Scene::Stage { world, menu: None, .. } = &game.scene {
                if world.dip > 0 {
                    dip_skip = !dip_skip;
                    if dip_skip {
                        continue;
                    }
                }
            }
            match &mut game.scene {
                Scene::Title { t, .. } => *t += 1,
                // the stage is over: everything holds still behind the panel
                Scene::Stage { world, over: Some(_), .. } if world.stage.as_ref().is_some_and(|s| s.over) => {}
                Scene::Stage { world, menu: None, over } => {
                    let (drag, bomb) = input.take();
                    let mut c = if game.settings.view.autopilot { bot.decide(world) } else { Controls { mv: held.mv, drag, fire: held.fire, focus: held.focus, bomb } };
                    if hooks.bomb_at == Some(world.frame + 1) {
                        c.bomb = true;
                    }
                    if hooks.die_at == Some(world.frame + 1) {
                        world.debug_strike();
                    }
                    world.tick(c);
                    if hooks.boom.is_some() && world.frame == 30 {
                        game.boom = hooks.boom.map(|i| Size::ALL[i]);
                    }
                    renderer.parts.tick((world.player.x, world.player.y));
                    renderer.fx.tick();
                    effects(world, &mut renderer, &mut input, &game.settings, &mut audio, &mut game.chains);
                    chain_effects(&mut game.chains, world, &mut renderer, &game.settings, &mut audio);
                    let cleared = world.stage.as_ref().is_some_and(|s| s.over);
                    if (world.life == Life::GameOver || cleared) && over.is_none() {
                        *over = Some(0);
                    }
                    if let Some(size) = game.boom.take() {
                        let me = (world.player.x, world.player.y);
                        explode(&mut renderer.fx, &mut renderer.parts, 135.0, 150.0, size, me, game.settings.reduce_flash, world.frame);
                        audio.blast(size);
                    }
                    audio.handle(world);
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
        audio.update(dt, game.paused());

        // the scene, then the post pass, then the HUD on top
        renderer.begin(&lay);
        let time = game.sim_frame() as f32 / tune::TICK_HZ as f32;
        match &game.scene {
            Scene::Title { t, sel } => {
                hud::title_bg(&renderer, &lay, *t);
                renderer.post(&lay, &PostParams { field: IRect::new(0, 0, lay.cw, lay.ch), time });
                hud::title(&renderer, &lay, *t, mode_known.then_some(input.mode), *sel);
            }
            Scene::Stage { world, menu, over } => {
                let wrecks: Vec<(f32, f32, f32)> = game.chains.iter().map(|c| (c.x, c.y, c.t as f32 / Chain::end() as f32)).collect();
                render::scene::draw(&renderer, &lay, world, game.settings.view.hitboxes, &wrecks);
                renderer.post(&lay, &PostParams { field: lay.field, time });
                let view = HudView {
                    settings: &game.settings,
                    mode: input.mode,
                    menu: *menu,
                    toast: game.toast.as_ref().map(|(m, l)| (m.as_str(), *l)),
                    bomb_flash: input.bomb_flash,
                    lives: world.lives,
                    bombs: world.bombs,
                    over: *over,
                    pattern: world.hazards.preset,
                    bullets: world.hazards.bullets.len(),
                    hits: world.deaths,
                    grazes: world.grazes,
                    stage: world.stage.as_ref().map(|s| stage_view(world, s)),
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
        println!("{}  rumble commands {}", perf.summary(), input.pads.commands);
    }
    // a controller keeps the last strength it was given: zero it before we go
    input.pads.stop();
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn the_title_starts_the_stage_or_the_sandbox() {
        let lay = Layout::fit(1280, 720);
        let mut g = Game::new(1, None, Settings::default());
        // no audio device in tests: drive the menu through its own logic
        let pick = |g: &mut Game, ui: &Ui| {
            if let Scene::Title { sel, .. } = &mut g.scene {
                *sel = (*sel as i32 + ui.nav.1).rem_euclid(2) as usize;
                if ui.confirm {
                    if *sel == 0 { g.start_stage(0.0) } else { g.start_sandbox() }
                }
            }
        };
        pick(&mut g, &Ui { nav: (0, 1), ..ui() });
        pick(&mut g, &Ui { confirm: true, ..ui() });
        assert!(matches!(&g.scene, Scene::Stage { world, .. } if world.stage.is_none()), "the sandbox");
        let mut g = Game::new(1, None, Settings::default());
        pick(&mut g, &Ui { confirm: true, ..ui() });
        assert!(matches!(&g.scene, Scene::Stage { world, .. } if world.stage.is_some()), "stage 1");
        let (cx, y0, rh) = hud::title_rows(&lay);
        assert_eq!(hud::title_tap(&lay, cx as f32, (y0 + rh + 2) as f32), Some(1));
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
        assert_eq!(menu_input(&mut sel, &Ui { nav: (0, -1), ..ui() }, &lay, &mut s), MenuAct::Nothing);
        assert_eq!(ITEMS[sel], Item::Quit, "wraps around");
        sel = ITEMS.iter().position(|i| *i == Item::Focus).unwrap();
        assert_eq!(menu_input(&mut sel, &Ui { nav: (1, 0), ..ui() }, &lay, &mut s), MenuAct::Changed);
        assert_eq!(s.tuning.focus_ratio, 0.60);
        assert_eq!(menu_input(&mut sel, &Ui { back: true, ..ui() }, &lay, &mut s), MenuAct::Resume);
        sel = 0;
        assert_eq!(menu_input(&mut sel, &Ui { confirm: true, ..ui() }, &lay, &mut s), MenuAct::Resume);
        sel = ITEMS.len() - 1;
        assert_eq!(menu_input(&mut sel, &Ui { confirm: true, ..ui() }, &lay, &mut s), MenuAct::Quit);
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
        let m = hud::menu_box(&lay);
        let row = ITEMS.iter().position(|i| *i == Item::Speed).unwrap();
        let y = (m.row0 + row as i32 * m.row_h + 3) as f32;
        let mut s = Settings::default();
        let mut sel = 0;
        let tap = Ui { tap: Some(((m.val1 - 2) as f32, y)), ..ui() };
        assert_eq!(menu_input(&mut sel, &tap, &lay, &mut s), MenuAct::Changed);
        assert_eq!((sel, s.tuning.speed), (row, tune::MOVE_SPEED + tune::SPEED_RANGE.2));
    }
}
