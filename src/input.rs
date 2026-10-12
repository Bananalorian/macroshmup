//! Input from three sources at once: keyboard, controllers and touch. They're all
//! merged into the same controls; whichever you used last decides the on-screen hints.
//!
//! Keyboard: arrows or WASD move, Z fire, Shift focus, X bomb, C slam, Esc pause.
//! Controller: D-pad or left stick (8-way by default), A fire, RB/RT focus, B bomb, X slam,
//! Start pause.
//! Touch: drag anywhere to steer (relative, so your thumb never covers the ship),
//! it fires while you steer, a second finger focuses, and there are bomb and slam buttons.

use crate::pad::{self, Pads};
use crate::render::layout::{Layout, Orient};
use crate::tune::STICK_DEADZONE;
use macroquad::miniquad;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Keys,
    Pad,
    Touch,
}

/// Held this frame.
#[derive(Clone, Copy, Default, Debug)]
pub struct Held {
    pub mv: (f32, f32),
    pub fire: bool,
    pub focus: bool,
}

/// Menu and one-off presses this frame.
#[derive(Clone, Copy, Default, Debug)]
pub struct Ui {
    pub confirm: bool,
    pub back: bool,
    pub pause: bool,
    pub mute: bool,
    pub nav: (i32, i32),
    /// A tap or click, in canvas px.
    pub tap: Option<(f32, f32)>,
    /// Keyboard tuning hotkeys while flying: (which knob 0..3, -1 or 1).
    pub tune: Option<(usize, i32)>,
    /// Step the sandbox pattern: -1 or 1.
    pub pattern: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Button {
    Bomb,
    Slam,
    Pause,
    /// The sandbox's next-pattern button.
    Next,
}

/// The on-screen touch buttons, in canvas px: (which, center x, center y, radius).
pub fn touch_buttons(lay: &Layout) -> [(Button, f32, f32, f32); 4] {
    let f = lay.field;
    match lay.orient {
        Orient::Portrait => [
            (Button::Bomb, (lay.cw - 34) as f32, (f.bottom() + 46) as f32, 17.0),
            (Button::Slam, 34.0, (f.bottom() + 46) as f32, 17.0),
            (Button::Pause, (lay.cw - 12) as f32, 12.0, 10.0),
            (Button::Next, (lay.cw - 80) as f32, (f.bottom() + 52) as f32, 13.0),
        ],
        Orient::Landscape => [
            (Button::Bomb, ((f.right() + lay.cw) / 2) as f32, (lay.ch - 96) as f32, 20.0),
            (Button::Slam, ((f.right() + lay.cw) / 2) as f32, (lay.ch - 150) as f32, 18.0),
            (Button::Pause, ((f.right() + lay.cw) / 2) as f32, 70.0, 12.0),
            (Button::Next, ((f.right() + lay.cw) / 2) as f32, (lay.ch - 200) as f32, 15.0),
        ],
    }
}

fn button_at(lay: &Layout, x: f32, y: f32) -> Option<Button> {
    // a little generous: thumbs are big
    touch_buttons(lay).into_iter().find(|(_, cx, cy, r)| (x - cx).hypot(y - cy) <= r * 1.35).map(|(b, ..)| b)
}

/// A stick position after the deadzone: snapped to 8 directions at full
/// strength, or analog (rescaled so the edge of the deadzone is zero).
pub fn stick(x: f32, y: f32, analog: bool) -> (f32, f32) {
    let len = x.hypot(y);
    if len < STICK_DEADZONE || !len.is_finite() {
        return (0.0, 0.0);
    }
    if analog {
        let k = ((len - STICK_DEADZONE) / (1.0 - STICK_DEADZONE)).min(1.0) / len;
        return (x * k, y * k);
    }
    let step = std::f32::consts::FRAC_PI_4;
    let a = (y.atan2(x) / step).round() * step;
    let clean = |v: f32| if v.abs() < 1e-4 { 0.0 } else { v };
    (clean(a.cos()), clean(a.sin()))
}

pub struct Input {
    pub mode: Mode,
    pub pads: Pads,
    pub analog: bool,
    /// The steering finger: its id and last position (screen px).
    steer: Option<(u64, (f32, f32))>,
    focus_finger: Option<u64>,
    /// Fingers that started on a button (they don't steer).
    button_fingers: Vec<u64>,
    seen: Vec<u64>,
    /// Touch steering since the last tick, in canvas px (scaled by sensitivity).
    pub drag: (f32, f32),
    /// The bomb and the slam were pressed since the last tick.
    pub bomb: bool,
    pub slam: bool,
    /// Frames the bomb and slam buttons show pressed.
    pub bomb_flash: u32,
    pub slam_flash: u32,
    /// A controller was just plugged in.
    pub connected: bool,
}

impl Input {
    pub fn new() -> Input {
        simulate_mouse_with_touch(false);
        Input {
            mode: Mode::Keys,
            pads: Pads::new(),
            analog: false,
            steer: None,
            focus_finger: None,
            button_fingers: Vec::new(),
            seen: Vec::new(),
            drag: (0.0, 0.0),
            bomb: false,
            slam: false,
            bomb_flash: 0,
            slam_flash: 0,
            connected: false,
        }
    }

