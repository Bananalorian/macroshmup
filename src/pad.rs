//! Gamepads, ported from Last Train to Shinjuku. Desktop builds read controllers
//! through gilrs; the web build reads the browser Gamepad API through a tiny JS
//! plugin in web/template.html. Both are normalized to the W3C "standard" layout
//! (Xbox naming below). Rumble works on both where the controller supports it.

use crate::tune::{RUMBLE_REFRESH, RUMBLE_TAIL};
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
    gate: Gate,
    /// Rumble commands sent to the controller this run (PG_PERF reports it).
    pub commands: u32,
    #[cfg(not(target_arch = "wasm32"))]
    native: native::Native,
}

/// What the motors should be told, if anything: (strong, weak, seconds).
pub type Command = (f32, f32, f64);

/// Decides when the controller actually hears about a rumble. Requests come every
/// few frames (the fire hum), but the motors only get a command when the strength
/// changes, when the last command is about to run out, or when it's time to stop.
#[derive(Default, Debug)]
pub struct Gate {
    /// The rumble the game wants: its strength (the stronger motor) and when it ends.
    until: f64,
    level: f32,
    /// What the motors were last told, and when that command runs out on its own.
    told: (f32, f32),
    told_until: f64,
}

impl Gate {
    /// A rumble request at time `now` (s). A weaker request never cuts off a stronger
    /// one that's still playing.
    pub fn request(&mut self, now: f64, strong: f32, weak: f32, ms: u32) -> Option<Command> {
        let level = strong.max(weak);
        if now < self.until && level < self.level {
            return None;
        }
        self.until = now + ms as f64 / 1000.0;
        self.level = level;
        if (strong, weak) == self.told && self.told_until - now > RUMBLE_REFRESH {
            return None; // already playing: just runs on
        }
        let secs = ms as f64 / 1000.0 + RUMBLE_TAIL;
        self.told = (strong, weak);
        self.told_until = now + secs;
        Some((strong, weak, secs))
    }

    /// Once a frame: the motors go off when the rumble's time is up.
    pub fn update(&mut self, now: f64) -> Option<Command> {
        if self.told != (0.0, 0.0) && now >= self.until {
            self.stop()
        } else {
            None
        }
    }

    /// Off now.
    pub fn stop(&mut self) -> Option<Command> {
        *self = Gate::default();
        Some((0.0, 0.0, 0.0))
    }
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
            gate: Gate::default(),
            commands: 0,
            #[cfg(not(target_arch = "wasm32"))]
            native: native::Native::new(),
        }
    }

    pub fn poll(&mut self) {
        if let Some(c) = self.gate.update(get_time()) {
            self.send(c);
        }
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

    /// Rumble, if the controller and platform support it (see `Gate` for when the
    /// controller actually gets told).
    pub fn rumble(&mut self, strong: f32, weak: f32, ms: u32) {
        if let Some(c) = self.gate.request(get_time(), strong, weak, ms) {
            self.send(c);
        }
    }

    fn send(&mut self, (strong, weak, secs): Command) {
        self.commands += 1;
        let ms = (secs * 1000.0).round() as u32;
        #[cfg(not(target_arch = "wasm32"))]
        self.native.rumble(strong, weak, ms);
        #[cfg(target_arch = "wasm32")]
        web::rumble(strong, weak, ms.max(1));
    }

    /// Stop the motors, and wait (briefly) until the controller has been told. Call
    /// this before the game exits: a controller holds the last strength it was given,
    /// so quitting mid-rumble would leave it buzzing.
    pub fn stop(&mut self) {
        self.gate.stop();
        #[cfg(not(target_arch = "wasm32"))]
        self.native.stop();
        #[cfg(target_arch = "wasm32")]
        web::rumble(0.0, 0.0, 1);
    }
}

