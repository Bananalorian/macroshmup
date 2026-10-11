//! Stage 1's checks: the dead zone and the ceasefire, crashing into enemies, heavies
//! taking their bullets with them, the opening seconds, the timeline, and the bot
//! clearing the stage.

use super::bot;
use super::enemy::{EnemyKind, Path};
use super::player::Controls;
use super::stage1::{self, frames};
use super::{Event, Life, Target, World};
use crate::tune::*;

fn fire() -> Controls {
    Controls { fire: true, ..Default::default() }
}

fn beam() -> Controls {
    Controls { fire: true, focus: true, ..Default::default() }
}

/// A stage world with nothing scheduled: just the enemies a test adds.
fn empty() -> World {
    let mut w = World::stage(1, Default::default());
    w.stage.as_mut().unwrap().skip_to(frames(SECTIONS[6].0));
    w
}

fn parked(x: f32, y: f32) -> Path {
    Path::Hover { from: (x, y), at: (x, y), enter: 1, stay: 100_000, sway: 0.0 }
}

#[test]
fn no_enemy_takes_damage_in_the_top_dead_zone() {
    for c in [fire(), beam()] {
        let mut w = empty();
        w.invincible = true;
        w.foes.add(EnemyKind::Heavy, 0, parked(START_X, DEAD_ZONE - 1.0));
        for _ in 0..300 {
            w.tick(c);
            w.events.clear();
        }
        let e = &w.foes.list[0];
        assert_eq!(e.parts[0].hp, e.parts[0].max_hp, "shots and the beam pass it by");
        assert!(w.beam.is_none_or(|b| b.top == 0.0 && b.hit.is_none()));
        // just below the line, it's fair game
        let mut w = empty();
        w.invincible = true;
        w.foes.add(EnemyKind::Heavy, 0, parked(START_X, DEAD_ZONE + 1.0));
        for _ in 0..60 {
            w.tick(c);
        }
        assert!(w.foes.list[0].parts[0].hp < w.foes.list[0].parts[0].max_hp);
    }
}

#[test]
fn nothing_fires_from_the_ceasefire_zone() {
    let mut w = empty();
    w.invincible = true;
    let y = FIELD_H as f32 - CEASEFIRE + 2.0;
    w.foes.add(EnemyKind::Heavy, 0, parked(60.0, y));
    w.player.x = 220.0;
    for _ in 0..600 {
        w.tick(Controls::default());
    }
    assert_eq!(w.hazards.bullets.len(), 0);
}

#[test]
fn crashing_into_an_enemy_is_a_hit() {
    let mut w = empty();
    let (x, y) = (w.player.x, w.player.y);
    w.foes.add(EnemyKind::Drone, 0, parked(x, y - 2.0));
    w.tick(Controls::default());
    assert!(matches!(w.life, Life::Struck { .. }));
    // but not while invincible after a bomb
    let mut w = empty();
    w.foes.add(EnemyKind::Drone, 0, parked(x, y - 2.0));
    w.tick(Controls { bomb: true, ..Default::default() });
    for _ in 0..30 {
        w.tick(Controls::default());
    }
    assert_eq!(w.hits, 0);
}

#[test]
fn killing_a_heavy_cancels_its_bullets_into_shards() {
    let mut w = empty();
    w.invincible = true;
    w.player.x = 30.0;
    let id = w.foes.add(EnemyKind::Heavy, 0, parked(150.0, 80.0));
    w.foes.add(EnemyKind::Gunship, 0, parked(220.0, 60.0));
    for _ in 0..240 {
        w.tick(Controls::default());
        w.events.clear();
    }
    let b = &w.hazards.bullets;
    let theirs = (0..b.len()).filter(|&i| b.owner[i] == id).count();
    let others = b.len() - theirs;
    assert!(theirs > 10 && others > 0, "{theirs} {others}");
    let i = w.foes.list.iter().position(|e| e.id == id).unwrap();
    w.build_boxes();
    w.hurt(Target::Foe(i, 0), HEAVY.0);
    assert!(w.events.iter().any(|e| matches!(e, Event::Kill { kind: EnemyKind::Heavy, .. })));
    assert!(w.cancel_shards && w.cancelled.len() == theirs, "all of its bullets became shards");
    assert_eq!(w.hazards.bullets.len(), others, "everyone else's stay");
}

