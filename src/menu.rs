//! The pause menu, which is also the live tuning panel: speed, focus ratio and
//! blend frames change the game the moment you resume (and are saved).

use crate::storage::{Rumble, Settings};
use crate::tune::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Resume,
    Speed,
    Focus,
    Blend,
    Touch,
    Stick,
    Rumble,
    Reset,
    Quit,
}

pub const ITEMS: [Item; 9] = [Item::Resume, Item::Speed, Item::Focus, Item::Blend, Item::Touch, Item::Stick, Item::Rumble, Item::Reset, Item::Quit];

impl Item {
    pub fn label(self) -> &'static str {
        match self {
            Item::Resume => "RESUME",
            Item::Speed => "SPEED",
            Item::Focus => "FOCUS",
            Item::Blend => "BLEND",
            Item::Touch => "TOUCH",
            Item::Stick => "STICK",
            Item::Rumble => "RUMBLE",
            Item::Reset => "RESET TUNING",
            Item::Quit => "QUIT TO TITLE",
        }
    }

    /// Items with a value you change with left and right.
    pub fn adjustable(self) -> bool {
        !matches!(self, Item::Resume | Item::Reset | Item::Quit)
    }
}

/// The value as shown, e.g. `4.00 PX`.
pub fn value(s: &Settings, item: Item) -> String {
    match item {
        Item::Speed => format!("{:.2} PX", s.tuning.speed),
        Item::Focus => format!("{:.2}X", s.tuning.focus_ratio),
        Item::Blend => format!("{} F", s.tuning.focus_blend),
        Item::Touch => format!("{:.1}X", s.touch_sens),
        Item::Stick => (if s.analog { "ANALOG" } else { "8-WAY" }).to_string(),
        Item::Rumble => match s.rumble {
            Rumble::Off => "OFF",
            Rumble::Low => "LOW",
            Rumble::Full => "FULL",
        }
        .to_string(),
        _ => String::new(),
    }
}

fn step(v: f32, (lo, hi, by): (f32, f32, f32), dir: i32) -> f32 {
    let n = ((v - lo) / by).round() + dir as f32;
    (lo + n * by).clamp(lo, hi)
}

/// Change an item's value one step left (-1) or right (1).
pub fn adjust(s: &mut Settings, item: Item, dir: i32) {
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
        Item::Rumble => {
            const ORDER: [Rumble; 3] = [Rumble::Off, Rumble::Low, Rumble::Full];
            let i = ORDER.iter().position(|r| *r == s.rumble).unwrap_or(2) as i32;
            s.rumble = ORDER[(i + dir).clamp(0, 2) as usize];
        }
        Item::Reset => s.tuning = Default::default(),
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
            adjust(&mut s, Item::Rumble, 1);
        }
        assert_eq!((s.tuning.speed, s.tuning.focus_blend, s.touch_sens, s.rumble), (SPEED_RANGE.1, BLEND_MAX, TOUCH_SENS_RANGE.1, Rumble::Full));
        for _ in 0..50 {
            adjust(&mut s, Item::Speed, -1);
            adjust(&mut s, Item::Blend, -1);
            adjust(&mut s, Item::Rumble, -1);
        }
        assert_eq!((s.tuning.speed, s.tuning.focus_blend, s.rumble), (SPEED_RANGE.0, 0, Rumble::Off));
        adjust(&mut s, Item::Reset, 0);
        assert_eq!(s.tuning, Default::default());
        assert_eq!(s.rumble, Rumble::Off, "reset only touches the tuning");
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
