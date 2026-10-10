//! Plays the synthesized sound bank (ported from Last Train to Shinjuku): one-shots
//! with variation and throttling, and the beam's hum as a loop that fades in and out.

use crate::sim::{Event, World};
use crate::synth::{self, Id};
use crate::tune::*;
use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound, set_sound_volume, stop_sound};
use macroquad::prelude::get_time;
use std::collections::HashMap;

/// Base mix level and minimum seconds between plays (stops machine-gun stacking).
fn mix(id: Id) -> (f32, f64) {
    match id {
        Id::Shot => (VOL_SHOT, SHOT_SOUND_GAP),
        Id::Hit => (VOL_HIT, HIT_SOUND_GAP),
        Id::Break => (VOL_BREAK, 0.05),
        Id::BigBreak => (VOL_BIG_BREAK, 0.2),
        Id::Bump => (0.3, 0.2),
        Id::NotYet => (VOL_UI, 0.15),
        Id::Select => (VOL_UI, 0.04),
        Id::Beam => (VOL_BEAM, 0.0),
        Id::Graze => (VOL_GRAZE, GRAZE_SOUND_GAP),
        Id::Death => (VOL_DEATH, 0.3),
    }
}

/// Lets a sound through at most once every `gap` seconds.
#[derive(Default)]
pub struct Throttle {
    last: HashMap<Id, f64>,
}

impl Throttle {
    pub fn allow(&mut self, id: Id, now: f64, gap: f64) -> bool {
        if self.last.get(&id).is_some_and(|t| now - t < gap) {
            return false;
        }
        self.last.insert(id, now);
        true
    }
}

pub struct Audio {
    bank: HashMap<Id, Vec<Sound>>,
    throttle: Throttle,
    next_variant: HashMap<Id, usize>,
    beam: f32,
    beam_target: f32,
    beam_playing: bool,
    pub muted: bool,
}

impl Audio {
    /// Synthesize and load everything. On the web each sound decodes asynchronously.
    pub async fn load() -> Audio {
        let mut bank: HashMap<Id, Vec<Sound>> = HashMap::new();
        for (id, bytes) in synth::bank() {
            if let Ok(s) = load_sound_from_bytes(&bytes).await {
                bank.entry(id).or_default().push(s);
            }
        }
        Audio {
            bank,
            throttle: Throttle::default(),
            next_variant: HashMap::new(),
            beam: 0.0,
            beam_target: 0.0,
            beam_playing: false,
            muted: false,
        }
    }

    pub fn play(&mut self, id: Id, vol: f32) {
        if self.muted || vol <= 0.01 {
            return;
        }
        let (base, gap) = mix(id);
        if !self.throttle.allow(id, get_time(), gap) {
            return;
        }
        let Some(list) = self.bank.get(&id).filter(|l| !l.is_empty()) else { return };
        // cycle the variants so the same one never plays twice running
        let k = self.next_variant.entry(id).or_insert(0);
        let s = &list[*k % list.len()];
        *k += 1;
        play_sound(s, PlaySoundParams { looped: false, volume: (base * vol).min(1.0) });
    }

    /// Turn this frame's events into sounds.
    pub fn handle(&mut self, w: &World) {
        for e in &w.events {
            match *e {
                Event::Volley { .. } => self.play(Id::Shot, 1.0),
                Event::Hit { beam, .. } => self.play(Id::Hit, if beam { 0.6 } else { 1.0 }),
                Event::Break { big: false, .. } => self.play(Id::Break, 1.0),
                Event::Break { big: true, .. } => self.play(Id::BigBreak, 1.0),
                Event::Bump => self.play(Id::Bump, 1.0),
                Event::BombStub => self.play(Id::NotYet, 1.0),
                Event::Graze { .. } => self.play(Id::Graze, 1.0),
                Event::Death { .. } => self.play(Id::Death, 1.0),
            }
        }
        self.beam_target = if w.player.beaming { 1.0 } else { 0.0 };
    }

    /// Fade the beam hum toward where it should be. Call every display frame.
    pub fn update(&mut self, dt: f32, paused: bool) {
        let target = if self.muted || paused { 0.0 } else { self.beam_target };
        let step = BEAM_FADE * dt;
        self.beam = if self.beam < target { (self.beam + step).min(target) } else { (self.beam - step).max(target) };
        let Some(s) = self.bank.get(&Id::Beam).and_then(|l| l.first()) else { return };
        let vol = self.beam * VOL_BEAM;
        if self.beam > 0.001 {
            if self.beam_playing {
                set_sound_volume(s, vol);
            } else {
                play_sound(s, PlaySoundParams { looped: true, volume: vol });
                self.beam_playing = true;
            }
        } else if self.beam_playing {
            stop_sound(s);
            self.beam_playing = false;
        }
    }

    /// Leaving the stage: the hum stops at once.
    pub fn silence(&mut self) {
        self.beam_target = 0.0;
        self.beam = 0.0;
        self.update(0.0, true);
    }
}

/// Write every synthesized sound to `./shots/sfx/` (PG_WAV=1).
#[cfg(not(target_arch = "wasm32"))]
pub fn dump() {
    let _ = std::fs::create_dir_all("shots/sfx");
    let mut n: HashMap<Id, u32> = HashMap::new();
    for (id, bytes) in synth::bank() {
        let k = n.entry(id).or_insert(0);
        let path = format!("shots/sfx/{id:?}_{k}.wav");
        let _ = std::fs::write(&path, bytes);
        println!("wrote {path}");
        *k += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shot_sound_is_throttled() {
        // volleys come every 3 frames; count how many shot sounds a second of fire makes
        let mut t = Throttle::default();
        let plays = (0..20).filter(|i| t.allow(Id::Shot, *i as f64 * 3.0 / 60.0, SHOT_SOUND_GAP)).count();
        assert!(plays <= 11 && plays >= 8, "{plays} shot sounds a second");
        assert!(t.allow(Id::Hit, 0.0, HIT_SOUND_GAP), "each sound throttles separately");
    }
}
