//! The pause menu, which is also the pattern sandbox's controls and the live
//! tuning panel: speed, focus ratio and blend frames change the game the moment
//! you resume (and are saved). The sandbox switches last for the session. And the
//! title's options: volumes, shake, flashing, the hitbox, the fire scheme, touch.

use crate::render::post::ShakeSetting;
use crate::storage::Settings;
use crate::tune::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Resume,
    Pattern,
    Autopilot,
    SlowMo,
    Invincible,
    Hitboxes,
    Gray,
    Fps,
    Boom,
    ReduceFlash,
    Shake,
    Speed,
    Focus,
    Blend,
    Touch,
    Stick,
    Reset,
    Quit,
    Music,
    Sound,
    ShowHitbox,
    Fire,
    Back,
}

pub const ITEMS: [Item; 18] = [
    Item::Resume,
    Item::Pattern,
    Item::Autopilot,
    Item::SlowMo,
    Item::Invincible,
    Item::Hitboxes,
    Item::Gray,
    Item::Fps,
    Item::Boom,
    Item::ReduceFlash,
    Item::Shake,
    Item::Speed,
    Item::Focus,
    Item::Blend,
    Item::Touch,
    Item::Stick,
    Item::Reset,
    Item::Quit,
];

/// The title's options screen.
pub const OPTIONS: [Item; 9] = [Item::Music, Item::Sound, Item::Shake, Item::ReduceFlash, Item::ShowHitbox, Item::Fire, Item::Touch, Item::Stick, Item::Back];

impl Item {
    pub fn label(self) -> &'static str {
        match self {
            Item::Resume => "RESUME",
            Item::Pattern => "PATTERN",
            Item::Autopilot => "AUTOPILOT",
            Item::SlowMo => "SLOW-MO",
            Item::Invincible => "INVINCIBLE",
            Item::Hitboxes => "HITBOXES",
            Item::Gray => "GRAYSCALE",
            Item::Fps => "SHOW FPS",
            Item::Boom => "BOOM TEST",
            Item::ReduceFlash => "REDUCE FLASH",
            Item::Shake => "SHAKE",
            Item::Speed => "SPEED",
            Item::Focus => "FOCUS",
            Item::Blend => "BLEND",
            Item::Touch => "TOUCH",
            Item::Stick => "STICK",
            Item::Reset => "RESET TUNING",
            Item::Quit => "QUIT TO TITLE",
            Item::Music => "MUSIC",
            Item::Sound => "SOUND",
            Item::ShowHitbox => "HITBOX",
            Item::Fire => "FIRE",
            Item::Back => "BACK",
        }
    }

    /// Items with a value you change with left and right.
    pub fn adjustable(self) -> bool {
        !matches!(self, Item::Resume | Item::Reset | Item::Quit | Item::Back)
    }
}

fn on_off(b: bool) -> String {
    (if b { "ON" } else { "OFF" }).to_string()
}

/// The value as shown, e.g. `4.00 PX`.
pub fn value(s: &Settings, item: Item) -> String {
    let v = &s.view;
    match item {
        Item::Pattern => SANDBOX[v.pattern % SANDBOX.len()].name.to_string(),
        Item::Autopilot => on_off(v.autopilot),
        Item::SlowMo => on_off(v.slowmo),
        Item::Invincible => on_off(v.invincible),
        Item::Hitboxes => on_off(v.hitboxes),
        Item::Gray => on_off(v.gray),
        Item::Fps => on_off(v.fps),
        Item::Boom => crate::render::explode::Size::ALL[v.boom % 4].label().to_string(),
        Item::ReduceFlash => on_off(s.reduce_flash),
        Item::Shake => match s.shake {
            ShakeSetting::Off => "OFF",
            ShakeSetting::Low => "LOW",
            ShakeSetting::Full => "FULL",
        }
        .to_string(),
        Item::Speed => format!("{:.2} PX", s.tuning.speed),
        Item::Focus => format!("{:.2}X", s.tuning.focus_ratio),
        Item::Blend => format!("{} F", s.tuning.focus_blend),
        Item::Touch => format!("{:.1}X", s.touch_sens),
        Item::Stick => (if s.analog { "ANALOG" } else { "8-WAY" }).to_string(),
        Item::Music => volume(s.music_vol),
        Item::Sound => volume(s.sound_vol),
        Item::ShowHitbox => (if s.hitbox_always { "ALWAYS" } else { "FOCUSED" }).to_string(),
        Item::Fire => (if s.hold_beam { "HOLD TO BEAM" } else { "FOCUS BUTTON" }).to_string(),
        _ => String::new(),
    }
}

