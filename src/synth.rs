//! Procedural audio, ported from Last Train to Shinjuku. Every sound is
//! synthesized here at startup and handed to macroquad as an in-memory WAV.
//! No audio files.

use std::f32::consts::TAU;

pub const SR: f32 = 44100.0;

// ---------------------------------------------------------------- building blocks

pub struct Rng(u32);
impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng(seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453) | 1)
    }
    /// White noise in -1..1
    pub fn noise(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (self.noise() * 0.5 + 0.5) * (b - a)
    }
}

#[derive(Default)]
struct Lp(f32);
impl Lp {
    fn run(&mut self, x: f32, fc: f32) -> f32 {
        let a = 1.0 - (-TAU * fc.max(1.0) / SR).exp();
        self.0 += a * (x - self.0);
        self.0
    }
}

#[derive(Default)]
struct Hp(Lp);
impl Hp {
    fn run(&mut self, x: f32, fc: f32) -> f32 {
        x - self.0.run(x, fc)
    }
}

/// RBJ biquad, recomputed per call so cutoffs can sweep.
#[derive(Default)]
struct Bq {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}
impl Bq {
    fn process(&mut self, x: f32, b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) -> f32 {
        let y = (b0 * x + b1 * self.x1 + b2 * self.x2 - a1 * self.y1 - a2 * self.y2) / a0;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
    fn bandpass(&mut self, x: f32, fc: f32, q: f32) -> f32 {
        let w = TAU * fc.clamp(20.0, SR * 0.45) / SR;
        let alpha = w.sin() / (2.0 * q);
        self.process(x, alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * w.cos(), 1.0 - alpha)
    }
    fn lowpass(&mut self, x: f32, fc: f32, q: f32) -> f32 {
        let w = TAU * fc.clamp(20.0, SR * 0.45) / SR;
        let (c, alpha) = (w.cos(), w.sin() / (2.0 * q));
        let b1 = 1.0 - c;
        self.process(x, b1 * 0.5, b1, b1 * 0.5, 1.0 + alpha, -2.0 * c, 1.0 - alpha)
    }
}

fn saw(ph: f32) -> f32 {
    2.0 * (ph - ph.floor()) - 1.0
}
fn sq(ph: f32, duty: f32) -> f32 {
    if ph - ph.floor() < duty { 1.0 } else { -1.0 }
}
fn attack(t: f32, a: f32) -> f32 {
    (t / a).min(1.0)
}

fn buf(secs: f32) -> Vec<f32> {
    vec![0.0; (secs * SR) as usize]
}

fn normalize(b: &mut [f32], peak: f32) {
    let m = b.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    if m > 1e-6 {
        let k = peak / m;
        b.iter_mut().for_each(|x| *x *= k);
    }
}

fn drive(b: &mut [f32], amount: f32) {
    b.iter_mut().for_each(|x| *x = (*x * amount).tanh());
}

/// Small Schroeder reverb (4 combs + 2 allpasses). `size` scales the delay lines.
fn reverb(b: &[f32], mix: f32, size: f32, fb: f32) -> Vec<f32> {
    let combs = [0.0297, 0.0371, 0.0411, 0.0437].map(|d| ((d * size * SR) as usize).max(1));
    let aps = [0.005, 0.0017].map(|d| ((d * SR) as usize).max(1));
    let mut lines: Vec<Vec<f32>> = combs.iter().map(|&n| vec![0.0; n]).collect();
    let mut idx = [0usize; 4];
    let mut damp = [0.0f32; 4];
    let mut out = Vec::with_capacity(b.len());
    for &x in b {
        let mut acc = 0.0;
        for k in 0..4 {
            let y = lines[k][idx[k]];
            damp[k] = y * 0.6 + damp[k] * 0.4;
            lines[k][idx[k]] = x + damp[k] * fb;
            idx[k] = (idx[k] + 1) % lines[k].len();
            acc += y;
        }
        out.push(acc * 0.25);
    }
    for &n in &aps {
        let mut line = vec![0.0; n];
        let mut i = 0;
        for s in out.iter_mut() {
            let d = line[i];
            let y = -*s * 0.5 + d;
            line[i] = *s + d * 0.5;
            i = (i + 1) % n;
            *s = y;
        }
    }
    b.iter().zip(out).map(|(d, w)| d * (1.0 - mix * 0.5) + w * mix).collect()
}

/// 16-bit PCM WAV. `ch` channels, samples interleaved.
pub fn wav(samples: &[f32], ch: u16) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut v = Vec::with_capacity(44 + data_len as usize);
    let sr = SR as u32;
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + data_len).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&ch.to_le_bytes());
    v.extend_from_slice(&sr.to_le_bytes());
    v.extend_from_slice(&(sr * ch as u32 * 2).to_le_bytes());
    v.extend_from_slice(&(ch * 2).to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data");
    v.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let i = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        v.extend_from_slice(&i.to_le_bytes());
    }
    v
}

