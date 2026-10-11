//! Sheets for review, native only: the ship's animation sheet and the enemy sheet
//! (PG_SHEET=1 or F9), laid out like Jason's reference sheet, and the effects sheet (PG_FXSHEET=1):
//! every explosion size at 4 moments, and the black hole at the start and end of
//! the stage.

use crate::render::art::Atlas;
use crate::render::explode::{Size, explode};
use crate::render::font;
use crate::render::hud::text;
use crate::render::layout::{IRect, Layout, Orient};
use crate::render::palette::{ASH, CREAM, DUSK, SIGNAL, SLATE, VOID};
use crate::render::particles::Particles;
use crate::render::post::Fx;
use crate::render::scene;
use crate::render::ship::{self, Pose};
use crate::render::{PostParams, Renderer};
use crate::sim::World;
use crate::tune::{BUMP_FLASH, HOLE_X, HOLE_Y, STAGE_FRAMES};
use macroquad::prelude::*;

const SHEET: (f32, f32) = (440.0, 344.0);
const SHEET_SCALE: u32 = 3;

/// Draw the ship's animation sheet, laid out like Jason's reference sheets.
fn draw_sheet(a: &Atlas) {
    let label = |s: &str, x: i32, y: i32| text(a, s, x, y, 1, 2, CREAM);
    text(a, "PROTAGONIST SHIP", 10, 8, 1, 2, CREAM);
    text(a, "TYPE-07  \"PILGRIM\"  //  SPLIT SWORD  //  BUILT IN CODE", 10, 18, 1, 2, ASH);
    a.fill(222, 36, 1, SHEET.1 as i32 - 50, DUSK);
    let base = Pose::default();
    let laser = Pose { form: 1.0, ..base };
    let row = |y: i32| y + 30;
    let col = |x0: i32, i: i32| x0 + 24 + i * 50;

    label("IDLE (4 FRAMES)", 10, 38);
    for i in 0..4 {
        ship::draw(a, col(10, i), row(38), &Pose { frame: i as u64 * 23 + 4, ..base });
    }
    label("THRUST (4 FRAMES)", 10, 110);
    for i in 0..4 {
        ship::draw(a, col(10, i), row(110), &Pose { thrust: true, frame: i as u64 * 3, ..base });
    }
    label("FIRE (4 FRAMES)", 10, 182);
    for i in 0..4 {
        ship::draw(a, col(10, i), row(182) + 4, &Pose { fire_t: i as u32 + 1, frame: i as u64 * 2, ..base });
    }
    label("HIT FLASH (4 FRAMES)", 10, 268);
    for i in 0..4 {
        let hit_t = BUMP_FLASH - (i as u32 * BUMP_FLASH / 4);
        ship::draw(a, col(10, i), row(268), &Pose { hit_t, frame: 4, ..base });
    }

    label("BANK (L2 L1 R1 R2)", 232, 38);
    for (i, b) in [-2i8, -1, 1, 2].into_iter().enumerate() {
        ship::draw(a, 232 + 20 + i as i32 * 46, row(38), &Pose { bank: b, frame: 4, ..base });
    }
    label("FLIGHT TO LASER (5 FRAMES)", 232, 110);
    for i in 0..ship::FORMS {
        let form = i as f32 / (ship::FORMS - 1) as f32;
        let locked = if i == ship::FORMS - 1 { 3 } else { 0 };
        ship::draw(a, 232 + 16 + i as i32 * 40, row(110), &Pose { form, locked, frame: 4, ..base });
    }
    label("LASER FORM  //  BEAM", 232, 182);
    let (fx, fy) = (232 + 20, row(182) + 14);
    ship::draw(a, fx, fy, &Pose { frame: 4, ..laser });
    ship::draw_hitbox(a, fx, fy, 1.0, 0);
    let (bx, by) = (232 + 66, row(182) + 14);
    scene::draw_beam(a, 0, 0, bx as f32, 191.0, (by + ship::TIP) as f32, 6);
    ship::draw(a, bx, by, &Pose { beaming: true, fire_t: 2, frame: 6, ..laser });
    ship::draw_hitbox(a, bx, by, 1.0, 9);
    let (kx, ky) = (232 + 112, row(182) + 14);
    ship::draw(a, kx, ky, &Pose { bank: 2, frame: 4, ..laser });
    let shot = Color::new(1.0, 1.0, 1.0, 0.6);
    label("SHOTS  //  AFTERIMAGES", 232, 268);
    for i in 0..4 {
        a.blit(a.r.ship.bolt[i], 232 + 8 + i as i32 * 10, 282, 1, shot);
    }
    for i in 0..2 {
        a.blit(a.r.ship.pod_shot[i], 232 + 50 + i as i32 * 6, 284, 1, Color::new(1.0, 1.0, 1.0, 0.45));
    }
    let (ax, ay) = (232 + 150, row(268));
    ship::draw_afterimages(a, 0.0, 2, &[(ax - 9, ay + 3), (ax - 18, ay + 6), (ax - 27, ay + 9)]);
    ship::draw(a, ax, ay, &Pose { bank: 2, frame: 4, ..base });
    a.fill(10, SHEET.1 as i32 - 13, SHEET.0 as i32 - 20, 1, DUSK);
    text(a, &crate::render::hud::version_stamp(), 10, SHEET.1 as i32 - 9, 1, 2, SLATE);
}

