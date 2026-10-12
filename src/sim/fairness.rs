//! The fairness checks from Sprint 2, run against every sandbox pattern:
//! determinism, no bullets spawning on the player, no walls, the spawn bloom,
//! grazing, the placeholder death, and the bot surviving each pattern.

use super::bot;
use super::bullets::{Behavior, Kind, Spawn};
use super::player::Controls;
use super::World;
use crate::tune::*;

/// The patterns with guns (not OFF, not the stress fills).
fn patterns() -> Vec<usize> {
    (0..SANDBOX.len()).filter(|&i| !SANDBOX[i].guns.is_empty()).collect()
}

fn world(preset: usize, seed: u64) -> World {
    let mut w = World::new(seed, Default::default());
    w.set_pattern(preset);
    w
}

#[test]
fn the_same_seed_gives_the_same_bullet_field() {
    for p in patterns() {
        let run = || {
            let mut w = world(p, 42);
            let mut bot = bot::Bot::new();
            for _ in 0..600 {
                let c = bot.decide(&w);
                w.tick(c);
                w.events.clear();
            }
            let b = &w.hazards.bullets;
            (b.x.clone(), b.y.clone(), b.vx.clone(), b.vy.clone(), b.age.clone(), w.player.x, w.player.y)
        };
        let (a, b) = (run(), run());
        assert!(!a.0.is_empty(), "{} fired nothing", SANDBOX[p].name);
        assert!(a == b, "{} differs between runs", SANDBOX[p].name);
    }
}

#[test]
fn no_bullet_spawns_within_24_px_of_the_player() {
    for p in patterns() {
        let mut w = world(p, 7);
        w.invincible = true;
        for f in 0..1500u32 {
            // park the ship right on top of each gun in turn, and under the split points
            let e = &w.hazards.emitters[(f / 100) as usize % w.hazards.emitters.len()];
            let (x, y) = if f % 200 < 100 { (e.x, e.y + 4.0) } else { (135.0, 160.0) };
            w.player.x = x.clamp(EDGE_X, FIELD_W as f32 - EDGE_X);
            w.player.y = y.clamp(EDGE_TOP, FIELD_H as f32 - EDGE_BOTTOM);
            w.tick(Controls::default());
            w.events.clear();
            let b = &w.hazards.bullets;
            for i in 0..b.len() {
                if b.age[i] == 0 {
                    let d = (b.x[i] - w.player.x).hypot(b.y[i] - w.player.y);
                    assert!(d >= SAFE_SPAWN, "{}: a bullet appeared {d:.1} px from the ship", SANDBOX[p].name);
                }
            }
        }
    }
}

/// The widest free gap across one horizontal band of the playfield.
pub(super) fn widest_gap(w: &World, y0: f32, y1: f32) -> f32 {
    widest_gap_drawn(w, y0, y1, 1.0)
}

/// The same, with every bullet's radius grown `look` times (as it's drawn).
pub(super) fn widest_gap_drawn(w: &World, y0: f32, y1: f32, look: f32) -> f32 {
    let b = &w.hazards.bullets;
    let extent = |i: usize| b.kind[i].radius() * look + b.kind[i].half_len();
    let mut spans: Vec<(f32, f32)> = (0..b.len())
        .filter(|&i| b.y[i] + extent(i) > y0 && b.y[i] - extent(i) < y1)
        .map(|i| (b.x[i] - extent(i), b.x[i] + extent(i)))
        .collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (mut best, mut edge) = (0.0f32, 0.0f32);
    for (a, z) in spans {
        best = best.max(a - edge);
        edge = edge.max(z);
    }
    best.max(FIELD_W as f32 - edge)
}