#[test]
fn shots_break_the_weak_points_before_the_hull() {
    let mut w = empty();
    w.invincible = true;
    let (tx, ty) = HULL_TURRETS[0];
    w.foes.add(EnemyKind::Hull, 0, parked(START_X - tx, 100.0));
    w.foes.list[0].t = 10; // risen out of the disk
    for _ in 0..30 {
        w.tick(fire());
    }
    let e = &w.foes.list[0];
    let (turret, hull) = (e.parts[1].max_hp - e.parts[1].hp, e.parts[0].max_hp - e.parts[0].hp);
    assert!(turret > 5.0 * hull, "the turret over the ship took the hits ({turret}; the hull {hull}, from the angled pod shots)");
    assert!(ty + HULL_TURRET.2 * 0.5 > HULL.2 * 0.5, "it hangs below the hull");
}

#[test]
fn the_derelict_cant_be_hurt_while_it_rises_and_a_bomb_only_dents_it() {
    let mut w = World::stage_at(1, Default::default(), SECTIONS[4].0);
    w.invincible = true;
    for _ in 0..HULL_EMERGE - 10 {
        w.tick(beam());
        w.events.clear();
    }
    let hull = w.foes.list.iter().find(|e| e.kind == EnemyKind::Hull).unwrap();
    assert_eq!(hull.parts[0].hp, hull.parts[0].max_hp, "rising out of the disk");
    for _ in 0..30 {
        w.tick(Controls::default());
    }
    w.tick(Controls { bomb: true, ..Default::default() });
    for _ in 0..BOMB_ACTIVE {
        w.tick(Controls::default());
    }
    let hull = w.foes.list.iter().find(|e| e.kind == EnemyKind::Hull).unwrap();
    let lost = hull.parts[0].max_hp - hull.parts[0].hp;
    assert!((lost - BOMB_DAMAGE.min(HULL.0 * BOMB_BOSS_CAP)).abs() < 1.0, "a bomb hurts it once, and only so much: {lost}");
    assert!(lost <= HULL.0 * BOMB_BOSS_CAP);
}

#[test]
fn no_bullets_in_the_opening_seconds() {
    let mut w = World::stage(5, Default::default());
    w.invincible = true;
    let mut b = bot::Bot::new();
    for _ in 0..frames(FIRST_FIRE) {
        let c = b.decide(&w);
        w.tick(c);
        assert_eq!(w.hazards.bullets.len(), 0, "frame {}", w.frame);
    }
    assert!(!w.foes.list.is_empty(), "but there's something to shoot");
}

#[test]
fn pg_scene_can_start_anywhere_in_the_stage() {
    let w = World::stage_at(1, Default::default(), SECTIONS[4].0);
    assert_eq!(w.clock(), frames(SECTIONS[4].0) as u64 - 1);
    let mut w = w;
    w.tick(Controls::default());
    assert_eq!(w.stage.as_ref().unwrap().section, 4);
    assert!(w.foes.list.iter().any(|e| e.kind == EnemyKind::Hull), "the derelict arrives on its cue");
    w.skip_section();
    w.tick(Controls::default());
    assert_eq!(w.stage.as_ref().unwrap().section, 5);
    assert!(w.foes.list.iter().all(|e| e.kind != EnemyKind::Hull), "F6 clears the screen");
}

#[test]
fn a_stage_run_is_deterministic() {
    let run = || {
        let r = bot::stage_trial(9, true, true);
        (r.deaths, r.bombs, r.killed, r.grazes, r.peak, r.began)
    };
    assert_eq!(run(), run());
}

/// The sprint's bar is 90% of 200 seeds (PG_SIM=200); CI plays a handful.
#[test]
fn the_bot_clears_the_stage_and_every_section_lands_on_time() {
    let seeds: Vec<u64> = (0..8).collect();
    let runs: Vec<bot::StageRun> = std::thread::scope(|s| {
        let handles: Vec<_> = seeds.iter().map(|&k| s.spawn(move || bot::stage_trial(100 + k, true, true))).collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let cleared = runs.iter().filter(|r| r.cleared).count();
    assert!(cleared >= 7, "cleared {cleared} of {}", runs.len());
    for r in runs.iter().filter(|r| r.cleared) {
        for (i, s) in SECTIONS.iter().enumerate() {
            let at = r.began[i].expect("every section happens");
            let off = (at as i64 - stage1::frames(s.0) as i64).abs();
            assert!(off <= frames(5.0) as i64, "{} began {:.1} s off", s.1, off as f32 / 60.0);
        }
        assert!(r.killed > r.spawned / 2, "it shoots things: {} of {}", r.killed, r.spawned);
    }
}