/// Stage 1's enemies: every sprite, the derelict whole, cracked and stripped, and the
/// breather's background extras.
fn draw_enemy_sheet(a: &Atlas) {
    use crate::render::palette::WHITE_A;
    let f = &a.r.foes;
    let label = |s: &str, x: i32, y: i32| text(a, s, x, y, 1, 2, CREAM);
    text(a, "STAGE 1 ENEMIES  //  LUNAR GOTHIC", 10, 8, 1, 2, CREAM);
    text(a, "OUTER DISK  //  BONE WHITE, BUILT IN CODE", 10, 18, 1, 2, ASH);
    let at = |r: Rect, cx: i32, cy: i32| a.blit(r, cx - (r.w * 0.5) as i32, cy - (r.h * 0.5) as i32, 1, WHITE_A);
    label("FODDER", 10, 34);
    at(f.drone.r, 20, 62);
    at(f.courier.r, 44, 62);
    at(f.usher.r, 76, 62);
    label("GUNSHIP", 104, 34);
    at(f.gunship.r, 126, 62);
    label("BUOY (4 FRAMES)", 164, 34);
    for i in 0..4 {
        at(f.buoy[i].r, 176 + i as i32 * 26, 62);
    }
    label("HEAVY", 300, 34);
    at(f.heavy.r, 330, 64);
    label("DERELICT  //  WHOLE, CRACKED, STRIPPED", 10, 90);
    for phase in 0..3 {
        let (cx, cy) = (72 + phase * 138, 156);
        at(f.hull.r, cx, cy);
        if phase >= 1 {
            for &(dx, dy) in &f.cracks {
                a.fill(cx + dx as i32, cy + dy as i32, 1, 1, SIGNAL);
            }
            a.blit(a.r.discs[4], cx - 6, cy - 2 - 6, 1, Color { a: 0.85, ..SIGNAL });
        }
        for (i, &(dx, dy, _, th)) in crate::tune::HULL_TURRETS.iter().enumerate() {
            let (tx, ty) = (cx + dx as i32, cy + dy as i32);
            if phase == 2 {
                a.blit(f.socket, tx - 6, ty - (th * 0.5) as i32 - 2, 1, WHITE_A);
            } else {
                at(if i == 1 { f.nose.r } else { f.pod.r }, tx, ty);
                a.fill(tx - 1, ty, 3, 3, Color { a: 0.6, ..SIGNAL });
            }
        }
    }
    label("WEAK POINTS, HIT FLASH", 10, 232);
    at(f.pod.r, 22, 270);
    at(f.nose.r, 44, 270);
    at(f.drone.r, 70, 270);
    a.blit(f.drone.sil, 70 - (f.drone.sil.w * 0.5) as i32, 270 - (f.drone.sil.h * 0.5) as i32, 1, Color { a: 0.55, ..SIGNAL });
    at(f.gunship.r, 100, 270);
    a.blit(f.gunship.sil, 100 - (f.gunship.sil.w * 0.5) as i32, 270 - (f.gunship.sil.h * 0.5) as i32, 1, Color { a: 0.55, ..SIGNAL });
    label("BREATHER EXTRAS", 196, 232);
    for i in 0..6 {
        at(f.debris[i], 192 + i as i32 * 16, 270);
    }
    at(f.figure, 300, 270);
    a.blit(f.figure, 318, 259, 2, WHITE_A);
    a.fill(10, SHEET.1 as i32 - 13, SHEET.0 as i32 - 20, 1, DUSK);
    text(a, &crate::render::hud::version_stamp(), 10, SHEET.1 as i32 - 9, 1, 2, SLATE);
}