// ---------------------------------------------------------------- from Shinjuku

pub fn tick(seed: u32) -> Vec<f32> {
    let mut b = buf(0.08);
    let mut r = Rng::new(seed);
    let mut hp = Hp::default();
    let f = r.range(2600.0, 3600.0);
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        *s = hp.run(r.noise(), 2500.0) * (-t * 120.0).exp() + (TAU * f * t).sin() * (-t * 45.0).exp() * 0.35;
    }
    normalize(&mut b, 0.7);
    b
}

pub fn hit(seed: u32) -> Vec<f32> {
    let mut b = buf(0.14);
    let mut r = Rng::new(seed);
    let mut lp = Lp::default();
    let f = r.range(80.0, 110.0);
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        *s = lp.run(r.noise(), 900.0) * (-t * 38.0).exp() * 1.6 + (TAU * f * t).sin() * (-t * 34.0).exp();
    }
    drive(&mut b, 1.4);
    normalize(&mut b, 0.8);
    b
}

pub fn explosion(seed: u32, heavy: bool) -> Vec<f32> {
    let len = if heavy { 1.4 } else { 2.0 };
    let mut b = buf(len);
    let mut r = Rng::new(seed);
    let (mut l1, mut l2) = (Lp::default(), Lp::default());
    let mut ph = 0.0f32;
    let top = if heavy { 2200.0 } else { 5500.0 };
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let mut n = r.noise();
        if r.noise() > 0.996 - 0.01 * (-t * 3.0).exp() {
            n *= 6.0 * (-t * 2.0).exp(); // crackle
        }
        let cut = 140.0 + top * (-t * 4.5).exp();
        let body = l2.run(l1.run(n, cut), cut) * (-t * 2.6).exp() * attack(t, 0.004) * 2.0;
        ph += (28.0 + 62.0 * (-t * 5.5).exp()) / SR;
        let boom = (TAU * ph).sin() * (-t * if heavy { 2.8 } else { 3.6 }).exp() * if heavy { 1.6 } else { 1.2 };
        *s = body + boom;
    }
    let mut b = reverb(&b, 0.38, 1.6, 0.7);
    drive(&mut b, 1.8);
    normalize(&mut b, 0.95);
    b
}

pub fn select() -> Vec<f32> {
    let mut b = buf(0.12);
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let f = if t < 0.05 { 880.0 } else { 1320.0 };
        *s = sq(f * t, 0.5) * (-t * 18.0).exp() * 0.4;
    }
    normalize(&mut b, 0.6);
    b
}

// ---------------------------------------------------------------- the Pilgrim

/// The Pilgrim's shot: soft and short, so a constant stream never tires the ear.
pub fn pilgrim_shot(seed: u32, pitch: f32) -> Vec<f32> {
    let mut b = buf(0.09);
    let mut r = Rng::new(seed);
    let (mut lp, mut hp, mut soft) = (Lp::default(), Hp::default(), Lp::default());
    let mut ph = 0.0f32;
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let tick = hp.run(lp.run(r.noise(), 4200.0), 900.0) * (-t * 90.0).exp();
        ph += pitch * (0.55 + 0.45 * (-t * 60.0).exp()) / SR;
        let tone = sq(ph, 0.5) * (-t * 38.0).exp() * 0.35;
        *s = soft.run(tick * 0.9 + tone, 2600.0);
    }
    normalize(&mut b, 0.7);
    b
}

/// The beam's hum, a seamless 2 s loop: a low detuned drone with a sizzle on top.
/// Every modulation completes a whole number of cycles in the loop.
pub fn beam_hum() -> Vec<f32> {
    let n = (2.0 * SR) as usize;
    let extra = (0.25 * SR) as usize;
    let mut b = vec![0.0; n + extra];
    let mut r = Rng::new(77);
    let (mut lp, mut bp) = (Bq::default(), Bq::default());
    let (mut p1, mut p2, mut p3) = (0.0f32, 0.0f32, 0.0f32);
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        p1 += 55.0 / SR;
        p2 += 55.5 / SR;
        p3 += 110.0 / SR;
        let drone = saw(p1) * 0.5 + saw(p2) * 0.5 + sq(p3, 0.3) * 0.2;
        let body = lp.lowpass(drone, 700.0 + 300.0 * (TAU * 0.5 * t).sin(), 1.2);
        let trem = 0.85 + 0.15 * (TAU * 15.0 * t).sin();
        let sizzle = bp.bandpass(r.noise(), 3200.0, 2.0) * 0.25;
        *s = body * trem + sizzle;
    }
    // crossfade the extra tail over the start
    for i in 0..extra {
        let k = i as f32 / extra as f32;
        b[i] = b[i] * k + b[n + i] * (1.0 - k);
    }
    b.truncate(n);
    normalize(&mut b, 0.6);
    b
}