/// A volume, 0-10, as a percentage.
fn volume(v: u8) -> String {
    if v == 0 { "OFF".to_string() } else { format!("{}%", v as u32 * 10) }
}

fn step(v: f32, (lo, hi, by): (f32, f32, f32), dir: i32) -> f32 {
    let n = ((v - lo) / by).round() + dir as f32;
    (lo + n * by).clamp(lo, hi)
}

/// Change an item's value one step left (-1) or right (1).
pub fn adjust(s: &mut Settings, item: Item, dir: i32) {
    let n = SANDBOX.len() as i32;
    let v = &mut s.view;
    match item {
        Item::Pattern => v.pattern = (v.pattern as i32 + dir).rem_euclid(n) as usize,
        Item::Autopilot => v.autopilot = !v.autopilot,
        Item::SlowMo => v.slowmo = !v.slowmo,
        Item::Invincible => v.invincible = !v.invincible,
        Item::Hitboxes => v.hitboxes = !v.hitboxes,
        Item::Gray => v.gray = !v.gray,
        Item::Fps => v.fps = !v.fps,
        Item::Boom => v.boom = (v.boom as i32 + dir).rem_euclid(4) as usize,
        _ => {}
    }
    match item {
        Item::ReduceFlash => s.reduce_flash = !s.reduce_flash,
        Item::Shake => {
            const ORDER: [ShakeSetting; 3] = [ShakeSetting::Off, ShakeSetting::Low, ShakeSetting::Full];
            let i = ORDER.iter().position(|x| *x == s.shake).unwrap_or(2) as i32;
            s.shake = ORDER[(i + dir).clamp(0, 2) as usize];
        }
        _ => {}
    }
    let t = &mut s.tuning;
    match item {
        Item::Speed => t.speed = step(t.speed, SPEED_RANGE, dir),
        Item::Focus => {
            // step through the presets from wherever the ratio is now
            let i = FOCUS_PRESETS.iter().position(|p| (p - t.focus_ratio).abs() < 0.005);
            let next = match (i, dir > 0) {
                (Some(i), true) => (i + 1).min(FOCUS_PRESETS.len() - 1),
                (Some(i), false) => i.saturating_sub(1),
                (None, true) => FOCUS_PRESETS.iter().position(|p| *p > t.focus_ratio).unwrap_or(FOCUS_PRESETS.len() - 1),
                (None, false) => FOCUS_PRESETS.iter().rposition(|p| *p < t.focus_ratio).unwrap_or(0),
            };
            t.focus_ratio = FOCUS_PRESETS[next];
        }
        Item::Blend => t.focus_blend = (t.focus_blend as i32 + dir).clamp(0, BLEND_MAX as i32) as u32,
        Item::Touch => s.touch_sens = step(s.touch_sens, TOUCH_SENS_RANGE, dir),
        Item::Stick => s.analog = !s.analog,
        Item::Reset => s.tuning = Default::default(),
        Item::Music => s.music_vol = (s.music_vol as i32 + dir).clamp(0, 10) as u8,
        Item::Sound => s.sound_vol = (s.sound_vol as i32 + dir).clamp(0, 10) as u8,
        Item::ShowHitbox => s.hitbox_always = !s.hitbox_always,
        Item::Fire => s.hold_beam = !s.hold_beam,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_steps_through_the_ab_presets() {
        let mut s = Settings::default();
        let mut seen = vec![s.tuning.focus_ratio];
        for _ in 0..10 {
            adjust(&mut s, Item::Focus, 1);
            seen.push(s.tuning.focus_ratio);
        }
        for want in [0.50, 0.55, 0.67] {
            let mut s = Settings::default();
            for _ in 0..6 {
                adjust(&mut s, Item::Focus, -1);
            }
            while (s.tuning.focus_ratio - want).abs() > 0.001 {
                adjust(&mut s, Item::Focus, 1);
            }
        }
        assert_eq!(*seen.last().unwrap(), *FOCUS_PRESETS.last().unwrap(), "stops at the end");
        s.tuning.focus_ratio = 0.52; // from a saved odd value
        adjust(&mut s, Item::Focus, 1);
        assert_eq!(s.tuning.focus_ratio, 0.55);
    }

    #[test]
    fn values_stay_in_range() {
        let mut s = Settings::default();
        for _ in 0..50 {
            adjust(&mut s, Item::Speed, 1);
            adjust(&mut s, Item::Blend, 1);
            adjust(&mut s, Item::Touch, 1);
        }
        assert_eq!((s.tuning.speed, s.tuning.focus_blend, s.touch_sens), (SPEED_RANGE.1, BLEND_MAX, TOUCH_SENS_RANGE.1));
        for _ in 0..50 {
            adjust(&mut s, Item::Speed, -1);
            adjust(&mut s, Item::Blend, -1);
            adjust(&mut s, Item::Touch, -1);
        }
        assert_eq!((s.tuning.speed, s.tuning.focus_blend), (SPEED_RANGE.0, 0));
        adjust(&mut s, Item::Reset, 0);
        assert_eq!(s.tuning, Default::default());
        assert_eq!(s.touch_sens, TOUCH_SENS_RANGE.0, "reset only touches the tuning");
    }

    #[test]
    fn the_sandbox_switches_flip_and_patterns_wrap() {
        let mut s = Settings::default();
        adjust(&mut s, Item::Pattern, -1);
        assert_eq!(s.view.pattern, SANDBOX.len() - 1);
        assert_eq!(value(&s, Item::Pattern), SANDBOX[SANDBOX.len() - 1].name);
        adjust(&mut s, Item::Pattern, 1);
        assert_eq!(s.view.pattern, 0);
        for item in [Item::Autopilot, Item::SlowMo, Item::Invincible, Item::Hitboxes, Item::Gray, Item::Fps] {
            assert_eq!(value(&s, item), "OFF");
            adjust(&mut s, item, 1);
            assert_eq!(value(&s, item), "ON");
        }
        let saved = Settings::parse(&s.format());
        assert_eq!(saved.view, Default::default(), "the sandbox switches aren't saved");
    }

    #[test]
    fn flashing_shake_and_the_boom_test_cycle() {
        let mut s = Settings::default();
        assert_eq!(value(&s, Item::ReduceFlash), "OFF");
        adjust(&mut s, Item::ReduceFlash, 1);
        assert!(s.reduce_flash);
        for _ in 0..3 {
            adjust(&mut s, Item::Shake, -1);
        }
        assert_eq!(s.shake, ShakeSetting::Off);
        assert_eq!(value(&s, Item::Boom), "SMALL");
        adjust(&mut s, Item::Boom, -1);
        assert_eq!(value(&s, Item::Boom), "BOSS");
        let saved = Settings::parse(&s.format());
        assert!(saved.reduce_flash && saved.shake == ShakeSetting::Off, "the options are saved");
    }

    #[test]
    fn the_options_change_and_are_saved() {
        let mut s = Settings::default();
        for _ in 0..20 {
            adjust(&mut s, Item::Music, -1);
            adjust(&mut s, Item::Sound, 1);
        }
        assert_eq!((s.music_vol, s.sound_vol), (0, 10), "volumes stop at their ends");
        assert_eq!((value(&s, Item::Music).as_str(), value(&s, Item::Sound).as_str()), ("OFF", "100%"));
        assert_eq!(value(&s, Item::Fire), "FOCUS BUTTON");
        adjust(&mut s, Item::Fire, 1);
        adjust(&mut s, Item::ShowHitbox, 1);
        assert_eq!((value(&s, Item::Fire).as_str(), value(&s, Item::ShowHitbox).as_str()), ("HOLD TO BEAM", "ALWAYS"));
        let saved = Settings::parse(&s.format());
        assert_eq!((saved.music_vol, saved.sound_vol, saved.hold_beam, saved.hitbox_always), (0, 10, true, true));
        assert!(OPTIONS.iter().all(|i| !i.label().is_empty()) && OPTIONS.last() == Some(&Item::Back));
    }

    #[test]
    fn labels_and_values_read() {
        let s = Settings::default();
        assert_eq!(value(&s, Item::Speed), "4.00 PX");
        assert_eq!(value(&s, Item::Focus), "0.55X");
        assert_eq!(value(&s, Item::Blend), "5 F");
        assert_eq!(value(&s, Item::Stick), "8-WAY");
        assert!(ITEMS.iter().all(|i| !i.label().is_empty()));
    }
}