/// Render the sheets off-screen and save them as `./shots/pilgrim_sheet.png` and
/// `./shots/enemy_sheet.png`, 3x.
pub fn export(a: &Atlas) {
    save_sheet(a, draw_sheet, "shots/pilgrim_sheet.png");
    save_sheet(a, draw_enemy_sheet, "shots/enemy_sheet.png");
}

fn save_sheet(a: &Atlas, draw: fn(&Atlas), path: &str) {
    let rt = render_target(SHEET.0 as u32, SHEET.1 as u32);
    rt.texture.set_filter(FilterMode::Nearest);
    set_camera(&crate::render::canvas_camera(&rt, SHEET.0, SHEET.1));
    clear_background(VOID);
    draw(a);
    unsafe { get_internal_gl().flush() };
    let img = rt.texture.get_texture_data();
    set_default_camera();
    let (w, h) = (img.width as u32, img.height as u32);
    let s = SHEET_SCALE;
    let mut big = Image::gen_image_color((w * s) as u16, (h * s) as u16, VOID);
    for y in 0..h * s {
        for x in 0..w * s {
            // render targets read back bottom row first
            big.set_pixel(x, y, img.get_pixel(x / s, h - 1 - y / s));
        }
    }
    let _ = std::fs::create_dir_all("shots");
    big.export_png(path);
    println!("saved {path}");
}


// ---------------------------------------------------------------- the boss's sheet (PG_MOCKUP=1)

/// A run of the boss fight for the sheet: phase `phase` from its start, played by
/// the bot for `frames` frames, then `tweak`ed into the state to show.
fn boss_world(phase: u8, frames: u32, tweak: impl Fn(&mut World)) -> World {
    let mut w = World::boss_at(7, Default::default(), phase);
    w.invincible = true;
    let mut bot = crate::sim::bot::Bot::new();
    for _ in 0..frames {
        let c = bot.decide(&w);
        w.tick(c);
        w.events.clear();
    }
    tweak(&mut w);
    w
}