/// The bomb button before the bomb exists: a soft two-note "not yet".
pub fn not_yet() -> Vec<f32> {
    let mut b = buf(0.22);
    let mut lp = Lp::default();
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let (f, t0) = if t < 0.09 { (587.0, t) } else { (392.0, t - 0.09) };
        *s = lp.run(sq(f * t, 0.25) * (-t0 * 22.0).exp() * 0.4, 3000.0);
    }
    normalize(&mut b, 0.6);
    b
}

/// The small blast: a short crack and thump.
pub fn pop(seed: u32) -> Vec<f32> {
    let mut b = buf(0.45);
    let mut r = Rng::new(seed);
    let (mut l1, mut l2) = (Lp::default(), Lp::default());
    let mut ph = 0.0f32;
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let cut = 300.0 + 6000.0 * (-t * 14.0).exp();
        let body = l2.run(l1.run(r.noise(), cut), cut) * (-t * 9.0).exp() * attack(t, 0.002) * 2.0;
        ph += (50.0 + 90.0 * (-t * 16.0).exp()) / SR;
        *s = body + (TAU * ph).sin() * (-t * 10.0).exp() * 0.9;
    }
    let mut b = reverb(&b, 0.25, 0.9, 0.55);
    drive(&mut b, 1.6);
    normalize(&mut b, 0.9);
    b
}

/// The boss's death: two heavy blasts, the second rolling on and on.
pub fn boss_blast(seed: u32) -> Vec<f32> {
    let a = explosion(seed, true);
    let late = explosion(seed + 1, true);
    let gap = (0.35 * SR) as usize;
    let mut b = vec![0.0; a.len().max(late.len() + gap) + (1.0 * SR) as usize];
    for (i, s) in a.iter().enumerate() {
        b[i] += s * 0.8;
    }
    for (i, s) in late.iter().enumerate() {
        b[i + gap] += s;
    }
    let mut b = reverb(&b, 0.5, 2.2, 0.75);
    drive(&mut b, 1.4);
    normalize(&mut b, 0.95);
    b
}

/// The bomb: Shinjuku's grenade in its heavy variant, over a deep downward sweep
/// and a rushing whoosh as the ring spreads.
pub fn bomb(seed: u32) -> Vec<f32> {
    let boom = explosion(seed, true);
    let mut b = vec![0.0; boom.len() + (0.6 * SR) as usize];
    let mut r = Rng::new(seed + 1);
    let mut bp = Bq::default();
    let mut ph = 0.0f32;
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        ph += (90.0 * (-t * 2.2).exp() + 26.0) / SR;
        let sub = (TAU * ph).sin() * (-t * 1.6).exp() * 0.9;
        let whoosh = bp.bandpass(r.noise(), 300.0 + 2600.0 * (-t * 3.0).exp(), 1.2) * (-t * 2.5).exp() * attack(t, 0.03) * 0.8;
        *s = sub + whoosh + boom.get(i).copied().unwrap_or(0.0) * 1.1;
    }
    let mut b = reverb(&b, 0.45, 2.0, 0.72);
    drive(&mut b, 1.5);
    normalize(&mut b, 0.95);
    b
}

/// Struck: a hard thud and two quick alarm blips (bomb now!).
pub fn struck(seed: u32) -> Vec<f32> {
    let thud = hit(seed);
    let mut b = buf(0.32);
    let mut lp = Lp::default();
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let blip = if t < 0.06 || (0.09..0.15).contains(&t) { sq(1320.0 * t, 0.5) * 0.35 } else { 0.0 };
        *s = lp.run(blip, 4000.0) + thud.get(i).copied().unwrap_or(0.0);
    }
    normalize(&mut b, 0.85);
    b
}

/// The next ship flies in: three soft rising notes.
pub fn respawn_chime() -> Vec<f32> {
    let mut b = buf(0.6);
    for (k, f) in [523.25f32, 659.25, 783.99].iter().enumerate() {
        let t0 = k as f32 * 0.09;
        for (i, s) in b.iter_mut().enumerate() {
            let t = i as f32 / SR - t0;
            if t > 0.0 {
                *s += (TAU * f * t).sin() * (-t * 6.0).exp() * attack(t, 0.01) * 0.4;
            }
        }
    }
    let b = reverb(&b, 0.35, 1.2, 0.6);
    let mut b = b;
    normalize(&mut b, 0.6);
    b
}

