//! The ship's animation sheet (PG_SHEET=1 or F9), laid out like Jason's reference
//! sheet so the two can be compared side by side. Native only.

use crate::render::art::Atlas;
use crate::render::hud::text;
use crate::render::palette::{ASH, CREAM, DUSK, SIGNAL, SLATE, VOID};
use crate::render::ship::{self, Pose};
use crate::tune::{BUMP_FLASH, POD_X, POD_X_FOCUS};
use macroquad::prelude::*;

const SHEET: (f32, f32) = (420.0, 312.0);
const SHEET_SCALE: u32 = 3;

/// Draw the ship's animation sheet, laid out like Jason's reference sheet.
fn draw_sheet(a: &Atlas) {
    let label = |s: &str, x: i32, y: i32| text(a, s, x, y, 1, 2, CREAM);
    text(a, "PROTAGONIST SHIP", 10, 8, 1, 2, CREAM);
    text(a, "TYPE-07  \"PILGRIM\"  //  BUILT IN CODE", 10, 18, 1, 2, ASH);
    a.fill(212, 36, 1, SHEET.1 as i32 - 50, DUSK);
    let base = Pose { pods: [(-POD_X, 0.0), (POD_X, 0.0)], ..Default::default() };
    let row = |y: i32| y + 28;
    let col = |x0: i32, i: i32| x0 + 20 + i * 46;
    let shot = Color::new(1.0, 1.0, 1.0, 0.6);

    label("IDLE (4 FRAMES)", 10, 38);
    for i in 0..4 {
        ship::draw(a, col(10, i), row(38), &Pose { frame: i as u64 * 23 + 4, ..base });
    }
    label("THRUST (4 FRAMES)", 10, 104);
    for i in 0..4 {
        ship::draw(a, col(10, i), row(104), &Pose { thrust: true, frame: i as u64 * 3, ..base });
    }
    label("FIRE (4 FRAMES)", 10, 170);
    for i in 0..4 {
        let (x, y) = (col(10, i), row(170) + 6);
        for side in [-4, 4] {
            a.blit(a.r.ship.bolt[i as usize], x + side - 1, y - 30 - i * 2, 1, shot);
        }
        ship::draw(a, x, y, &Pose { fire_t: i as u32 + 1, frame: i as u64, ..base });
    }
    label("HIT FLASH (4 FRAMES)", 10, 236);
    for i in 0..4 {
        let hit_t = BUMP_FLASH - (i as u32 * BUMP_FLASH / 4);
        ship::draw(a, col(10, i), row(236), &Pose { hit_t, frame: 4, ..base });
    }

    label("BANK (L2 L1 R1 R2)", 222, 38);
    for (i, b) in [-2i8, -1, 1, 2].into_iter().enumerate() {
        ship::draw(a, col(222, i as i32) - 6, row(38), &Pose { bank: b, frame: 4, ..base });
    }
    label("FOCUS + BEAM", 222, 104);
    let tucked = Pose { pods: [(-POD_X_FOCUS, 0.0), (POD_X_FOCUS, 0.0)], frame: 4, ..base };
    let (fx, fy) = (col(222, 0), row(104) + 4);
    ship::draw(a, fx, fy, &tucked);
    ship::draw_hitbox(a, fx, fy, 1.0, 0);
    let (bx, by) = (col(222, 2), row(104) + 4);
    a.fill(bx - 6, by - 32, 12, 21, Color { a: 0.18, ..SIGNAL });
    a.fill(bx - 4, by - 32, 8, 21, Color { a: 0.34, ..SIGNAL });
    a.fill(bx - 1, by - 32, 2, 21, Color { a: 0.55, ..CREAM });
    ship::draw(a, bx, by, &Pose { beaming: true, fire_t: 2, ..tucked });
    ship::draw_hitbox(a, bx, by, 1.0, 9);
    label("PROJECTILE (4 FRAMES)", 222, 170);
    for i in 0..4 {
        a.blit(a.r.ship.bolt[i], col(222, i as i32) - 6, row(170) - 12, 1, shot);
        a.blit(a.r.ship.bolt[i], col(222, i as i32) + 2, row(170) - 16, 2, shot);
    }
    label("POD SHOT  //  AFTERIMAGES", 222, 236);
    for i in 0..2 {
        a.blit(a.r.ship.pod_shot[i], col(222, 0) - 8 + i as i32 * 8, row(236) - 8, 1, Color::new(1.0, 1.0, 1.0, 0.45));
    }
    let (ax, ay) = (col(222, 2), row(236));
    ship::draw_afterimages(a, 2, &[(ax - 9, ay + 3), (ax - 18, ay + 6), (ax - 27, ay + 9)]);
    ship::draw(a, ax, ay, &Pose { bank: 2, frame: 4, ..base });
    a.fill(10, SHEET.1 as i32 - 13, SHEET.0 as i32 - 20, 1, DUSK);
    text(a, &crate::render::hud::version_stamp(), 10, SHEET.1 as i32 - 9, 1, 2, SLATE);
}

/// Render the sheet off-screen and save it as `./shots/pilgrim_sheet.png`, 3x.
pub fn export(a: &Atlas) {
    let rt = render_target(SHEET.0 as u32, SHEET.1 as u32);
    rt.texture.set_filter(FilterMode::Nearest);
    set_camera(&crate::render::canvas_camera(&rt, SHEET.0, SHEET.1));
    clear_background(VOID);
    draw_sheet(a);
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
    big.export_png("shots/pilgrim_sheet.png");
    println!("saved shots/pilgrim_sheet.png");
}