    /// Hand the next tick its share: the touch drag and any bomb or slam press, then
    /// clear them.
    pub fn take(&mut self) -> ((f32, f32), bool, bool) {
        let out = (self.drag, self.bomb, self.slam);
        self.drag = (0.0, 0.0);
        self.bomb = false;
        self.slam = false;
        out
    }

    /// Forget fingers (after a pause or a scene change).
    pub fn release(&mut self) {
        self.steer = None;
        self.focus_finger = None;
        self.button_fingers.clear();
        self.drag = (0.0, 0.0);
        self.bomb = false;
        self.slam = false;
    }

    /// Read every device once per display frame.
    pub fn poll(&mut self, lay: &Layout, sens: f32) -> (Held, Ui) {
        let mut h = Held::default();
        let mut ui = Ui::default();
        self.bomb_flash = self.bomb_flash.saturating_sub(1);
        self.slam_flash = self.slam_flash.saturating_sub(1);

        // ---- touch (positions arrive in screen px)
        for t in touches() {
            self.mode = Mode::Touch;
            let pos = (t.position.x, t.position.y);
            let c = lay.to_canvas(pos.0, pos.1);
            let started = matches!(t.phase, TouchPhase::Started);
            // a quick tap can start and end within one frame
            let quick = matches!(t.phase, TouchPhase::Ended) && !self.seen.contains(&t.id);
            if started {
                self.seen.push(t.id);
            }
            if started || quick {
                ui.tap = Some(c);
                match button_at(lay, c.0, c.1) {
                    Some(Button::Bomb) => {
                        self.bomb = true;
                        self.bomb_flash = 8;
                        self.button_fingers.push(t.id);
                    }
                    Some(Button::Slam) => {
                        self.slam = true;
                        self.slam_flash = 8;
                        self.button_fingers.push(t.id);
                    }
                    Some(Button::Pause) => {
                        ui.pause = true;
                        self.button_fingers.push(t.id);
                    }
                    Some(Button::Next) => {
                        ui.pattern += 1;
                        self.button_fingers.push(t.id);
                    }
                    None if started => {
                        if self.steer.is_none() {
                            self.steer = Some((t.id, pos));
                        } else if self.focus_finger.is_none() {
                            self.focus_finger = Some(t.id);
                        }
                    }
                    None => {}
                }
            }
            if matches!(t.phase, TouchPhase::Moved | TouchPhase::Stationary) {
                if let Some((id, last)) = self.steer {
                    if id == t.id {
                        let k = sens / lay.scale as f32;
                        self.drag.0 += (pos.0 - last.0) * k;
                        self.drag.1 += (pos.1 - last.1) * k;
                        self.steer = Some((id, pos));
                    }
                }
            }
            if matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                self.seen.retain(|&id| id != t.id);
                self.button_fingers.retain(|&id| id != t.id);
                if self.steer.is_some_and(|(id, _)| id == t.id) {
                    // the second finger takes over steering, so you can swap thumbs
                    self.steer = self.focus_finger.take().map(|id| (id, pos_of(id).unwrap_or(pos)));
                }
                if self.focus_finger == Some(t.id) {
                    self.focus_finger = None;
                }
            }
        }
        h.fire |= self.steer.is_some(); // touch fires while you steer
        h.focus |= self.focus_finger.is_some();

        // ---- controller
        self.pads.poll();
        self.connected = self.pads.just_connected();
        if self.pads.active() {
            self.mode = Mode::Pad;
        }
        if self.pads.cur.connected {
            let p = &self.pads;
            let s = p.left_raw();
            let (mut x, mut y) = stick(s.x, s.y, self.analog);
            if p.down(pad::LEFT) {
                x = -1.0;
            }
            if p.down(pad::RIGHT) {
                x = 1.0;
            }
            if p.down(pad::UP) {
                y = -1.0;
            }
            if p.down(pad::DOWN) {
                y = 1.0;
            }
            h.mv = (x, y);
            h.fire |= p.down(pad::A);
            h.focus |= p.down(pad::RB) || p.trigger(pad::RT) > 0.3 || p.trigger(pad::LT) > 0.3;
            if p.pressed(pad::B) {
                self.bomb = true;
            }
            if p.pressed(pad::X) {
                self.slam = true;
            }
            ui.confirm |= p.pressed(pad::A);
            ui.back |= p.pressed(pad::B);
            ui.pause |= p.pressed(pad::START) || p.pressed(pad::BACK);
            ui.pattern += p.pressed(pad::LB) as i32;
            let n = p.nav();
            ui.nav.0 += n.x;
            ui.nav.1 += n.y;
        }