#[test]
fn every_band_in_the_bottom_half_keeps_a_12_px_gap() {
    for p in patterns() {
        for seed in [1, 2] {
            let mut w = world(p, seed);
            w.invincible = true; // let the pattern run uninterrupted
            for f in 0..2400 {
                w.tick(Controls::default());
                w.events.clear();
                if f < 300 {
                    continue;
                }
                let mut y = FIELD_H as f32 / 2.0;
                while y < FIELD_H as f32 {
                    let gap = widest_gap(&w, y, y + 4.0);
                    assert!(gap >= 12.0, "{} builds a wall at y={y} (widest gap {gap:.1} px, frame {f})", SANDBOX[p].name);
                    y += 4.0;
                }
            }
        }
    }
}

#[test]
fn the_wall_check_would_catch_a_wall() {
    let mut w = World::new(1, Default::default());
    for k in 0..=FIELD_W / 10 {
        w.hazards.bullets.push(Spawn { x: 5.0 + k as f32 * 10.0, y: 250.0, vx: 0.0, vy: 0.0, kind: Kind::Orb, behavior: Behavior::Straight, owner: 0, hue: Default::default() });
    }
    assert!(widest_gap(&w, 248.0, 252.0) < 12.0);
    assert!(widest_gap(&w, 300.0, 304.0) >= FIELD_W as f32);
}

fn parked_bullet(w: &mut World, dx: f32, kind: Kind, age: u16) {
    let (x, y) = (w.player.x + dx, w.player.y);
    w.hazards.bullets.push(Spawn { x, y, vx: 0.0, vy: 0.0, kind, behavior: Behavior::Straight, owner: 0, hue: Default::default() });
    let i = w.hazards.bullets.len() - 1;
    w.hazards.bullets.age[i] = age;
}

#[test]
fn new_bullets_bloom_harmlessly_for_four_frames() {
    let mut w = World::new(1, Default::default());
    w.dummies.clear();
    parked_bullet(&mut w, 0.0, Kind::Orb, 0);
    // ages 1, 2, 3 after these ticks (age 0 was its first frame)
    for _ in 0..3 {
        w.tick(Controls::default());
        assert_eq!(w.hits, 0);
    }
    w.tick(Controls::default());
    assert_eq!(w.hits, 1, "live from its fifth frame");
}

#[test]
fn a_graze_counts_once_per_bullet() {
    let mut w = World::new(1, Default::default());
    w.dummies.clear();
    parked_bullet(&mut w, 10.0, Kind::Small, 20);
    for _ in 0..30 {
        w.tick(Controls::default());
    }
    assert_eq!(w.grazes, 1);
    assert_eq!(w.hits, 0, "close isn't touching");
}

#[test]
fn invincibility_and_stress_scenes_never_kill() {
    let mut w = World::new(1, Default::default());
    w.invincible = true;
    parked_bullet(&mut w, 0.0, Kind::Large, 30);
    w.tick(Controls::default());
    assert_eq!(w.hits, 0);
    let mut w = world(SANDBOX.iter().position(|p| p.fill > 0).unwrap(), 1);
    for _ in 0..300 {
        w.tick(Controls::default());
    }
    assert_eq!(w.hits, 0);
}

#[test]
fn standing_still_gets_you_hit() {
    // the patterns are real threats, so the bot's survival means something
    let mut hit = 0;
    for p in patterns() {
        let mut w = world(p, 5);
        for _ in 0..3600 {
            w.tick(Controls::default());
            w.events.clear();
        }
        hit += (w.hits > 0) as usize;
    }
    assert!(hit * 3 >= patterns().len() * 2, "only {hit} of {} patterns ever hit a sitting target", patterns().len());
}

#[test]
fn the_bot_survives_each_pattern_for_60_seconds_on_most_seeds() {
    let seeds = 6u64;
    let ps = patterns();
    let results: Vec<(usize, u32)> = std::thread::scope(|s| {
        let handles: Vec<_> = ps
            .iter()
            .map(|&p| s.spawn(move || (p, (0..seeds).filter(|&k| bot::trial(p, 100 + k, 60 * TICK_HZ, false).0 == 0).count() as u32)))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for (p, survived) in results {
        assert!(survived as u64 * 4 >= seeds * 3, "{} survived only {survived} of {seeds} seeds", SANDBOX[p].name);
    }
}
