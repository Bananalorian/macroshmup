//! Light: the post pass (shock rings, chromatic aberration, bloom, vignette,
//! grain, all locked to the low-res pixel grid), the black hole shader, the
//! additive blend for sparks, and the state behind them: shocks, the aberration
//! spike, screen shake and full-screen flashes.

use crate::tune::*;

pub const VERT: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying vec2 uv;
varying vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

/// Sparks, embers and streaks add light instead of covering what's under them.
pub const ADD_FRAG: &str = r#"#version 100
precision mediump float;
varying vec2 uv;
varying vec4 color;
uniform sampler2D Texture;
void main() {
    gl_FragColor = texture2D(Texture, uv) * color;
}
"#;

/// The post pass over the scene. Everything samples whole pixels, so distortion
/// stays on the grid. `Fx` is (aberration px, time s, vignette, bloom). `Cap` is
/// the bullet core color: only bullet cores reach it, so however many sparks pile
/// up, the cores stay the brightest pixels on screen.
pub const POST_FRAG: &str = r#"#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
uniform sampler2D Texture;
uniform vec2 Res;
uniform vec4 Field;
uniform vec4 Shock0;
uniform vec4 Shock1;
uniform vec4 Shock2;
uniform vec4 Shock3;
uniform vec4 Fx;
uniform vec2 Shake;
uniform vec3 Cap;
uniform float Grain;

float h21(vec2 q) { return fract(sin(dot(q, vec2(127.1, 311.7))) * 43758.5453); }

vec2 push(vec4 s, vec2 p) {
    vec2 d = p - s.xy;
    float dist = length(d);
    float k = (dist - s.z) / 3.0;
    return dist > 0.5 ? d / dist * s.w * exp(-k * k) : vec2(0.0);
}

vec3 tap(vec2 p) {
    p = clamp(p, Field.xy + 0.5, Field.xy + Field.zw - 0.5);
    return texture2D(Texture, (floor(p) + 0.5) / Res).rgb;
}

void main() {
    // gl_FragCoord is exact on every GPU (and in a render target, row 0 is the canvas top)
    vec2 px = floor(gl_FragCoord.xy) + 0.5;
    bool inside = px.x > Field.x && px.x < Field.x + Field.z && px.y > Field.y && px.y < Field.y + Field.w;
    if (!inside) {
        gl_FragColor = vec4(texture2D(Texture, px / Res).rgb, 1.0);
        return;
    }
    // shake, then the shock rings push pixels outward, in whole pixels
    vec2 p = px - Shake;
    vec2 off = push(Shock0, p) + push(Shock1, p) + push(Shock2, p) + push(Shock3, p);
    p -= floor(off + 0.5);

    // chromatic aberration: red and blue pulled apart along the radius
    vec2 c = Field.xy + Field.zw * 0.5;
    vec2 dir = p - c;
    dir = dir / max(length(dir), 1.0);
    vec2 ca = floor(dir * Fx.x + 0.5);
    vec3 col = vec3(tap(p + ca).r, tap(p).g, tap(p - ca).b);

    // bloom: the hot spots glow onto the dark around them
    vec3 b = vec3(0.0);
    for (int i = 0; i < 8; i++) {
        float a = float(i) * 0.7854 + 0.39;
        float r = 2.0 + mod(float(i), 2.0) * 2.0;
        b += max(tap(p + floor(vec2(cos(a), sin(a)) * r + 0.5)) - 0.6, 0.0);
    }
    float base = dot(col, vec3(0.2126, 0.7152, 0.0722));
    col += b * Fx.w * (1.0 - smoothstep(0.3, 0.7, base));

    // vignette and film grain
    vec2 v = (px - c) / Field.zw;
    col *= 1.0 - dot(v, v) * Fx.z;
    col += (h21(px + floor(Fx.y * 24.0) * 7.31) - 0.5) * Grain;
    // only a bullet's core may reach the core color; everything else stays a step below
    vec3 here = tap(p);
    bool core = all(lessThan(abs(here - Cap), vec3(0.01)));
    gl_FragColor = vec4(min(max(col, 0.0), core ? Cap : Cap * 0.93), 1.0);
}
"#;

/// The black hole, after the falling-figure image: a shadow, a thin photon ring,
/// the disk seen nearly edge-on with its far side lensed over the top, one side
/// brighter, engraved stipple shading, and stars smeared into arcs round the edge.
/// Premultiplied: the light covers what's behind in proportion and the shadow
/// hides it, so the background stays inside its value budget.
pub const HOLE_FRAG: &str = r#"#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
uniform vec2 Origin;
uniform vec2 Size;
uniform vec3 Hole;
uniform vec4 Look;
uniform vec3 Ink;

float h21(vec2 q) { return fract(sin(dot(q, vec2(127.1, 311.7))) * 43758.5453); }
float bayer2(vec2 a) { a = floor(a); return fract(a.x / 2.0 + a.y * a.y * 0.75); }
float bayer4(vec2 a) { return bayer2(0.5 * a) * 0.25 + bayer2(a); }

