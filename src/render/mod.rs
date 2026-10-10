//! Drawing. Everything renders into a low-res canvas (640x360 landscape, 270x480
//! portrait) in whole pixels, then the canvas is scaled up by a whole number with
//! nearest filtering and centered on the screen.

pub mod art;
pub mod bg;
pub mod font;
pub mod hud;
pub mod layout;
pub mod palette;
pub mod particles;
pub mod scene;
pub mod ship;

use layout::Layout;
use macroquad::miniquad;
use macroquad::prelude::*;

const VERT: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
"#;

/// The value check (F2): the canvas in grayscale, Rec. 709 luma like `palette::luma`.
const GRAY_FRAG: &str = r#"#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
void main() {
    vec3 c = texture2D(Texture, uv).rgb;
    float l = dot(c, vec3(0.2126, 0.7152, 0.0722));
    gl_FragColor = vec4(l, l, l, 1.0);
}
"#;

pub struct Renderer {
    pub atlas: art::Atlas,
    pub stars: bg::Starfield,
    pub fx: particles::Particles,
    /// Show the canvas in grayscale (F2).
    pub gray: bool,
    gray_mat: Material,
    canvas: RenderTarget,
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

impl Renderer {
    pub fn new() -> Renderer {
        let lay = Self::layout();
        let gray_mat = load_material(ShaderSource::Glsl { vertex: VERT, fragment: GRAY_FRAG }, MaterialParams::default()).expect("grayscale shader");
        Renderer {
            atlas: art::Atlas::new(),
            // enough sky for the biggest canvas (portrait is 270x480, landscape 640x360)
            stars: bg::Starfield::new(640.0 * 480.0),
            fx: particles::Particles::new(),
            gray: false,
            gray_mat,
            canvas: make_canvas(lay.cw, lay.ch),
            size: (lay.cw, lay.ch),
        }
    }

    /// The layout for the current window, in physical pixels (crisp on high-DPI phones).
    pub fn layout() -> Layout {
        let (w, h) = miniquad::window::screen_size();
        Layout::fit(w.round() as u32, h.round() as u32)
    }

    /// Start a frame: everything drawn until `end` lands on the canvas, in canvas pixels.
    pub fn begin(&mut self, lay: &Layout) {
        if self.size != (lay.cw, lay.ch) {
            self.canvas = make_canvas(lay.cw, lay.ch);
            self.size = (lay.cw, lay.ch);
        }
        set_camera(&canvas_camera(&self.canvas, lay.cw as f32, lay.ch as f32));
        clear_background(palette::VOID);
    }

    /// Finish a frame: scale the canvas up by a whole number onto the screen.
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
            &self.canvas.texture,
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