        // ---- keyboard and mouse
        let key = |k: &[KeyCode]| k.iter().any(|&k| is_key_down(k));
        let hit = |k: &[KeyCode]| k.iter().any(|&k| is_key_pressed(k));
        let mut mv = (0.0, 0.0);
        if key(&[KeyCode::Left, KeyCode::A]) {
            mv.0 -= 1.0;
        }
        if key(&[KeyCode::Right, KeyCode::D]) {
            mv.0 += 1.0;
        }
        if key(&[KeyCode::Up, KeyCode::W]) {
            mv.1 -= 1.0;
        }
        if key(&[KeyCode::Down, KeyCode::S]) {
            mv.1 += 1.0;
        }
        let fire = key(&[KeyCode::Z, KeyCode::J]);
        let focus = key(&[KeyCode::LeftShift, KeyCode::RightShift]);
        if mv != (0.0, 0.0) || fire || focus || get_last_key_pressed().is_some() {
            self.mode = Mode::Keys;
        }
        if mv != (0.0, 0.0) {
            h.mv = mv; // the sim normalizes diagonals
        }
        h.fire |= fire;
        h.focus |= focus;
        if hit(&[KeyCode::X, KeyCode::K]) {
            self.bomb = true;
        }
        if hit(&[KeyCode::C, KeyCode::L]) {
            self.slam = true;
        }
        ui.confirm |= hit(&[KeyCode::Enter, KeyCode::KpEnter, KeyCode::Z, KeyCode::Space]);
        ui.back |= hit(&[KeyCode::Escape, KeyCode::Backspace, KeyCode::X]);
        ui.pause |= hit(&[KeyCode::Escape, KeyCode::P]);
        ui.mute |= hit(&[KeyCode::M]);
        ui.pattern += hit(&[KeyCode::RightBracket]) as i32 - hit(&[KeyCode::LeftBracket]) as i32;
        if hit(&[KeyCode::Left, KeyCode::A]) {
            ui.nav.0 -= 1;
        }
        if hit(&[KeyCode::Right, KeyCode::D]) {
            ui.nav.0 += 1;
        }
        if hit(&[KeyCode::Up, KeyCode::W]) {
            ui.nav.1 -= 1;
        }
        if hit(&[KeyCode::Down, KeyCode::S]) {
            ui.nav.1 += 1;
        }
        for (i, pair) in [[KeyCode::Key1, KeyCode::Key2], [KeyCode::Key3, KeyCode::Key4], [KeyCode::Key5, KeyCode::Key6]].iter().enumerate() {
            if is_key_pressed(pair[0]) {
                ui.tune = Some((i, -1));
            }
            if is_key_pressed(pair[1]) {
                ui.tune = Some((i, 1));
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            // mouse positions are in logical px; the layout works in physical ones
            let (mx, my) = mouse_position();
            let d = miniquad::window::dpi_scale();
            ui.tap = Some(lay.to_canvas(mx * d, my * d));
        }
        (h, ui)
    }
}

fn pos_of(id: u64) -> Option<(f32, f32)> {
    touches().into_iter().find(|t| t.id == id).map(|t| (t.position.x, t.position.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stick_snaps_to_eight_directions_at_full_strength() {
        assert_eq!(stick(0.1, 0.1, false), (0.0, 0.0), "inside the deadzone");
        assert_eq!(stick(0.9, 0.1, false), (1.0, 0.0));
        assert_eq!(stick(0.0, -0.4, false), (0.0, -1.0), "a light push is still full speed");
        let (x, y) = stick(0.6, 0.5, false);
        assert!((x - y).abs() < 1e-5 && (x.hypot(y) - 1.0).abs() < 1e-5, "a clean diagonal");
        assert_eq!(stick(-0.2, -0.8, false), (0.0, -1.0), "14 degrees off up is up");
        let (x, y) = stick(-0.3, -0.7, false);
        assert!(x < -0.7 && y < -0.7, "23 degrees off up is up-left");
    }

    #[test]
    fn analog_mode_scales_from_the_deadzone_edge() {
        let (x, _) = stick(STICK_DEADZONE + 0.0001, 0.0, true);
        assert!(x < 0.01);
        let (x, y) = stick(1.0, 0.0, true);
        assert_eq!((x, y), (1.0, 0.0));
        let (x, _) = stick(0.5 + STICK_DEADZONE * 0.5, 0.0, true);
        assert!((x - 0.5).abs() < 1e-4);
    }

    #[test]
    fn touch_buttons_sit_outside_the_playfield() {
        for (w, h) in [(1280, 720), (390, 844), (1170, 2532), (844, 390)] {
            let lay = Layout::fit(w, h);
            for (b, cx, cy, r) in touch_buttons(&lay) {
                let f = lay.field;
                let inside = cx + r > f.x as f32 && cx - r < f.right() as f32 && cy + r > f.y as f32 && cy - r < f.bottom() as f32;
                assert!(!inside, "{b:?} covers the playfield at {w}x{h}");
                assert!(cx - r >= 0.0 && cx + r <= lay.cw as f32 && cy - r >= 0.0 && cy + r <= lay.ch as f32, "{b:?} off the canvas");
                assert_eq!(button_at(&lay, cx, cy), Some(b));
            }
            assert_eq!(button_at(&lay, (lay.field.x + lay.field.w / 2) as f32, (lay.field.y + 180) as f32), None);
        }
    }
}