void main() {
    vec2 px = floor(gl_FragCoord.xy - Origin) + 0.5;
    vec2 d = (px - Hole.xy) / Hole.z;
    float r = length(d);
    float t = Look.x;
    float tilt = Look.w;
    // the approaching side of the disk is brighter
    float dop = 1.0 + 0.6 * clamp(-d.x / max(r, 0.6), -1.0, 1.0);

    // fine engraved streaks: grooves that wander a little, so they read as lines, not rings
    float wob = h21(floor(px / 2.0)) * 0.6;

    // the disk, nearly edge-on: brightest at its inner edge, fading outward
    vec2 q = vec2(d.x, d.y / tilt);
    float rho = length(q);
    float disk = smoothstep(1.25, 1.6, rho) * pow(1.55 / max(rho, 1.55), 1.7) * (1.0 - smoothstep(3.5, 5.0, rho));
    disk *= (0.7 + 0.3 * sin(rho * 21.0 - t * 0.7 + wob)) * dop;

    // the far side, lensed up over the shadow in a thick glowing arc (and a sliver underneath)
    float top = clamp(-d.y / max(r, 0.001), 0.0, 1.0);
    float bot = clamp(d.y / max(r, 0.001), 0.0, 1.0);
    float arc = smoothstep(1.03, 1.1, r) * pow(1.12 / max(r, 1.12), 3.0) * (0.15 + 0.85 * pow(top, 0.6));
    arc += smoothstep(1.02, 1.06, r) * (1.0 - smoothstep(1.06, 1.25, r)) * bot * 0.45;
    arc *= (0.72 + 0.28 * sin(r * 30.0 + t * 0.5 + wob)) * dop * 1.2;

    // the photon ring: about a pixel of light at the shadow's edge
    float ring = exp(-pow((r - 1.04) * Hole.z / 0.9, 2.0));

    // stars behind, smeared into short arcs by the lensing
    float stars = 0.0;
    if (r > 1.15 && r < 3.2) {
        vec2 sp = vec2(atan(d.y, d.x) * 34.0 / r, log(r) * 70.0);
        if (h21(floor(sp)) > 0.982) stars = 0.8 * (1.0 - smoothstep(1.6, 3.2, r));
    }

    float shadow = 1.0 - smoothstep(0.97, 1.0, r);
    float near = d.y > 0.0 ? 1.0 : 1.0 - shadow;
    float light = (arc + ring + stars) * (1.0 - shadow) + disk * near;

    // engraved stipple: a few levels of cream, ordered-dithered
    float v = clamp(light, 0.0, 1.0);
    float lv = floor(v * 4.0 + bayer4(px)) / 4.0;
    // premultiplied: the light covers what's behind in proportion, so the sky
    // never gets brighter than the hole's own ceiling
    gl_FragColor = vec4(Ink * lv * Look.y, max(shadow, lv));
}
"#;

/// A shock ring in flight.
#[derive(Clone, Copy, Debug)]
pub struct Shock {
    /// Center in playfield px.
    pub x: f32,
    pub y: f32,
    pub age: u32,
    pub life: u32,
    pub reach: f32,
    pub push: f32,
}

impl Shock {
    /// (radius px, push px) right now.
    pub fn now(&self) -> (f32, f32) {
        let k = self.age as f32 / self.life.max(1) as f32;
        let ease = 1.0 - (1.0 - k) * (1.0 - k);
        (self.reach * ease, self.push * (1.0 - k))
    }
}

/// Full-screen flashes, never more than 3 a second (WCAG 2.3.1).
#[derive(Default)]
pub struct Flash {
    last: Option<u64>,
    left: u32,
}

impl Flash {
    /// Ask for a flash on sim frame `frame`; returns whether it was allowed.
    pub fn trigger(&mut self, frame: u64) -> bool {
        if self.last.is_some_and(|l| frame.saturating_sub(l) < FLASH_GAP) {
            return false;
        }
        self.last = Some(frame);
        self.left = FLASH_FRAMES;
        true
    }

    pub fn tick(&mut self) {
        self.left = self.left.saturating_sub(1);
    }

