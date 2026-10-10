//! Axis-aligned boxes in playfield px.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl Rect {
    /// A box `w` x `h` centered on `(cx, cy)`.
    pub fn centered(cx: f32, cy: f32, w: f32, h: f32) -> Rect {
        Rect { x0: cx - w * 0.5, y0: cy - h * 0.5, x1: cx + w * 0.5, y1: cy + h * 0.5 }
    }

    pub fn overlaps(&self, o: &Rect) -> bool {
        self.x0 < o.x1 && o.x0 < self.x1 && self.y0 < o.y1 && o.y0 < self.y1
    }

    /// Distance from a point to the nearest part of the box (0 inside it).
    pub fn dist(&self, x: f32, y: f32) -> f32 {
        let dx = (self.x0 - x).max(0.0).max(x - self.x1);
        let dy = (self.y0 - y).max(0.0).max(y - self.y1);
        (dx * dx + dy * dy).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boxes_overlap_and_measure() {
        let a = Rect::centered(0.0, 0.0, 10.0, 10.0);
        assert!(a.overlaps(&Rect::centered(9.0, 0.0, 10.0, 10.0)));
        assert!(!a.overlaps(&Rect::centered(10.0, 0.0, 10.0, 10.0)), "touching edges don't count");
        assert_eq!(a.dist(0.0, 0.0), 0.0);
        assert_eq!(a.dist(8.0, 0.0), 3.0);
        assert_eq!(a.dist(8.0, 9.0), 5.0);
    }
}