/// Game over: a slow falling phrase over a low drone.
pub fn game_over() -> Vec<f32> {
    let mut b = buf(2.4);
    let mut lp = Lp::default();
    for (k, f) in [392.0f32, 349.23, 311.13, 261.63].iter().enumerate() {
        let t0 = k as f32 * 0.32;
        for (i, s) in b.iter_mut().enumerate() {
            let t = i as f32 / SR - t0;
            if t > 0.0 {
                *s += sq(f * t, 0.3) * (-t * 2.5).exp() * attack(t, 0.01) * 0.25;
            }
        }
    }
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        *s = lp.run(*s, 1800.0) + (TAU * 65.41 * t).sin() * (1.0 - t / 2.4).max(0.0) * 0.25;
    }
    let mut b = reverb(&b, 0.4, 1.6, 0.7);
    normalize(&mut b, 0.7);
    b
}

/// A graze: a tiny bright tick, quiet enough to hear dozens of.
pub fn graze(seed: u32) -> Vec<f32> {
    let mut b = buf(0.045);
    let mut r = Rng::new(seed);
    let mut hp = Hp::default();
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        *s = hp.run(r.noise(), 5000.0) * (-t * 140.0).exp() * 0.6 + (TAU * 2900.0 * t).sin() * (-t * 90.0).exp() * 0.5;
    }
    normalize(&mut b, 0.6);
    b
}

/// The placeholder death: a crunch and a falling tone.
pub fn death(seed: u32) -> Vec<f32> {
    let mut b = explosion(seed, false);
    b.truncate((0.9 * SR) as usize);
    let mut ph = 0.0f32;
    for (i, s) in b.iter_mut().enumerate() {
        let t = i as f32 / SR;
        ph += (520.0 * (-t * 3.0).exp() + 60.0) / SR;
        *s = *s * 0.8 + sq(ph, 0.5) * (-t * 4.0).exp() * 0.25;
        *s *= (1.0 - t / 0.9).max(0.0);
    }
    normalize(&mut b, 0.9);
    b
}

// ---------------------------------------------------------------- the whole bank

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Id {
    Shot,
    Hit,
    Bump,
    NotYet,
    Select,
    Beam,
    Graze,
    Death,
    BlastSmall,
    BlastMedium,
    BlastLarge,
    BlastBoss,
    Bomb,
    Struck,
    Respawn,
    GameOver,
}

/// Every sound as (id, wav bytes). Variants share an id.
pub fn bank() -> Vec<(Id, Vec<u8>)> {
    let mono = |b: Vec<f32>| wav(&b, 1);
    let mut v = Vec::new();
    for (k, p) in [880.0, 940.0, 1000.0].iter().enumerate() {
        v.push((Id::Shot, mono(pilgrim_shot(100 + k as u32, *p))));
    }
    for k in 0..2 {
        v.push((Id::Hit, mono(tick(200 + k))));
    }
    for k in 0..2 {
        v.push((Id::BlastSmall, mono(pop(860 + k))));
        v.push((Id::BlastMedium, mono(explosion(900 + k, false))));
    }
    v.push((Id::BlastLarge, mono(explosion(950, true))));
    v.push((Id::BlastBoss, mono(boss_blast(970))));
    v.push((Id::Bomb, mono(bomb(980))));
    v.push((Id::Struck, mono(struck(990))));
    v.push((Id::Respawn, mono(respawn_chime())));
    v.push((Id::GameOver, mono(game_over())));
    v.push((Id::Bump, mono(hit(300))));
    v.push((Id::NotYet, mono(not_yet())));
    v.push((Id::Select, mono(select())));
    v.push((Id::Beam, mono(beam_hum())));
    for k in 0..2 {
        v.push((Id::Graze, mono(graze(400 + k))));
    }
    v.push((Id::Death, mono(death(500))));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sound_is_a_clean_wav() {
        for (id, bytes) in bank() {
            assert_eq!(&bytes[0..4], b"RIFF", "{id:?}");
            assert_eq!(&bytes[8..12], b"WAVE");
            assert!(bytes.len() > 44 + 100, "{id:?} is empty");
        }
    }

    #[test]
    fn nothing_clips() {
        for b in [pilgrim_shot(1, 900.0), beam_hum(), not_yet(), tick(2), explosion(3, true), graze(4), death(5), pop(6), boss_blast(7), bomb(8), struck(9), respawn_chime(), game_over()] {
            assert!(b.iter().all(|s| s.is_finite() && s.abs() <= 1.0));
        }
    }

    #[test]
    fn the_beam_hum_loops_without_a_click() {
        let b = beam_hum();
        let (first, last) = (b[0], b[b.len() - 1]);
        assert!((first - last).abs() < 0.08, "seam jump {}", (first - last).abs());
    }

    #[test]
    fn shots_are_short_and_soft() {
        let b = pilgrim_shot(1, 900.0);
        assert!(b.len() as f32 / SR < 0.1);
    }
}
