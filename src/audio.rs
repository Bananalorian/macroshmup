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
        Id::BlastSmall => (VOL_BLAST[0], 0.04),
        Id::BlastMedium => (VOL_BLAST[1], 0.05),
        Id::BlastLarge => (VOL_BLAST[2], 0.15),
        Id::BlastBoss => (VOL_BLAST[3], 0.5),
        Id::Bomb => (VOL_BOMB, 0.3),
        Id::Struck => (VOL_STRUCK, 0.2),
        Id::Respawn => (VOL_RESPAWN, 0.5),
        Id::GameOver => (VOL_GAME_OVER, 1.0),
        Id::Bump => (0.3, 0.2),
        Id::NotYet => (VOL_UI, 0.15),
        Id::Select => (VOL_UI, 0.04),
        Id::Beam => (VOL_BEAM, 0.0),
        Id::Graze => (VOL_GRAZE, GRAZE_SOUND_GAP),
        Id::Death => (VOL_DEATH, 0.3),
        Id::Warning => (VOL_WARNING, 2.0),
        Id::BossTheme => (VOL_MUSIC, 0.0),
        Id::Approach => (VOL_APPROACH, 4.0),
        Id::Roar => (VOL_ROAR, 1.0),
        Id::LaserCharge => (VOL_LASER, 0.3),
        Id::LaserBeam => (VOL_LASER, 0.3),
        Id::Glitch => (VOL_GLITCH, 0.12),
        Id::Chomp => (VOL_CHOMP, 0.15),
        Id::Crunch => (VOL_CRUNCH, 0.04),
        Id::Swallowed => (VOL_SWALLOWED, 2.0),
        Id::Tick => (VOL_TICK, 0.03),
        Id::Slam => (VOL_SLAM, 0.3),
        Id::Extend => (VOL_EXTEND, 1.0),
    }
}

