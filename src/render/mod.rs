//! Drawing, in three passes, all at low resolution until the last:
//!   1. the scene (background, the black hole, everything in play) into a canvas
//!      (640x360 landscape, 270x480 portrait), in whole pixels;
//!   2. the post pass (shock rings, aberration, bloom, vignette, grain, shake) into
//!      a second canvas, then the HUD on top, untouched by the effects;
//!   3. that canvas scaled up by a whole number with nearest filtering, centered.

pub mod art;
pub mod bg;
pub mod bullets;
pub mod explode;
pub mod font;
pub mod foes;
pub mod hud;
pub mod layout;
pub mod palette;
pub mod particles;
pub mod post;
pub mod scene;
pub mod ship;

use layout::{IRect, Layout};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams};
use macroquad::miniquad;
use macroquad::prelude::*;
use post::{ADD_FRAG, Fx, HOLE_FRAG, POST_FRAG, VERT};

/// The value check (F2): the canvas in grayscale, Rec. 709 luma like `palette::luma`.
const GRAY_FRAG: &str = r#"#version 100
precision mediump float;
varying vec2 uv;
uniform sampler2D Texture;
void main() {
    vec3 c = texture2D(Texture, uv).rgb;
    float l = dot(c, vec3(0.2126, 0.7152, 0.0722));
    gl_FragColor = vec4(l, l, l, 1.0);
}
"#;

/// Everything the post pass does this frame.
pub struct PostParams {
    /// Where the effects apply (the playfield, or the whole canvas on the title).
    pub field: IRect,
    /// Seconds, for the grain.
    pub time: f32,
}

pub struct Renderer {
    pub atlas: art::Atlas,
    pub stars: bg::Starfield,
    pub parts: particles::Particles,
    pub fx: Fx,
    /// Show the canvas in grayscale (F2).
    pub gray: bool,
    gray_mat: Material,
    post_mat: Material,
    hole_mat: Material,
    add_mat: Material,
    scene: RenderTarget,
    post: RenderTarget,
    size: (i32, i32),
}

fn make_canvas(w: i32, h: i32) -> RenderTarget {
    let rt = render_target(w as u32, h as u32);
    rt.texture.set_filter(FilterMode::Nearest);
    rt
}

/// A camera drawing into `rt` in its own pixels, y down.
pub fn canvas_camera(rt: &RenderTarget, w: f32, h: f32) -> Camera2D {
    Camera2D {
        target: vec2(w * 0.5, h * 0.5),
        zoom: vec2(2.0 / w, 2.0 / h),
        render_target: Some(rt.clone()),
        ..Default::default()
    }
}

fn material(frag: &str, uniforms: &[(&str, UniformType)], blend: Option<(BlendFactor, BlendFactor)>) -> Material {
    let pipeline_params = match blend {
        Some((src, dst)) => PipelineParams {
            color_blend: Some(BlendState::new(Equation::Add, src, dst)),
            alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One)),
            ..Default::default()
        },
        None => PipelineParams::default(),
    };
    load_material(
        ShaderSource::Glsl { vertex: VERT, fragment: frag },
        MaterialParams { uniforms: uniforms.iter().map(|(n, t)| UniformDesc::new(n, *t)).collect(), pipeline_params, ..Default::default() },
    )
    .expect("shader")
}

impl Renderer {
    pub fn new() -> Renderer {
        let lay = Self::layout();
        use UniformType::*;
        let post_mat = material(
            POST_FRAG,
            &[("Res", Float2), ("Field", Float4), ("Shock0", Float4), ("Shock1", Float4), ("Shock2", Float4), ("Shock3", Float4), ("Fx", Float4), ("Shake", Float2), ("Cap", Float3), ("Grain", Float1)],
            None,
        );
        // the hole adds its light and its shadow covers what's behind (premultiplied)
        let hole_mat = material(HOLE_FRAG, &[("Origin", Float2), ("Size", Float2), ("Hole", Float3), ("Look", Float4), ("Ink", Float3)], Some((BlendFactor::One, BlendFactor::OneMinusValue(BlendValue::SourceAlpha))));
        let add_mat = material(ADD_FRAG, &[], Some((BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::One)));
        Renderer {
            atlas: art::Atlas::new(),
            // enough sky for the biggest canvas (portrait is 270x480, landscape 640x360)
            stars: bg::Starfield::new(640.0 * 480.0),
            parts: particles::Particles::new(),
            fx: Fx::new(),
            gray: false,
            gray_mat: material(GRAY_FRAG, &[], None),
            post_mat,
            hole_mat,
            add_mat,
            scene: make_canvas(lay.cw, lay.ch),
            post: make_canvas(lay.cw, lay.ch),
            size: (lay.cw, lay.ch),
        }
    }

    /// The layout for the current window, in physical pixels (crisp on high-DPI phones).
    pub fn layout() -> Layout {
        let (w, h) = miniquad::window::screen_size();
        Layout::fit(w.round() as u32, h.round() as u32)
    }