    /// How strong the flash is now, 0..1.
    pub fn level(&self) -> f32 {
        self.left as f32 / FLASH_FRAMES as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShakeSetting {
    Off,
    Low,
    Full,
}

/// Everything the post pass animates. Cosmetic, advanced once per sim tick.
#[derive(Default)]
pub struct Fx {
    pub shocks: Vec<Shock>,
    /// Chromatic aberration, px.
    pub ca: f32,
    /// Shake strength, 0..1.
    pub shake: f32,
    pub flash: Flash,
    frame: u64,
}

impl Fx {
    pub fn new() -> Fx {
        Fx { shocks: Vec::with_capacity(SHOCK_MAX), ..Default::default() }
    }

    /// A shock ring; past the limit, it replaces the oldest.
    pub fn shock(&mut self, x: f32, y: f32, reach: f32, push: f32, life: u32) {
        let s = Shock { x, y, age: 0, life, reach, push };
        if self.shocks.len() < SHOCK_MAX {
            self.shocks.push(s);
        } else if let Some(old) = self.shocks.iter_mut().max_by_key(|s| s.age) {
            *old = s;
        }
    }

    pub fn aberrate(&mut self, px: f32) {
        self.ca = self.ca.max(px);
    }

    /// Shake the screen. Never while enemy bullets are live (they'd be harder to
    /// read); only after the screen's been cancelled. Returns whether it shook.
    pub fn shake(&mut self, amount: f32, bullets_live: usize, setting: ShakeSetting) -> bool {
        let k = match setting {
            ShakeSetting::Off => 0.0,
            ShakeSetting::Low => SHAKE_LOW,
            ShakeSetting::Full => 1.0,
        };
        if bullets_live > 0 || k == 0.0 {
            return false;
        }
        self.shake = self.shake.max(amount * k);
        true
    }

    pub fn tick(&mut self) {
        self.frame += 1;
        for s in &mut self.shocks {
            s.age += 1;
        }
        self.shocks.retain(|s| s.age < s.life);
        self.ca *= CA_DECAY;
        if self.ca < 0.05 {
            self.ca = 0.0;
        }
        self.shake *= SHAKE_DECAY;
        if self.shake < 0.05 {
            self.shake = 0.0;
        }
        self.flash.tick();
    }

    /// Stop shaking at once (enemy bullets appeared).
    pub fn calm(&mut self) {
        self.shake = 0.0;
    }

    /// This frame's shake offset in whole pixels.
    pub fn shake_px(&self) -> (f32, f32) {
        if self.shake <= 0.0 {
            return (0.0, 0.0);
        }
        let mut z = self.frame.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        z ^= z >> 31;
        let a = (z % 6283) as f32 / 1000.0;
        let m = self.shake * SHAKE_PX;
        ((a.cos() * m).round(), (a.sin() * m).round())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_more_than_three_flashes_a_second() {
        let mut f = Flash::default();
        // ask every frame for 10 seconds
        let allowed: Vec<u64> = (0..600).filter(|&fr| f.trigger(fr)).collect();
        for w in 0..540 {
            let in_window = allowed.iter().filter(|&&a| a >= w && a < w + 60).count();
            assert!(in_window <= 3, "{in_window} flashes in the second from frame {w}");
        }
        assert!(allowed.len() >= 29, "but it does flash");
    }

    #[test]
    fn no_shake_while_enemy_bullets_are_live() {
        let mut fx = Fx::new();
        assert!(!fx.shake(1.0, 12, ShakeSetting::Full));
        assert_eq!(fx.shake_px(), (0.0, 0.0));
        assert!(fx.shake(1.0, 0, ShakeSetting::Full));
        assert!(fx.shake > 0.0);
        let mut seen = false;
        for _ in 0..4 {
            fx.tick();
            seen |= fx.shake_px() != (0.0, 0.0);
        }
        assert!(seen, "it does move the screen");
        fx.calm();
        assert_eq!(fx.shake_px(), (0.0, 0.0));
    }

    #[test]
    fn the_shake_setting_scales_or_stops_it() {
        let mut fx = Fx::new();
        assert!(!fx.shake(1.0, 0, ShakeSetting::Off));
        assert_eq!(fx.shake, 0.0);
        fx.shake(1.0, 0, ShakeSetting::Low);
        assert_eq!(fx.shake, SHAKE_LOW);
        for _ in 0..120 {
            fx.tick();
        }
        assert_eq!(fx.shake, 0.0, "it settles");
    }

    #[test]
    fn aberration_spikes_and_fades_in_about_a_third_of_a_second() {
        let mut fx = Fx::new();
        fx.aberrate(3.0);
        for _ in 0..18 {
            fx.tick();
        }
        assert!(fx.ca < 3.0 * 0.05 + 0.01, "{}", fx.ca);
        for _ in 0..30 {
            fx.tick();
        }
        assert_eq!(fx.ca, 0.0, "it rests at zero");
    }

    #[test]
    fn at_most_four_shock_rings() {
        let mut fx = Fx::new();
        for i in 0..6 {
            fx.shock(i as f32, 0.0, 50.0, 2.0, 30);
            fx.tick();
        }
        assert_eq!(fx.shocks.len(), SHOCK_MAX);
        assert!(fx.shocks.iter().any(|s| s.x == 5.0), "the newest replaced the oldest");
        let s = fx.shocks[0];
        let (r0, p0) = Shock { age: 0, ..s }.now();
        let (r1, p1) = Shock { age: 20, ..s }.now();
        assert!(r1 > r0 && p1 < p0, "it spreads and weakens");
        for _ in 0..40 {
            fx.tick();
        }
        assert!(fx.shocks.is_empty());
    }
}
