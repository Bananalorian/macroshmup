//! The boss's checks, Sprint 2's fairness rules applied to each of its phases: the
//! same seed gives the same fight, no bullet appears on top of you, no walls in the
//! bottom half, and the bot gets through each phase without bombs. Then the sprint's
//! bars: each phase takes 20-35 s to break, the death cancels everything and slows
//! time, and the stage ends with the boss.

use super::boss::{Act, Muzzle, Outcome};
use super::bot;
use super::fairness::widest_gap;
use super::player::Controls;
use super::{Event, Life, World};
use crate::tune::*;

/// Phase `p` running, the bot playing (and unhittable, if `safe`).
fn phase(p: u8, seed: u64, safe: bool) -> World {
    let mut w = World::boss_at(seed, Default::default(), p);
    w.invincible = safe;
    w.lives = u8::MAX;
    w
}

#[test]
fn the_same_seed_gives_the_same_fight() {
    for p in 1..=4 {
        let run = || {
            let mut w = phase(p, 42, false);
            let mut bot = bot::Bot::new();
            for _ in 0..900 {
                let c = bot.decide(&w);
                w.tick(c);
                w.events.clear();
            }
            let b = &w.hazards.bullets;
            let boss = w.boss.as_ref().unwrap();
            (b.x.clone(), b.y.clone(), boss.x, boss.y, boss.hp, w.player.x, w.player.y)
        };
        let (a, b) = (run(), run());
        assert!(!a.0.is_empty(), "phase {p} fired nothing");
        assert!(a == b, "phase {p} differs between runs");
    }
}

#[test]
fn no_boss_bullet_spawns_within_24_px_of_the_player() {
    for p in 1..=4 {
        let mut w = phase(p, 7, true);
        let muzzles = [Muzzle::Mouth, Muzzle::Window(0), Muzzle::Window(3), Muzzle::Pectoral, Muzzle::Tail, Muzzle::Belly];
        for f in 0..1800u32 {
            // park right under each of its guns in turn
            let m = muzzles[(f / 150) as usize % muzzles.len()];
            let (x, y) = w.boss.as_ref().unwrap().muzzle(m);
            w.player.x = x.clamp(EDGE_X, FIELD_W as f32 - EDGE_X);
            w.player.y = (y + 26.0).clamp(EDGE_TOP, FIELD_H as f32 - EDGE_BOTTOM);
            w.tick(Controls::default());
            w.events.clear();
            let b = &w.hazards.bullets;
            for i in 0..b.len() {
                if b.age[i] == 0 {
                    let d = (b.x[i] - w.player.x).hypot(b.y[i] - w.player.y);
                    assert!(d >= SAFE_SPAWN, "phase {p}: a bullet appeared {d:.1} px from the ship");
                }
            }
        }
    }
}

#[test]
fn every_band_in_the_bottom_half_keeps_a_12_px_gap_in_every_phase() {
    for p in 1..=4 {
        for seed in [1, 2] {
            let mut w = phase(p, seed, true);
            let mut bot = bot::Bot::new();
            for f in 0..JAWS_PHASES[p as usize - 1].1 * TICK_HZ {
                let c = bot.decide(&w);
                w.tick(Controls { fire: false, ..c });
                w.events.clear();
                if f < 240 {
                    continue;
                }
                let mut y = FIELD_H as f32 / 2.0;
                while y < FIELD_H as f32 {
                    let gap = widest_gap(&w, y, y + 4.0);
                    assert!(gap >= 12.0, "phase {p} builds a wall at y={y} (widest gap {gap:.1} px, frame {f})");
                    y += 4.0;
                }
            }
        }
    }
}

