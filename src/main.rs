// No console window behind the game on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! PILGRIM
//! A vertical bullet-hell shmup. Somewhere beyond the event horizon, a small ship,
//! a long way from home.
//!
//! Everything you see is generated in code: the sprite atlas (render/art.rs), the
//! font (render/font.rs) and the starfield (render/bg.rs). The game runs a fixed
//! 60 Hz simulation (sim/) and renders every display frame.

mod clock;
mod debug;
mod render;
mod sim;
mod tune;

use debug::SceneReq;
use macroquad::prelude::*;
use render::Renderer;
use sim::World;

enum Scene {
    /// The title, `t` ticks after it appeared.
    Title { t: u64 },
    /// A run: for now, an empty playfield.
    Stage(World),
}

/// Buttons pressed since the last tick. Read every display frame, consumed by the
/// next tick, so a press on a frame that runs no tick still counts.
#[derive(Clone, Copy, Default)]
struct Pressed {
    start: bool,
    back: bool,
}

impl Pressed {
    fn read(&mut self) {
        self.start |= [KeyCode::Z, KeyCode::Enter, KeyCode::KpEnter, KeyCode::Space].iter().any(|&k| is_key_pressed(k))
            || is_mouse_button_pressed(MouseButton::Left); // a tap counts as a click
        self.back |= is_key_pressed(KeyCode::Escape);
    }
}

struct Game {
    scene: Scene,
    seed: u64,
}

impl Game {
    fn new(seed: u64, req: Option<SceneReq>) -> Game {
        let scene = match req {
            None | Some(SceneReq::Title) => Scene::Title { t: 0 },
            Some(SceneReq::Stage(secs)) => {
                let mut w = World::new(seed);
                for _ in 0..(secs * tune::TICK_HZ as f32).round() as u64 {
                    w.tick();
                }
                Scene::Stage(w)
            }
        };
        Game { scene, seed }
    }

    /// One 60 Hz tick.
    fn tick(&mut self, p: Pressed) {
        let next = match &mut self.scene {
            Scene::Title { t } => {
                *t += 1;
                p.start.then(|| Scene::Stage(World::new(self.seed)))
            }
            Scene::Stage(w) => {
                w.tick();
                p.back.then_some(Scene::Title { t: 0 })
            }
        };
        if let Some(s) = next {
            self.scene = s;
        }
    }

    fn draw(&self, r: &Renderer, lay: &render::layout::Layout) {
        match &self.scene {
            Scene::Title { t } => render::hud::title(r, lay, *t),
            Scene::Stage(w) => render::hud::stage(r, lay, w.frame),
        }
    }

    fn sim_frame(&self) -> u64 {
        match &self.scene {
            Scene::Title { t } => *t,
            Scene::Stage(w) => w.frame,
        }
    }

    fn sim_seed(&self) -> u64 {
        match &self.scene {
            Scene::Title { .. } => self.seed,
            Scene::Stage(w) => w.seed,
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

#[macroquad::main(window_conf)]
async fn main() {
    let hooks = debug::hooks();
    let seed = hooks.seed.unwrap_or_else(|| (miniquad::date::now() * 1000.0) as u64);
    let scripted = !hooks.shots.is_empty();
    let last_shot = hooks.shots.iter().copied().max().unwrap_or(0);

    let mut renderer = Renderer::new();
    let mut game = Game::new(seed, hooks.scene);
    let mut clock = clock::Clock::new();
    let mut perf = debug::Perf::new();
    let mut show_perf = hooks.perf;
    let mut fullscreen = false;
    let mut pressed = Pressed::default();
    let mut frame: u64 = 0;

    loop {
        let alt = is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt);
        if is_key_pressed(KeyCode::F11) || (alt && is_key_pressed(KeyCode::Enter)) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        } else {
            pressed.read();
        }
        if cfg!(debug_assertions) && is_key_pressed(KeyCode::F8) {
            show_perf = !show_perf;
        }

        // scripted screenshot runs advance exactly one tick per frame
        let dt = get_frame_time();
        let ticks = if scripted { 1 } else { clock.advance(dt as f64) };
        for _ in 0..ticks {
            game.tick(std::mem::take(&mut pressed));
        }
        perf.record(dt, ticks);

        let lay = Renderer::layout();
        renderer.begin(&lay);
        game.draw(&renderer, &lay);
        if show_perf {
            perf.draw(&renderer.atlas, &lay, game.sim_frame(), game.sim_seed());
        }
        renderer.end(&lay);

        frame += 1;
        if hooks.shots.contains(&frame) {
            debug::save_shot(frame, hooks.gray);
        }
        if scripted && frame >= last_shot {
            break;
        }
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(start: bool, back: bool) -> Pressed {
        Pressed { start, back }
    }

    #[test]
    fn start_leaves_the_title_and_back_returns() {
        let mut g = Game::new(5, None);
        g.tick(Pressed::default());
        assert!(matches!(g.scene, Scene::Title { t: 1 }));
        g.tick(press(true, false));
        assert!(matches!(&g.scene, Scene::Stage(w) if w.frame == 0 && w.seed == 5));
        g.tick(Pressed::default());
        assert_eq!(g.sim_frame(), 1);
        g.tick(press(false, true));
        assert!(matches!(g.scene, Scene::Title { t: 0 }));
    }

    #[test]
    fn pg_scene_stage_fast_forwards_in_frames() {
        let g = Game::new(1, Some(SceneReq::Stage(2.5)));
        assert_eq!(g.sim_frame(), 150);
    }
}