/// JAWSTHIC for review, through the game's own drawing and post pass: in play, and
/// at rest, charging, in each phase's look, glitching, facing either way. Saves
/// `./shots/jawsthic_sheet.png`, 2x.
pub fn export_mockup(r: &mut Renderer) {
    use crate::sim::boss::Act;
    let (w, h) = (16 + 270 + 24 + 2 * 250 + 16, 40 + 3 * 172 + 8);
    let mut sheet = Image::gen_image_color(w as u16, h as u16, VOID);
    label(&mut sheet, "JAWSTHIC  //  STAGE 1 BOSS  //  BUILT IN CODE", 16, 12, CREAM);
    // in play: phase 1, the bot dodging
    let lay = cell(270, 360);
    let play = |world: &World, r: &mut Renderer| {
        r.parts = Particles::new();
        r.fx = Fx::new();
        r.begin(&lay);
        scene::draw(r, &lay, world, false, &[]);
        r.post(&lay, &PostParams { field: lay.field, time: world.frame as f32 / 60.0 });
        r.read_post(&lay)
    };
    let img = play(&boss_world(1, 230, |_| {}), r);
    paste(&mut sheet, &img, 16, 32);
    label(&mut sheet, "IN PLAY: PHASE 1, TEETH", 16, (32 + 360 + 6) as u32, ASH);
    // its looks, each a crop of the playfield's top
    let states: [(&str, World); 6] = [
        ("AT REST", boss_world(1, 2, |w| {
            let b = w.boss.as_mut().unwrap();
            (b.jaw, b.swim) = (0.0, 0.7);
            w.hazards.bullets.clear();
        })),
        ("JAWS OPEN, A WINDOW CHARGING", boss_world(1, 69, |w| {
            w.boss.as_mut().unwrap().jaw = 1.0;
            w.hazards.bullets.clear();
        })),
        ("PHASE 2: THE FIN TURRETS", boss_world(2, 90, |w| {
            w.hazards.bullets.clear();
            w.boss.as_mut().unwrap().flash = 0;
        })),
        ("PHASE 3: THE LASER", boss_world(3, 150 + 70, |w| {
            w.hazards.bullets.clear();
        })),
        ("PHASE 4: CRACKED, GLITCHING", boss_world(4, 30, |w| {
            let b = w.boss.as_mut().unwrap();
            b.turn = 20;
            w.hazards.bullets.clear();
        })),
        ("FACING RIGHT, ITS TAIL FIN BROKEN", boss_world(2, 4, |w| {
            let b = w.boss.as_mut().unwrap();
            (b.facing, b.x) = (1.0, crate::tune::JAWS_HOME.0);
            b.fins[1].hp = 0.0;
            b.jaw = 0.3;
            w.hazards.bullets.clear();
        })),
    ];
    for (i, (name, world)) in states.iter().enumerate() {
        let img = play(world, r);
        let (col, row) = (i % 2, i / 2);
        let (x, y) = (16 + 270 + 24 + col * 250, 32 + row * 172);
        label(&mut sheet, name, x as u32, y as u32, CREAM);
        let mut crop = Image::gen_image_color(240, 150, VOID);
        for cy in 0..150u32 {
            for cx in 0..240u32 {
                crop.set_pixel(cx, cy, img.get_pixel(cx + 15, cy + 6));
            }
        }
        paste(&mut sheet, &crop, x as u32, y as u32 + 12);
        let _ = world.boss.as_ref().map(|b| b.act == Act::Fight);
    }
    let s = 2;
    let mut big = Image::gen_image_color((w as u32 * s) as u16, (h as u32 * s) as u16, VOID);
    for y in 0..h as u32 * s {
        for x in 0..w as u32 * s {
            big.set_pixel(x, y, sheet.get_pixel(x / s, h as u32 - 1 - y / s));
        }
    }
    let _ = std::fs::create_dir_all("shots");
    big.export_png("shots/jawsthic_sheet.png");
    println!("saved shots/jawsthic_sheet.png");
}

// ---------------------------------------------------------------- the effects sheet (PG_FXSHEET=1)

/// Write a label straight into a CPU image (the sheet is assembled on the CPU).
fn label(img: &mut Image, s: &str, x: u32, y: u32, c: Color) {
    for (i, ch) in s.chars().enumerate() {
        for gy in 0..font::GH {
            for gx in 0..font::GW {
                if font::bit(ch, gx, gy) {
                    let (px, py) = (x + i as u32 * 7 + gx as u32, y + gy as u32);
                    if px < img.width as u32 && py < img.height as u32 {
                        img.set_pixel(px, py, c);
                    }
                }
            }
        }
    }
}

