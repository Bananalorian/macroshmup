//! The rumble map: which moments shake the controller, and how hard. Firing hums
//! gently while held; breaking things, bumping and the bomb button kick. Phones
//! playing by touch get a short buzz on the big moments instead.

use crate::sim::{Event, World};
use crate::storage::Rumble;
use crate::tune::*;

/// (strong motor, weak motor, ms)
pub type Kick = (f32, f32, u32);

/// One-off rumble for an event.
pub fn for_event(e: &Event) -> Option<Kick> {
    match e {
        Event::Break { big: false, .. } => Some(RUMBLE_BREAK),
        Event::Break { big: true, .. } => Some(RUMBLE_BIG_BREAK),
        Event::Bump => Some(RUMBLE_BUMP),
        Event::Bomb { .. } => Some(RUMBLE_BOMB),
        Event::Struck { .. } => Some(RUMBLE_STRUCK),
        Event::Death { .. } => Some(RUMBLE_DEATH),
        Event::Respawn => Some(RUMBLE_RESPAWN),
        Event::GameOver => Some(RUMBLE_GAME_OVER),
        Event::Graze { .. } => Some(RUMBLE_GRAZE),
        Event::Volley { .. } | Event::Hit { .. } | Event::BombSwept | Event::NoBomb | Event::Cancel => None,
    }
}

/// The steady hum while shooting, re-sent every few frames so it lasts while held.
pub fn held(w: &World) -> Option<Kick> {
    if w.frame % RUMBLE_REPEAT != 0 {
        return None;
    }
    if w.player.beaming {
        Some(RUMBLE_BEAM)
    } else if w.player.firing {
        Some(RUMBLE_FIRE)
    } else {
        None
    }
}

/// Scale by the player's setting; `None` when it rounds to nothing.
pub fn scaled(k: Kick, level: Rumble) -> Option<Kick> {
    let s = level.scale();
    let (a, b) = (k.0 * s, k.1 * s);
    (a.max(b) > 0.01).then_some((a, b, k.2))
}

/// Phone buzz length (ms) for touch players, only for the big moments.
pub fn buzz(e: &Event) -> Option<u32> {
    match e {
        Event::Break { big: true, .. } => Some(40),
        Event::Bomb { .. } => Some(60),
        Event::Struck { .. } => Some(30),
        Event::Death { .. } => Some(80),
        Event::Bump => Some(15),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::player::{Controls, Tuning};

    #[test]
    fn big_moments_kick_harder() {
        let small = for_event(&Event::Break { x: 0.0, y: 0.0, big: false }).unwrap();
        let big = for_event(&Event::Break { x: 0.0, y: 0.0, big: true }).unwrap();
        let bomb = for_event(&Event::Bomb { x: 0.0, y: 0.0 }).unwrap();
        assert!(big.0 > small.0 && bomb.0 >= big.0);
        assert!(for_event(&Event::Hit { x: 0.0, y: 0.0, beam: false }).is_none(), "every hit would be a constant buzz");
    }

    #[test]
    fn shooting_hums_and_the_beam_hums_harder() {
        let mut w = World::new(1, Tuning::default());
        w.dummies.clear();
        let mut fire = None;
        let mut beam = None;
        for _ in 0..RUMBLE_REPEAT {
            w.tick(Controls { fire: true, ..Default::default() });
            fire = fire.or(held(&w));
        }
        for _ in 0..RUMBLE_REPEAT {
            w.tick(Controls { fire: true, focus: true, ..Default::default() });
            beam = beam.or(held(&w));
        }
        let (fire, beam) = (fire.unwrap(), beam.unwrap());
        assert!(beam.0 + beam.1 > fire.0 + fire.1);
        assert!(fire.2 as u64 > RUMBLE_REPEAT * 1000 / 60, "each pulse outlasts the gap to the next");
        w.tick(Controls::default());
        assert!((0..8).all(|_| {
            w.tick(Controls::default());
            held(&w).is_none()
        }));
    }

    #[test]
    fn the_setting_scales_or_silences() {
        let k = RUMBLE_BOMB;
        assert_eq!(scaled(k, Rumble::Full), Some(k));
        assert_eq!(scaled(k, Rumble::Low), Some((k.0 * RUMBLE_LOW, k.1 * RUMBLE_LOW, k.2)));
        assert_eq!(scaled(k, Rumble::Off), None);
    }
}