/// An options volume (0-10) as a gain: a gentle curve, so the low steps stay usable.
pub fn gain(v: u8) -> f32 {
    (v.min(10) as f32 / 10.0).powf(1.5)
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
    /// The boss's theme: how loud it is now, where it's heading, and whether it's playing.
    music: f32,
    music_target: f32,
    music_playing: bool,
    pub muted: bool,
    /// The options' music and sound volumes, as gains.
    music_gain: f32,
    sfx_gain: f32,
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
            music: 0.0,
            music_target: 0.0,
            music_playing: false,
            muted: false,
            music_gain: gain(VOLUME_DEFAULT),
            sfx_gain: gain(VOLUME_DEFAULT),
        }
    }

    /// The options' volumes, 0-10.
    pub fn volumes(&mut self, music: u8, sound: u8) {
        self.music_gain = gain(music);
        self.sfx_gain = gain(sound);
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
        play_sound(s, PlaySoundParams { looped: false, volume: (base * vol * self.sfx_gain).min(1.0) });
    }

    /// Turn this frame's events into sounds.
    pub fn handle(&mut self, w: &World) {
        for e in &w.events {
            match *e {
                Event::Volley { .. } => self.play(Id::Shot, 1.0),
                Event::Hit { beam, .. } => self.play(Id::Hit, if beam { 0.6 } else { 1.0 }),
                // blasts are played by the explosion kit, which knows their size
                Event::Break { .. } => {}
                Event::Bump => self.play(Id::Bump, 1.0),
                Event::Graze { .. } => self.play(Id::Graze, 1.0),
                Event::Bomb { .. } => self.play(Id::Bomb, 1.0),
                Event::NoBomb => self.play(Id::NotYet, 1.0),
                Event::Struck { .. } => self.play(Id::Struck, 1.0),
                Event::Death { .. } => self.play(Id::Death, 1.0),
                Event::Respawn => self.play(Id::Respawn, 1.0),
                Event::GameOver => self.play(Id::GameOver, 1.0),
                Event::Warning => self.play(Id::Warning, 1.0),
                Event::StageEnd => self.play(Id::Respawn, 1.0),
                // kills, broken turrets and cracks are blasts too
                Event::BombSwept | Event::Cancel | Event::Kill { .. } | Event::PartBreak { .. } | Event::Crack { .. } | Event::Section(_) => {}
                // the boss: its approach, then its theme from the breach till it goes down
                Event::BossArrive => self.play(Id::Approach, 1.0),
                Event::BossBreach { .. } => self.music_target = 1.0,
                Event::BossRoar { .. } => self.play(Id::Roar, 1.0),
                Event::BossBite { .. } => self.play(Id::Chomp, 1.0),
                Event::LaserWarn => self.play(Id::LaserCharge, 1.0),
                Event::LaserFire => self.play(Id::LaserBeam, 1.0),
                Event::BossTurn | Event::BossBreak { .. } => self.play(Id::Glitch, 1.0),
                Event::BossDown { .. } => self.music_target = 0.0,
                Event::BossFall => {
                    self.music_target = 0.0;
                    self.play(Id::Swallowed, 1.0);
                }
                // fins breaking and the death's blasts are blasts too
                Event::FinBreak { .. } | Event::BossPhase(_) | Event::BossBlast { .. } | Event::BossGone => {}
                Event::Points { how: crate::sim::score::Award::Grade(..), .. } => self.play(Id::Slam, 1.0),
                Event::Points { .. } => {}
                Event::Extend => self.play(Id::Extend, 1.0),
            }
        }
        self.beam_target = if w.player.beaming { 1.0 } else { 0.0 };
    }

    /// A kill: the blast that matches its size, a crunch under it.
    pub fn kill(&mut self, size: crate::render::explode::Size) {
        self.blast(size);
        self.play(Id::Crunch, CRUNCH_BY_SIZE[size as usize]);
    }

    /// The blast that matches an explosion's size.
    pub fn blast(&mut self, size: crate::render::explode::Size) {
        use crate::render::explode::Size;
        let id = match size {
            Size::Small => Id::BlastSmall,
            Size::Medium => Id::BlastMedium,
            Size::Large => Id::BlastLarge,
            Size::Boss => Id::BlastBoss,
        };
        self.play(id, 1.0);
    }

    /// Fade the boss's theme toward where it should be (silent while paused).
    fn update_music(&mut self, dt: f32, paused: bool) {
        let target = if self.muted || paused { 0.0 } else { self.music_target };
        let step = MUSIC_FADE * dt;
        self.music = if self.music < target { (self.music + step).min(target) } else { (self.music - step).max(target) };
        let Some(s) = self.bank.get(&Id::BossTheme).and_then(|l| l.first()) else { return };
        if self.music > 0.001 {
            if self.music_playing {
                set_sound_volume(s, self.music * VOL_MUSIC * self.music_gain);
            } else if self.music_target > 0.0 {
                play_sound(s, PlaySoundParams { looped: true, volume: self.music * VOL_MUSIC * self.music_gain });
                self.music_playing = true;
            }
        } else if self.music_playing && self.music_target == 0.0 {
            stop_sound(s);
            self.music_playing = false;
        }
    }

    /// Fade the beam hum toward where it should be. Call every display frame.
    pub fn update(&mut self, dt: f32, paused: bool) {
        self.update_music(dt, paused);
        let target = if self.muted || paused { 0.0 } else { self.beam_target };
        let step = BEAM_FADE * dt;
        self.beam = if self.beam < target { (self.beam + step).min(target) } else { (self.beam - step).max(target) };
        let Some(s) = self.bank.get(&Id::Beam).and_then(|l| l.first()) else { return };
        let vol = self.beam * VOL_BEAM * self.sfx_gain;
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

    /// Start (or fade out) the boss's theme: boss practice starts mid-fight, past the
    /// breach that would have started it.
    pub fn music(&mut self, on: bool) {
        self.music_target = if on { 1.0 } else { 0.0 };
    }

    /// Leaving the stage: the hum and the music stop at once.
    pub fn silence(&mut self) {
        self.beam_target = 0.0;
        self.beam = 0.0;
        self.music_target = 0.0;
        self.music = 0.0;
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
    fn volumes_curve_from_silent_to_full() {
        assert_eq!(gain(0), 0.0);
        assert_eq!(gain(10), 1.0);
        assert!((1..=10).all(|v| gain(v) > gain(v - 1)), "every step is louder");
        assert!(gain(5) < 0.5, "a gentle curve");
    }

    #[test]
    fn the_shot_sound_is_throttled() {
        // volleys come every 3 frames; count how many shot sounds a second of fire makes
        let mut t = Throttle::default();
        let plays = (0..20).filter(|i| t.allow(Id::Shot, *i as f64 * 3.0 / 60.0, SHOT_SOUND_GAP)).count();
        assert!(plays <= 11 && plays >= 8, "{plays} shot sounds a second");
        assert!(t.allow(Id::Hit, 0.0, HIT_SOUND_GAP), "each sound throttles separately");
    }
}