fn paste(dst: &mut Image, src: &Image, x: u32, y: u32) {
    for sy in 0..src.height as u32 {
        for sx in 0..src.width as u32 {
            dst.set_pixel(x + sx, y + sy, src.get_pixel(sx, sy));
        }
    }
}

/// A layout for an off-screen cell `w` x `h`, the effects applying to all of it.
fn cell(w: i32, h: i32) -> Layout {
    Layout { orient: Orient::Portrait, cw: w, ch: h, field: IRect::new(0, 0, w, h), scale: 1, ox: 0, oy: 0 }
}

/// Every explosion size at 4 moments, and the black hole at the start and end of
/// the stage, all through the real post pass. Saves `./shots/fx_sheet.png`, 2x.
pub fn export_fx(r: &mut Renderer) {
    const C: i32 = 180;
    const MOMENTS: [u32; 4] = [2, 8, 18, 34];
    let (w, h) = (16 + 4 * C + 24 + 2 * 270 + 16, 40 + 4 * C + 30);
    let mut sheet = Image::gen_image_color(w as u16, h as u16, VOID);
    label(&mut sheet, "EXPLOSIONS  //  SMALL MEDIUM LARGE BOSS  //  FRAMES 2 8 18 34", 16, 12, CREAM);
    for (row, size) in Size::ALL.iter().enumerate() {
        for (col, &m) in MOMENTS.iter().enumerate() {
            let lay = cell(C, C);
            r.parts = Particles::new();
            r.fx = Fx::new();
            let (cx, cy) = (C as f32 * 0.5, C as f32 * 0.5);
            // the Pilgrim just below, so the aberration is felt
            explode(&mut r.fx, &mut r.parts, cx, cy, *size, (cx, cy + 60.0), false, 100);
            for _ in 0..m {
                r.parts.tick((cx, cy + 60.0));
                r.fx.tick();
            }
            r.begin(&lay);
            r.parts.draw(&r.atlas, 0, 0, false);
            r.additive(true);
            r.parts.draw(&r.atlas, 0, 0, true);
            r.additive(false);
            let flash = r.fx.flash.level();
            if flash > 0.0 {
                r.atlas.fill(0, 0, C, C, Color { a: 0.35 * flash, ..CREAM });
            }
            r.post(&lay, &PostParams { field: lay.field, time: m as f32 / 60.0 });
            let img = r.read_post(&lay);
            paste(&mut sheet, &img, (16 + col as i32 * C) as u32, (32 + row as i32 * C) as u32);
        }
        label(&mut sheet, size.label(), 20, (36 + row as i32 * C) as u32, ASH);
    }
    let hx = (16 + 4 * C + 24) as u32;
    label(&mut sheet, "THE BLACK HOLE  //  START AND END OF THE STAGE", hx, 12, CREAM);
    for (i, frame) in [0, STAGE_FRAMES].iter().enumerate() {
        let lay = cell(270, 360);
        r.parts = Particles::new();
        r.fx = Fx::new();
        r.begin(&lay);
        r.stars.draw(&r.atlas, lay.field, *frame, 1.0);
        r.hole(lay.field, HOLE_X, HOLE_Y, scene::hole_radius(*frame), scene::hole_bright(1.0), *frame as f32 / 60.0, 0.0);
        r.post(&lay, &PostParams { field: lay.field, time: 0.0 });
        let img = r.read_post(&lay);
        paste(&mut sheet, &img, hx + i as u32 * 270, 32);
    }
    // scale up 2x, bottom row first (export_png flips it back)
    let s = 2;
    let mut big = Image::gen_image_color((w as u32 * s) as u16, (h as u32 * s) as u16, VOID);
    for y in 0..h as u32 * s {
        for x in 0..w as u32 * s {
            big.set_pixel(x, y, sheet.get_pixel(x / s, h as u32 - 1 - y / s));
        }
    }
    let _ = std::fs::create_dir_all("shots");
    big.export_png("shots/fx_sheet.png");
    println!("saved shots/fx_sheet.png");
}
