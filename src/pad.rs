//! Gamepads, ported from Last Train to Shinjuku. Desktop builds read controllers
//! through gilrs; the web build reads the browser Gamepad API through a tiny JS
//! plugin in web/template.html. Both are normalized to the W3C "standard" layout
//! (Xbox naming below). The game never sends a controller anything: no rumble (it
//! lagged a Bluetooth controller's input; it may come back later as its own feature).

use macroquad::prelude::*;

pub const A: usize = 0;
pub const B: usize = 1;
pub const LB: usize = 4;
pub const RB: usize = 5;
pub const LT: usize = 6;
pub const RT: usize = 7;
pub const BACK: usize = 8;
pub const START: usize = 9;
pub const UP: usize = 12;
pub const DOWN: usize = 13;
pub const LEFT: usize = 14;
pub const RIGHT: usize = 15;
const NB: usize = 17;

#[derive(Default, Clone, Copy)]
pub struct PadState {
    pub connected: bool,
    pub axes: [f32; 4], // left x, left y, right x, right y (y points down)
    pub buttons: [f32; NB],
}

pub struct Pads {
    pub cur: PadState,
    prev: PadState,
    #[cfg(not(target_arch = "wasm32"))]
    native: native::Native,
}

fn deadzone(v: Vec2, dz: f32) -> Vec2 {
    let l = v.length();
    if l < dz { Vec2::ZERO } else { v / l * ((l - dz) / (1.0 - dz)).min(1.0) }
}

impl Pads {
    pub fn new() -> Self {
        Pads {
            cur: PadState::default(),
            prev: PadState::default(),
            #[cfg(not(target_arch = "wasm32"))]
            native: native::Native::new(),
        }
    }

    pub fn poll(&mut self) {
        self.prev = self.cur;
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.cur = self.native.read();
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.cur = web::read();
        }
    }

    pub fn just_connected(&self) -> bool {
        self.cur.connected && !self.prev.connected
    }
    pub fn pressed(&self, b: usize) -> bool {
        self.cur.buttons[b] > 0.5 && self.prev.buttons[b] <= 0.5
    }
    pub fn trigger(&self, b: usize) -> f32 {
        self.cur.buttons[b]
    }
    /// The left stick, raw (the deadzone and 8-way snap happen in input.rs).
    pub fn left_raw(&self) -> Vec2 {
        vec2(self.cur.axes[0], self.cur.axes[1])
    }
    pub fn left(&self) -> Vec2 {
        deadzone(vec2(self.cur.axes[0], self.cur.axes[1]), 0.2)
    }
    pub fn down(&self, b: usize) -> bool {
        self.cur.buttons[b] > 0.5
    }
    fn prev_left(&self) -> Vec2 {
        deadzone(vec2(self.prev.axes[0], self.prev.axes[1]), 0.2)
    }

    /// Menu navigation from the d-pad or a flick of the left stick: (-1, 0, 1) per axis.
    pub fn nav(&self) -> IVec2 {
        let mut n = IVec2::ZERO;
        if self.pressed(LEFT) { n.x -= 1; }
        if self.pressed(RIGHT) { n.x += 1; }
        if self.pressed(UP) { n.y -= 1; }
        if self.pressed(DOWN) { n.y += 1; }
        let (l, p) = (self.left(), self.prev_left());
        if l.x < -0.6 && p.x >= -0.6 { n.x -= 1; }
        if l.x > 0.6 && p.x <= 0.6 { n.x += 1; }
        if l.y < -0.6 && p.y >= -0.6 { n.y -= 1; }
        if l.y > 0.6 && p.y <= 0.6 { n.y += 1; }
        n
    }

    pub fn active(&self) -> bool {
        self.cur.connected && (self.left().length() > 0.3 || self.cur.buttons.iter().any(|b| *b > 0.5))
    }

}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::{PadState, NB};

    unsafe extern "C" {
        fn pg_pad_poll(ptr: *mut f32, len: u32) -> u32;
    }

    /// Lets the JS loader confirm the plugin matches this build.
    #[unsafe(no_mangle)]
    pub extern "C" fn pg_gamepad_crate_version() -> u32 {
        1
    }

    pub fn read() -> PadState {
        let mut buf = [0.0f32; 24];
        let ok = unsafe { pg_pad_poll(buf.as_mut_ptr(), buf.len() as u32) };
        let mut s = PadState::default();
        if ok == 1 {
            s.connected = true;
            s.axes.copy_from_slice(&buf[2..6]);
            s.buttons.copy_from_slice(&buf[6..6 + NB]);
        }
        s
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::{PadState, NB};
    use gilrs::{Axis, Button, GamepadId, Gilrs};

    const MAP: [Button; NB] = [
        Button::South,
        Button::East,
        Button::West,
        Button::North,
        Button::LeftTrigger,
        Button::RightTrigger,
        Button::LeftTrigger2,
        Button::RightTrigger2,
        Button::Select,
        Button::Start,
        Button::LeftThumb,
        Button::RightThumb,
        Button::DPadUp,
        Button::DPadDown,
        Button::DPadLeft,
        Button::DPadRight,
        Button::Mode,
    ];

    pub struct Native {
        gilrs: Option<Gilrs>,
        active: Option<GamepadId>,
    }

    impl Native {
        pub fn new() -> Self {
            // no force feedback: gilrs's would re-send the motor level to every
            // controller 20 times a second, and over Bluetooth that traffic lags input
            let gilrs = gilrs::GilrsBuilder::new().with_force_feedback(false).build().ok();
            Native { gilrs, active: None }
        }

        pub fn read(&mut self) -> PadState {
            let mut s = PadState::default();
            let Some(g) = &mut self.gilrs else { return s };
            while let Some(ev) = g.next_event() {
                self.active = Some(ev.id);
            }
            if self.active.map_or(true, |id| !g.gamepad(id).is_connected()) {
                self.active = g.gamepads().next().map(|(id, _)| id);
            }
            let Some(id) = self.active else { return s };
            let gp = g.gamepad(id);
            s.connected = true;
            s.axes = [gp.value(Axis::LeftStickX), -gp.value(Axis::LeftStickY), gp.value(Axis::RightStickX), -gp.value(Axis::RightStickY)];
            for (i, b) in MAP.iter().enumerate() {
                let analog = gp.button_data(*b).map(|d| d.value()).unwrap_or(0.0);
                s.buttons[i] = if gp.is_pressed(*b) { analog.max(1.0) } else { analog.min(0.49) };
            }
            s
        }

    }
}

#[cfg(test)]
mod tests {
    /// Rumble is out for now: nothing may send a controller a motor command, and
    /// gilrs runs without its force-feedback thread.
    #[test]
    fn the_game_never_talks_to_a_controllers_motors() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = vec![root.join("web/template.html")];
        let mut stack = vec![root.join("src")];
        while let Some(dir) = stack.pop() {
            for e in std::fs::read_dir(dir).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    files.push(p);
                }
            }
        }
        for f in &files {
            let src = std::fs::read_to_string(f).unwrap();
            // the game code only (this test names what it bans)
            let code = src.split("#[cfg(test)]").next().unwrap();
            for banned in ["vibrationActuator", "XInputSetState", "SetVibration", "gilrs::ff", "navigator.vibrate"] {
                assert!(!code.contains(banned), "{} uses {banned}", f.display());
            }
        }
        let pad = std::fs::read_to_string(root.join("src/pad.rs")).unwrap();
        assert!(pad.contains("with_force_feedback(false)"));
    }
}