/// The sprint's bars, on a handful of seeds here (PG_SIM=boss:200 runs the full set):
/// the bot gets through each phase without bombs on at least 80% of seeds, and each
/// phase takes 20-35 s to break with full power.
#[test]
fn the_bot_gets_through_each_phase_without_bombs_in_20_to_35_seconds() {
    let seeds = 6u64;
    let runs: Vec<(u8, bot::PhaseRun)> = std::thread::scope(|s| {
        let handles: Vec<_> = (1..=4u8).flat_map(|p| (0..seeds).map(move |k| (p, k))).map(|(p, k)| s.spawn(move || (p, bot::phase_trial(500 + k, p, false)))).collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for p in 1..=4u8 {
        let mine: Vec<&bot::PhaseRun> = runs.iter().filter(|r| r.0 == p).map(|r| &r.1).collect();
        let clean = mine.iter().filter(|r| r.deaths == 0).count() as u64;
        assert!(clean * 5 >= seeds * 4, "phase {p}: survived {clean} of {seeds}");
        let mut kills: Vec<u32> = mine.iter().filter_map(|r| r.kill).collect();
        kills.sort();
        assert!(kills.len() as u64 * 5 >= seeds * 4, "phase {p}: broken in only {} of {seeds}", kills.len());
        let median = kills[kills.len() / 2] as f32 / TICK_HZ as f32;
        assert!((20.0..=35.0).contains(&median), "phase {p} takes {median:.1} s");
    }
}

#[test]
fn its_laser_and_its_body_are_as_deadly_as_bullets() {
    // stand on the beam
    let mut w = phase(3, 3, false);
    let mut struck_by_beam = false;
    for _ in 0..600 {
        let laser = w.boss.as_ref().unwrap().laser.filter(|l| l.firing());
        if let Some(l) = laser {
            let (s, c) = l.angle().to_radians().sin_cos();
            w.player.x = l.x + c * 80.0;
            w.player.y = l.y + s * 80.0;
            w.hazards.bullets.clear();
            w.tick(Controls::default());
            struck_by_beam = matches!(w.life, Life::Struck { .. });
            break;
        }
        w.tick(Controls::default());
        w.events.clear();
        w.invuln = 0;
        w.life = Life::Alive;
    }
    assert!(struck_by_beam, "the beam hurts");
    // fly into its belly
    let mut w = phase(1, 3, false);
    let r = w.boss.as_ref().unwrap().rect(0);
    w.player.x = (r.x0 + r.x1) * 0.5;
    w.player.y = (r.y0 + r.y1) * 0.5;
    w.tick(Controls::default());
    assert!(matches!(w.life, Life::Struck { .. }), "crashing into it hurts");
}

#[test]
fn its_death_cancels_every_bullet_runs_slow_and_ends_the_stage() {
    let mut w = phase(4, 5, true);
    for _ in 0..240 {
        w.tick(Controls::default());
        w.events.clear();
    }
    assert!(w.hazards.bullets.len() > 20);
    w.skip_section();
    assert!(w.events.iter().any(|e| matches!(e, Event::BossDown { .. })));
    assert_eq!(w.hazards.bullets.len(), 0, "a full-screen cancel");
    assert!(w.cancel_shards && !w.cancelled.is_empty(), "into shards");
    assert_eq!(w.slow, JAWS_SLOWMO, "quarter speed from here");
    let mut ended = false;
    for _ in 0..JAWS_DEATH + 5 {
        w.tick(Controls::default());
        ended |= w.events.contains(&Event::StageEnd);
        w.events.clear();
    }
    assert!(ended && w.stage.as_ref().unwrap().over);
    let b = w.boss.as_ref().unwrap();
    assert_eq!(b.act, Act::Gone);
    assert_eq!(b.outcomes[3], Some(Outcome::Broken));
}

#[test]
fn the_boss_rises_after_the_warning() {
    let mut w = World::stage_at(9, Default::default(), SECTIONS[WARNING_SECTION].0);
    w.invincible = true;
    let mut warned = false;
    let mut arrived_at = None;
    for f in 0..frames_for(JAWS_AT + 7.0 - SECTIONS[WARNING_SECTION].0) {
        w.tick(Controls::default());
        warned |= w.events.contains(&Event::Warning);
        if w.events.contains(&Event::BossArrive) {
            assert!(warned, "the WARNING first");
            arrived_at = Some(f);
        }
        w.events.clear();
    }
    assert!(arrived_at.is_some(), "it came");
    let b = w.boss.as_ref().unwrap();
    assert_eq!((b.act, b.phase), (Act::Fight, 1), "and it's fighting once it's risen");
}

fn frames_for(secs: f32) -> u32 {
    (secs * TICK_HZ as f32) as u32
}