    /// Start a frame: the scene pass, in canvas pixels.
    pub fn begin(&mut self, lay: &Layout) {
        if self.size != (lay.cw, lay.ch) {
            self.scene = make_canvas(lay.cw, lay.ch);
            self.post = make_canvas(lay.cw, lay.ch);
            self.size = (lay.cw, lay.ch);
        }
        set_camera(&canvas_camera(&self.scene, lay.cw as f32, lay.ch as f32));
        clear_background(palette::VOID);
    }

    /// Draw the black hole over `area`: center `(cx, cy)` and shadow radius in area px,
    /// `bright` its value ceiling, `time` in seconds, `flare` how much the disk flares
    /// (0 calm; it lights more of the disk, never past the ceiling).
    pub fn hole(&self, area: IRect, cx: f32, cy: f32, radius: f32, bright: f32, time: f32, flare: f32) {
        let ink = palette::CREAM;
        let k = 1.0 / palette::luma(ink);
        let m = &self.hole_mat;
        m.set_uniform("Origin", vec2(area.x as f32, area.y as f32));
        m.set_uniform("Size", vec2(area.w as f32, area.h as f32));
        m.set_uniform("Hole", vec3(cx, cy, radius));
        m.set_uniform("Look", vec4(time, bright, flare, crate::tune::HOLE_TILT));
        m.set_uniform("Ink", vec3(ink.r * k, ink.g * k, ink.b * k));
        gl_use_material(m);
        draw_texture_ex(&self.atlas.tex, area.x as f32, area.y as f32, WHITE, DrawTextureParams { dest_size: Some(vec2(area.w as f32, area.h as f32)), ..Default::default() });
        gl_use_default_material();
    }

    /// Draw enough quads (off-screen, then cleared) to make macroquad create every
    /// batch buffer a busy frame will need, before play starts.
    pub fn prewarm(&self) {
        let quads = crate::tune::PREWARM_DRAW_CALLS * crate::tune::DRAW_CALL_INDICES / 6;
        for i in 0..quads {
            self.atlas.fill((i % 8) as i32, 0, 1, 1, Color::new(0.0, 0.0, 0.0, 0.0));
        }
    }

    /// Switch particle drawing to the additive blend (and back).
    pub fn additive(&self, on: bool) {
        if on {
            gl_use_material(&self.add_mat);
        } else {
            gl_use_default_material();
        }
    }

    /// End the scene pass: run the post pass into the HUD canvas. Anything drawn
    /// after this (the HUD) sits on top, untouched by the effects.
    pub fn post(&self, lay: &Layout, p: &PostParams) {
        let (w, h) = (lay.cw as f32, lay.ch as f32);
        set_camera(&canvas_camera(&self.post, w, h));
        clear_background(palette::VOID);
        let m = &self.post_mat;
        let f = p.field;
        m.set_uniform("Res", vec2(w, h));
        m.set_uniform("Field", vec4(f.x as f32, f.y as f32, f.w as f32, f.h as f32));
        for i in 0..4 {
            let s = match self.fx.shocks.get(i) {
                Some(s) => {
                    let (r, push) = s.now();
                    vec4(f.x as f32 + s.x, f.y as f32 + s.y, r, push)
                }
                None => vec4(-999.0, -999.0, 0.0, 0.0),
            };
            m.set_uniform(&format!("Shock{i}"), s);
        }
        use crate::tune::{BLOOM, GRAIN, VIGNETTE};
        m.set_uniform("Fx", vec4(self.fx.ca, p.time, VIGNETTE, BLOOM));
        let (sx, sy) = self.fx.shake_px();
        m.set_uniform("Shake", vec2(sx, sy));
        let c = palette::CORE;
        m.set_uniform("Cap", vec3(c.r, c.g, c.b));
        m.set_uniform("Grain", GRAIN);
        gl_use_material(m);
        draw_texture_ex(&self.scene.texture, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
        gl_use_default_material();
    }

    /// Read the HUD canvas back (for the review sheets), top row first (render
    /// targets come back that way; `Image::export_png` expects bottom row first).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_post(&self, lay: &Layout) -> Image {
        unsafe { get_internal_gl().flush() };
        let img = self.post.texture.get_texture_data();
        let (w, h) = (img.width as u32, img.height as u32);
        let mut out = Image::gen_image_color(lay.cw as u16, lay.ch as u16, palette::VOID);
        for y in 0..h.min(lay.ch as u32) {
            for x in 0..w.min(lay.cw as u32) {
                out.set_pixel(x, y, img.get_pixel(x, y));
            }
        }
        out
    }

    /// Finish a frame: scale the HUD canvas up by a whole number onto the screen.
    pub fn end(&self, lay: &Layout) {
        let (pw, ph) = miniquad::window::screen_size();
        set_camera(&Camera2D {
            target: vec2(pw * 0.5, ph * 0.5),
            zoom: vec2(2.0 / pw, 2.0 / ph),
            ..Default::default()
        });
        clear_background(palette::VOID);
        if self.gray {
            gl_use_material(&self.gray_mat);
        }
        let s = lay.scale as f32;
        draw_texture_ex(
            &self.post.texture,
            lay.ox as f32,
            lay.oy as f32,
            WHITE,
            DrawTextureParams { dest_size: Some(vec2(lay.cw as f32 * s, lay.ch as f32 * s)), ..Default::default() },
        );
        if self.gray {
            gl_use_default_material();
        }
    }
}
