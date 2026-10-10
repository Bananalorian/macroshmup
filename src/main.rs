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
use render::Renderer;
use render::hud::{self, HudView};
use render::layout::Layout;
use sim::player::Controls;
use sim::{Event, World};
use storage::Settings;
use synth::Id;

enum Scene {
    /// The title, `t` ticks after it appeared.
    Title { t: u64 },
    /// A run. `menu` is the pause menu's selected row while paused.
    Stage { world: Box<World>, menu: Option<usize> },
}

struct Game {
    scene: Scene,
    seed: u64,
    settings: Settings,
    toast: Option<(String, u32)>,
    /// Debug: extra shots kept alive in the stage (PG_SCENE=shots:<n>).
    fill_shots: usize,
}

/// What a frame of menu input asked for.
#[derive(Debug, PartialEq)]
enum MenuAct {
    Nothing,
    Changed,
    Resume,
    Quit,
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
        let mut g = Game { scene: Scene::Title { t: 0 }, seed, settings, toast: None, fill_shots: 0 };
        match req {
            None | Some(SceneReq::Title) => {}
            Some(SceneReq::Stage(secs)) => {
                g.start_stage();
                if let Scene::Stage { world, .. } = &mut g.scene {
                    for _ in 0..(secs * tune::TICK_HZ as f32).round() as u64 {
                        world.tick(Controls::default());
                        world.events.clear();
                    }
                }
            }
            Some(SceneReq::Shots(n)) => {
                g.fill_shots = n;
                g.start_stage();
            }
        }
        g
    }

    fn start_stage(&mut self) {
        let mut w = World::new(self.seed, self.settings.tuning);
        w.fill_shots = self.fill_shots;
        self.scene = Scene::Stage { world: Box::new(w), menu: None };
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
            Scene::Title { t } => *t,
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
            Scene::Title { .. } => {
                if ui.confirm || ui.tap.is_some() {
                    audio.play(Id::Select, 1.0);
                    input.release();
                    self.start_stage();
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
                        self.scene = Scene::Title { t: 0 };
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

/// Feed one tick's events to the particles, the rumble and the phone buzz.
fn effects(w: &World, r: &mut Renderer, input: &mut Input, s: &Settings) {
    let p = &w.player;
    if w.frame % 2 == 0 {
        r.fx.trail(p.x, p.y + 23.0, p.thrust());
    }
    if w.frame % 4 == 0 {
        for (gx, gy) in p.pod_guns() {
            r.fx.trail(gx, gy + 13.0, false);
        }
    }
    for e in &w.events {
        match *e {
            Event::Hit { x, y, beam } if !beam || w.frame % 3 == 0 => r.fx.hit(x, y, beam),
            Event::Break { x, y, big } => r.fx.burst(x, y, big),
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
}

#[macroquad::main(window_conf)]
async fn main() {
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

    // a frame to look at while the sounds are synthesized
    let lay = Renderer::layout();
    renderer.begin(&lay);
    hud::text(&renderer.atlas, "P I L G R I M", lay.cw / 2 - 38, lay.ch / 2 - 4, 1, 1, render::palette::ASH);
    renderer.end(&lay);
    next_frame().await;

    let settings = Settings::load();
    let mut audio = Audio::load().await;
    audio.muted = settings.muted;
    let seed = hooks.seed.unwrap_or_else(|| (miniquad::date::now() * 1000.0) as u64);
    let scripted = !hooks.shots.is_empty();
    let last_shot = hooks.shots.iter().copied().max().unwrap_or(0);

    let mut game = Game::new(seed, hooks.scene, settings);
    if let (true, Scene::Stage { menu, .. }) = (hooks.menu, &mut game.scene) {
        *menu = Some(2);
    }
    let mut input = Input::new();
    input.analog = game.settings.analog;
    let mut clock = clock::Clock::new();
    let mut perf = debug::Perf::new();
    let mut keys = debug::Keys::new(hooks.perf);
    let mut fullscreen = false;
    let mut mode_known = false;
    let mut frame: u64 = 0;

    loop {
        let t0 = miniquad::date::now();
        let alt = is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt);
        let fs_key = is_key_pressed(KeyCode::F11) || (alt && is_key_pressed(KeyCode::Enter));
        if fs_key {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }
        keys.read(&mut renderer.gray);
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

        // the fixed-step sim: scripted screenshot runs advance exactly one tick per frame
        let dt = get_frame_time();
        let ticks = keys.filter(if scripted { 1 } else { clock.advance(dt as f64) });
        for _ in 0..ticks {
            match &mut game.scene {
                Scene::Title { t } => *t += 1,
                Scene::Stage { world, menu: None } => {
                    let (drag, bomb) = input.take();
                    let c = if hooks.auto { debug::autopilot(world) } else { Controls { mv: held.mv, drag, fire: held.fire, focus: held.focus, bomb } };
                    world.tick(c);
                    renderer.fx.tick();
                    effects(world, &mut renderer, &mut input, &game.settings);
                    audio.handle(world);
                    if world.events.contains(&Event::BombStub) {
                        game.toast = Some(("THE BOMB ARRIVES IN V0.5".into(), tune::TOAST_FRAMES));
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

        renderer.begin(&lay);
        match &game.scene {
            Scene::Title { t } => hud::title(&renderer, &lay, *t, mode_known.then_some(input.mode)),
            Scene::Stage { world, menu } => {
                render::scene::draw(&renderer, &lay, world, keys.hitboxes);
                let view = HudView {
                    settings: &game.settings,
                    mode: input.mode,
                    menu: *menu,
                    toast: game.toast.as_ref().map(|(m, l)| (m.as_str(), *l)),
                    bomb_flash: input.bomb_flash,
                };
                hud::stage(&renderer, &lay, &view);
            }
        }
        if keys.perf {
            let (shots, sim_seed) = match &game.scene {
                Scene::Stage { world, .. } => (world.shots.len(), world.seed),
                _ => (0, game.seed),
            };
            perf.draw(&renderer.atlas, &lay, game.sim_frame(), sim_seed, shots, renderer.fx.len());
        }
        renderer.end(&lay);
        perf.record(dt, (miniquad::date::now() - t0) as f32, ticks);

        frame += 1;
        if hooks.shots.contains(&frame) {
            debug::save_shot(frame, hooks.gray);
        }
        if scripted && frame >= last_shot {
            if hooks.perf {
                println!("{}", perf.summary());
            }
            break;
        }
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ui() -> Ui {
        Ui::default()
    }

    #[test]
    fn pg_scene_stage_fast_forwards_in_frames() {
        let g = Game::new(1, Some(SceneReq::Stage(2.5)), Settings::default());
        assert_eq!(g.sim_frame(), 150);
        let g = Game::new(1, Some(SceneReq::Shots(500)), Settings::default());
        assert!(matches!(&g.scene, Scene::Stage { world, .. } if world.fill_shots == 500));
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
