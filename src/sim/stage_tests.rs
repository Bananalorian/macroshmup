//! Stage 1's checks: the dead zone and the ceasefire, crashing into enemies, heavies
//! taking their bullets with them, the Swarm (lancers, remoras, scenery), the opening
//! seconds, the timeline, and the bot clearing the stage.

use super::bot;
use super::enemy::{EnemyKind, Path};
use super::player::Controls;
use super::score::Award;
use super::stage1::{self, frames};
use super::{Event, Life, Target, World};
use crate::tune::*;

fn fire() -> Controls {
    Controls { fire: true, ..Default::default() }
}

fn beam() -> Controls {
    Controls { fire: true, focus: true, ..Default::default() }
}

/// A stage world with nothing scheduled: just the enemies a test adds (and no boss).
fn empty() -> World {
    let mut w = World::stage(1, Default::default());
    let s = w.stage.as_mut().unwrap();
    s.skip_to(frames(SECTIONS[WARNING_SECTION].0));
    s.boss_called = true;
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

/// A lancer flies in, holds still while it takes aim, locks on to where you are when
/// its aim runs out, and rushes there in a straight line: dodge after it locks and it
/// misses. It never fires.
#[test]
fn a_lancer_takes_aim_locks_on_and_rushes_where_you_were() {
    let mut w = empty();
    w.invincible = true;
    let (enter, wait, speed) = LANCER_RUSH;
    w.foes.add(EnemyKind::Lancer, 0, Path::Rush { from: (100.0, -16.0), at: (100.0, 90.0), enter, wait, speed, dir: (0.0, 1.0) });
    w.player.x = 220.0;
    for _ in 0..enter + 1 {
        w.tick(Controls::default());
    }
    assert_eq!((w.foes.list[0].x, w.foes.list[0].y), (100.0, 90.0), "in place");
    for _ in enter + 1..enter + wait - 1 {
        w.tick(Controls::default());
        let e = &w.foes.list[0];
        assert_eq!((e.x, e.y), (100.0, 90.0), "it holds still while it aims");
        assert!(e.path.aiming(e.t).is_some());
    }
    let me = (w.player.x, w.player.y);
    w.tick(Controls::default());
    assert!(w.foes.list[0].path.aiming(w.foes.list[0].t).is_none(), "locked on");
    // you move: it doesn't follow
    w.player.x = 40.0;
    for _ in 0..10 {
        w.tick(Controls::default());
    }
    let e = &w.foes.list[0];
    let (mx, my) = (e.x - 100.0, e.y - 90.0);
    let (tx, ty) = (me.0 - 100.0, me.1 - 90.0);
    let cross = (mx * ty - my * tx) / tx.hypot(ty);
    assert!(cross.abs() < 0.5, "straight at where you were: {mx},{my} vs {tx},{ty}");
    assert!((mx.hypot(my) - speed * 10.0).abs() < 0.5, "at its speed");
    assert_eq!(w.hazards.bullets.len(), 0, "it never fires");
}

/// Scenery is harmless: it never fires and touching it does nothing. Shot down, it
/// breaks into shards that fly to you, and it isn't counted among the stage's kills;
/// the slam never picks it.
#[test]
fn scenery_is_harmless_and_breaks_into_shards() {
    for (k, kind) in [EnemyKind::Debris, EnemyKind::Satellite, EnemyKind::Monolith].into_iter().enumerate() {
        let mut w = empty();
        let (x, y) = (w.player.x, w.player.y);
        w.foes.add(kind, 0, parked(x, y - 2.0));
        for _ in 0..120 {
            w.tick(Controls::default());
        }
        assert_eq!(w.hits, 0, "{kind:?}: touching it is harmless");
        assert!(matches!(w.life, Life::Alive), "{kind:?}");
        assert_eq!(w.hazards.bullets.len(), 0, "{kind:?}: it never fires");
        assert!(w.slam_target((x, y), (0.0, 0.0), None).is_none(), "{kind:?}: nothing worth a slam");
        let killed = w.stage.as_ref().unwrap().killed;
        w.events.clear();
        w.build_boxes();
        w.hurt(Target::Foe(0, 0), kind.stats().0);
        assert!(w.events.iter().any(|e| matches!(e, Event::Kill { kind: k, .. } if *k == kind)));
        let n = WRECK_SHARDS[k];
        assert!(w.events.iter().any(|e| matches!(e, Event::Points { n: p, how: Award::Shards, .. } if *p == n * SHARD_POINTS)), "{kind:?}: its shards pay");
        assert!(w.cancel_shards && w.cancelled.len() == n as usize, "{kind:?}: {n} shards fly to you");
        assert_eq!(w.stage.as_ref().unwrap().killed, killed, "{kind:?}: not counted among the kills");
    }
}

/// A school of remoras crosses the screen in the upper half and swims out the far
/// side; in a firing school, every other one fires a single aimed shot.
#[test]
fn a_school_of_remoras_crosses_and_every_other_one_fires_once() {
    let mut w = empty();
    w.invincible = true;
    w.player.x = 30.0;
    let school: Vec<_> = stage1::arrivals().into_iter().filter(|a| a.kind == EnemyKind::Remora && a.variant == 1).take(1).collect();
    let first = school[0].frame;
    let all: Vec<_> = stage1::arrivals().into_iter().filter(|a| a.kind == EnemyKind::Remora && a.frame >= first && a.frame < first + 120).collect();
    assert!(all.len() >= 8);
    assert_eq!(all.iter().filter(|a| a.variant == 1).count(), all.len().div_ceil(2), "every other one fires");
    for a in &all {
        w.foes.add(a.kind, a.variant, a.path);
    }
    let mut owners = std::collections::HashSet::new();
    for t in 0..400 {
        w.tick(Controls::default());
        w.events.clear();
        let b = &w.hazards.bullets;
        for i in 0..b.len() {
            assert_eq!((0..b.len()).filter(|&j| b.owner[j] == b.owner[i]).count(), 1, "one shot each");
            owners.insert(b.owner[i]);
        }
        for e in &w.foes.list {
            assert!(e.y > 0.0 && e.y < FIELD_H as f32 * 0.5 + 30.0, "frame {t}: in the upper half");
        }
    }
    assert!(w.foes.list.is_empty(), "all swam out the far side");
    let firing = all.iter().filter(|a| a.variant == 1).count();
    assert!(!owners.is_empty() && owners.len() <= firing, "{} fired, of {firing} firing remoras", owners.len());
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
    let (tx, ty, _, th) = HULL_TURRETS[0];
    w.foes.add(EnemyKind::Hull, 0, parked(START_X - tx, 100.0));
    w.foes.list[0].t = 10; // risen out of the disk
    for _ in 0..30 {
        w.tick(fire());
    }
    let e = &w.foes.list[0];
    let (turret, hull) = (e.parts[1].max_hp - e.parts[1].hp, e.parts[0].max_hp - e.parts[0].hp);
    assert!(turret > 5.0 * hull, "the turret over the ship took the hits ({turret}; the hull {hull}, from the angled pod shots)");
    assert!(HULL_TURRETS.iter().all(|t| t.1 + t.3 * 0.5 > HULL.2 * 0.5), "every weak point hangs below the hull");
    assert!(ty + th * 0.5 > HULL.2 * 0.5);
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