impl Pads {
    /// A buzz for phones (touch play): the browser's vibrate API, where it exists.
    pub fn vibrate(&self, ms: u32) {
        #[cfg(target_arch = "wasm32")]
        web::vibrate(ms);
        #[cfg(not(target_arch = "wasm32"))]
        let _ = ms;
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::{PadState, NB};

    unsafe extern "C" {
        fn pg_pad_poll(ptr: *mut f32, len: u32) -> u32;
        fn pg_pad_rumble(strong: f32, weak: f32, ms: f32);
        fn pg_vibrate(ms: f32);
    }

    pub fn vibrate(ms: u32) {
        unsafe { pg_vibrate(ms as f32) };
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

    pub fn rumble(strong: f32, weak: f32, ms: u32) {
        unsafe { pg_pad_rumble(strong, weak, ms as f32) };
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
        #[cfg(not(windows))]
        effect: Option<gilrs::ff::Effect>,
        #[cfg(windows)]
        motor: motor::Motor,
    }

    impl Native {
        pub fn new() -> Self {
            // On Windows gilrs's force feedback re-sends the motor level to every
            // controller 20 times a second, even at rest, and over Bluetooth that radio
            // traffic delays the controller's input. The game drives the motors itself
            // there (`motor`), only when the strength changes.
            let gilrs = gilrs::GilrsBuilder::new().with_force_feedback(!cfg!(windows)).build().ok();
            Native {
                gilrs,
                active: None,
                #[cfg(not(windows))]
                effect: None,
                #[cfg(windows)]
                motor: motor::Motor::new(),
            }
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

        #[cfg(windows)]
        pub fn rumble(&mut self, strong: f32, weak: f32, _ms: u32) {
            // the motors hold the level until the next command (the Gate sends the stop)
            self.motor.set(strong, weak);
        }

        #[cfg(windows)]
        pub fn stop(&mut self) {
            self.motor.stop_and_wait(std::time::Duration::from_millis(300));
        }

        #[cfg(not(windows))]
        pub fn rumble(&mut self, strong: f32, weak: f32, ms: u32) {
            use gilrs::ff::{BaseEffect, BaseEffectType, EffectBuilder, Replay, Ticks};
            if strong <= 0.0 && weak <= 0.0 {
                self.stop_effect();
                return;
            }
            let (Some(g), Some(id)) = (&mut self.gilrs, self.active) else { return };
            if !g.gamepad(id).is_ff_supported() {
                return;
            }
            let rep = Replay { play_for: Ticks::from_ms(ms), ..Default::default() };
            let mag = |k: f32| (k.clamp(0.0, 1.0) * 65535.0) as u16;
            let eff = EffectBuilder::new()
                .add_effect(BaseEffect { kind: BaseEffectType::Strong { magnitude: mag(strong) }, scheduling: rep, ..Default::default() })
                .add_effect(BaseEffect { kind: BaseEffectType::Weak { magnitude: mag(weak) }, scheduling: rep, ..Default::default() })
                .gamepads(&[id])
                .finish(g);
            if let Ok(e) = eff {
                let _ = e.play();
                self.effect = Some(e); // keep it alive until the next one replaces it
            }
        }

        #[cfg(not(windows))]
        fn stop_effect(&mut self) {
            if let Some(e) = self.effect.take() {
                let _ = e.stop();
            }
        }

        /// Stop the effect, and give gilrs a couple of its 50 ms ticks to zero the motors.
        #[cfg(not(windows))]
        pub fn stop(&mut self) {
            self.stop_effect();
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
    }

    /// The controller's motors on Windows, through XInput, from a thread of their own:
    /// a write to a Bluetooth controller can take tens of ms and must never hold up a
    /// frame. Only the latest level matters, so a backlog collapses into one write.
    #[cfg(windows)]
    mod motor {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::mpsc::{Sender, channel};
        use std::time::{Duration, Instant};

        #[repr(C)]
        struct Vibration {
            left: u16,
            right: u16,
        }

        #[link(name = "xinput1_4", kind = "raw-dylib")]
        unsafe extern "system" {
            fn XInputSetState(user: u32, vibration: *mut Vibration) -> u32;
        }

        pub struct Motor {
            tx: Sender<(u16, u16, u32)>,
            /// The newest command sent, and the newest one written.
            sent: u32,
            done: Arc<AtomicU32>,
        }

        impl Motor {
            pub fn new() -> Motor {
                let (tx, rx) = channel::<(u16, u16, u32)>();
                let done = Arc::new(AtomicU32::new(0));
                let written = done.clone();
                std::thread::spawn(move || {
                    while let Ok(mut c) = rx.recv() {
                        while let Ok(newer) = rx.try_recv() {
                            c = newer;
                        }
                        for slot in 0..4 {
                            let mut v = Vibration { left: c.0, right: c.1 };
                            // SAFETY: a plain call into xinput1_4.dll with a valid struct
                            unsafe { XInputSetState(slot, &mut v) };
                        }
                        written.store(c.2, Ordering::Release);
                    }
                });
                Motor { tx, sent: 0, done }
            }

            pub fn set(&mut self, strong: f32, weak: f32) {
                let m = |k: f32| (k.clamp(0.0, 1.0) * 65535.0) as u16;
                self.sent += 1;
                let _ = self.tx.send((m(strong), m(weak), self.sent));
            }

            /// Motors off, waiting up to `limit` for the write to happen.
            pub fn stop_and_wait(&mut self, limit: Duration) {
                self.set(0.0, 0.0);
                let t0 = Instant::now();
                while self.done.load(Ordering::Acquire) < self.sent && t0.elapsed() < limit {
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fire_hum_tells_the_controller_once_not_every_few_frames() {
        let mut g = Gate::default();
        let mut sent = 0;
        // the held hum: requested every 4 frames for 3 seconds
        for f in (0..180).step_by(4) {
            if g.request(f as f64 / 60.0, 0.0, 0.07, 90).is_some() {
                sent += 1;
            }
            assert_eq!(g.update(f as f64 / 60.0), None, "never cut off while it's held");
        }
        assert!(sent <= 3 + 180 / 60 * 3, "{sent} commands in 3 s");
        // let go: the motors are switched off once its time is up
        assert_eq!(g.update(3.0), None);
        assert_eq!(g.update(3.2), Some((0.0, 0.0, 0.0)));
        assert_eq!(g.update(3.3), None, "only once");
    }

    #[test]
    fn changes_go_through_and_weaker_never_cuts_off_stronger() {
        let mut g = Gate::default();
        assert!(g.request(0.0, 0.0, 0.07, 90).is_some());
        let kick = g.request(0.01, 1.0, 0.8, 450).expect("a stronger kick goes straight through");
        assert!(kick.2 >= 0.45);
        assert_eq!(g.request(0.05, 0.0, 0.07, 90), None, "the hum waits for the kick");
        assert_eq!(g.update(0.3), None);
        assert_eq!(g.update(0.47), Some((0.0, 0.0, 0.0)), "off when the kick is over");
        assert!(g.request(0.5, 0.0, 0.07, 90).is_some(), "then the hum again");
        assert_eq!(g.stop(), Some((0.0, 0.0, 0.0)));
        assert_eq!(g.update(10.0), None, "nothing left to stop");
    }
}
