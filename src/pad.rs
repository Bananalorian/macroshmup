//! Gamepads, ported from Last Train to Shinjuku. On Windows the game reads Xbox-style
//! controllers through XInput, the moment each frame needs them, like most PC games
//! (gilrs's Windows.Gaming.Input path lagged a Bluetooth controller that every other
//! game read fine). Linux and macOS read controllers through gilrs; the web build
//! reads the browser Gamepad API through a tiny JS plugin in web/template.html. All
//! are normalized to the W3C "standard" layout (Xbox naming below). The game never sends a controller anything: no rumble (it
//! lagged a Bluetooth controller's input; it may come back later as its own feature).

use macroquad::prelude::*;

pub const A: usize = 0;
pub const B: usize = 1;
pub const X: usize = 2;
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

/// Windows: XInput, read on the spot. Connected controllers are read every frame (a
/// cached state, about 10 microseconds); empty slots are checked once a second,
/// since asking an empty slot is slow.
#[cfg(windows)]
mod native {
    use super::{PadState, NB};

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Gamepad {
        buttons: u16,
        lt: u8,
        rt: u8,
        lx: i16,
        ly: i16,
        rx: i16,
        ry: i16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct State {
        packet: u32,
        pad: Gamepad,
    }

    #[link(name = "xinput1_4", kind = "raw-dylib")]
    unsafe extern "system" {
        fn XInputGetState(user: u32, state: *mut State) -> u32;
    }

    /// XInput's button bits, in the standard layout's order (the guide button isn't in XInput).
    const MAP: [(u16, usize); 14] = [
        (0x1000, 0),  // A
        (0x2000, 1),  // B
        (0x4000, 2),  // X
        (0x8000, 3),  // Y
        (0x0100, 4),  // LB
        (0x0200, 5),  // RB
        (0x0020, 8),  // back
        (0x0010, 9),  // start
        (0x0040, 10), // left stick
        (0x0080, 11), // right stick
        (0x0001, 12), // d-pad up
        (0x0002, 13), // down
        (0x0004, 14), // left
        (0x0008, 15), // right
    ];

    pub struct Native {
        connected: [bool; 4],
        active: usize,
        frame: u64,
    }

    fn axis(v: i16) -> f32 {
        (v as f32 / 32767.0).clamp(-1.0, 1.0)
    }

    /// One controller's state in the standard layout (y pointing down).
    fn standard(p: &Gamepad) -> PadState {
        let mut s = PadState { connected: true, ..Default::default() };
        s.axes = [axis(p.lx), -axis(p.ly), axis(p.rx), -axis(p.ry)];
        for (bit, i) in MAP {
            s.buttons[i] = if p.buttons & bit != 0 { 1.0 } else { 0.0 };
        }
        s.buttons[6] = p.lt as f32 / 255.0;
        s.buttons[7] = p.rt as f32 / 255.0;
        debug_assert_eq!(s.buttons.len(), NB);
        s
    }

    fn busy(s: &PadState) -> bool {
        s.buttons.iter().any(|b| *b > 0.5) || s.axes[0].hypot(s.axes[1]) > 0.3
    }

    impl Native {
        pub fn new() -> Self {
            Native { connected: [false; 4], active: 0, frame: 0 }
        }

        pub fn read(&mut self) -> PadState {
            let recheck = self.frame % 60 == 0;
            self.frame += 1;
            let mut states: [Option<PadState>; 4] = [None; 4];
            for slot in 0..4 {
                if !self.connected[slot] && !recheck {
                    continue;
                }
                let mut st = State::default();
                // SAFETY: a plain call into xinput1_4.dll with a valid out-pointer
                let ok = unsafe { XInputGetState(slot as u32, &mut st) } == 0;
                self.connected[slot] = ok;
                if ok {
                    states[slot] = Some(standard(&st.pad));
                }
            }
            // the controller in use: whichever is being touched, else the last one used
            if let Some(i) = (0..4).find(|&i| states[i].as_ref().is_some_and(busy)) {
                self.active = i;
            } else if states[self.active].is_none() {
                if let Some(i) = (0..4).find(|&i| states[i].is_some()) {
                    self.active = i;
                }
            }
            states[self.active].unwrap_or_default()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn xinput_maps_to_the_standard_layout() {
            let p = Gamepad { buttons: 0x1000 | 0x0200 | 0x0001, lt: 255, rt: 0, lx: 32767, ly: 32767, rx: 0, ry: -32768 };
            let s = standard(&p);
            assert!(s.connected);
            assert_eq!((s.buttons[0], s.buttons[5], s.buttons[12], s.buttons[1]), (1.0, 1.0, 1.0, 0.0), "A, RB, d-pad up");
            assert_eq!((s.buttons[6], s.buttons[7]), (1.0, 0.0), "the triggers are analog");
            assert_eq!(s.axes[0], 1.0);
            assert_eq!(s.axes[1], -1.0, "stick up is up the screen (y points down)");
            assert_eq!(s.axes[3], 1.0);
        }

        #[test]
        fn reading_with_or_without_a_controller_never_fails() {
            let mut n = Native::new();
            for _ in 0..3 {
                let s = n.read();
                assert!(s.axes.iter().all(|a| a.is_finite() && a.abs() <= 1.0));
            }
        }
    }
}

/// Linux and macOS: gilrs.
#[cfg(not(any(windows, target_arch = "wasm32")))]
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
    /// Rumble is out for now: nothing may send a controller a motor command (no
    /// XInputSetState on Windows), and gilrs runs without its force-feedback thread.
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
            for banned in ["vibrationActuator", "XInputSetState", "SetVibration", "gilrs::ff", "navigator.vibrate", "ff::Effect"] {
                assert!(!code.contains(banned), "{} uses {banned}", f.display());
            }
        }
        let pad = std::fs::read_to_string(root.join("src/pad.rs")).unwrap();
        assert!(pad.contains("with_force_feedback(false)"));
    }
}
