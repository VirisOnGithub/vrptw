use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};

use crate::problem::Problem;

pub(crate) struct MapTransform {
    min_x: f64,
    max_y: f64,
    scale: f64,
    origin: Pos2,
}

impl MapTransform {
    pub fn build(problem: &Problem, rect: Rect) -> Self {
        let inner = rect.shrink(40.0);

        let xs = problem.clients.iter().map(|c| c.x).chain([problem.repo.x]);
        let ys = problem.clients.iter().map(|c| c.y).chain([problem.repo.y]);

        let min_x = xs.clone().fold(f64::INFINITY, |a, b| a.min(b as f64));
        let max_x = xs.fold(f64::NEG_INFINITY, |a, b| a.max(b as f64));
        let min_y = ys.clone().fold(f64::INFINITY, |a, b| a.min(b as f64));
        let max_y = ys.fold(f64::NEG_INFINITY, |a, b| a.max(b as f64));

        let sx = inner.width() as f64 / (max_x - min_x).max(1.0);
        let sy = inner.height() as f64 / (max_y - min_y).max(1.0);

        MapTransform {
            min_x,
            max_y,
            scale: sx.min(sy),
            origin: inner.min,
        }
    }

    #[inline]
    pub fn to_screen(&self, x: impl Into<f64>, y: impl Into<f64>) -> Pos2 {
        Pos2 {
            x: self.origin.x + ((x.into() - self.min_x) * self.scale) as f32,
            y: self.origin.y + ((self.max_y - y.into()) * self.scale) as f32,
        }
    }
}

pub(crate) fn draw_arrow(painter: &Painter, from: Pos2, to: Pos2, stroke: Stroke) {
    painter.line_segment([from, to], stroke);
    let dir = (to - from).normalized();
    let perp = Vec2::new(-dir.y, dir.x);
    let tip = to - dir * 8.0;
    painter.line_segment([to, tip + perp * 4.0], stroke);
    painter.line_segment([to, tip - perp * 4.0], stroke);
}

const PALETTE: &[Color32] = &[
    Color32::from_rgb(220, 50, 47),
    Color32::from_rgb(38, 139, 210),
    Color32::from_rgb(133, 153, 0),
    Color32::from_rgb(211, 54, 130),
    Color32::from_rgb(42, 161, 152),
    Color32::from_rgb(181, 137, 0),
    Color32::from_rgb(108, 113, 196),
    Color32::from_rgb(203, 75, 22),
    Color32::from_rgb(0, 168, 132),
    Color32::from_rgb(255, 128, 0),
    Color32::from_rgb(0, 102, 204),
    Color32::from_rgb(153, 0, 204),
];

pub(crate) fn route_color(idx: usize) -> Color32 {
    PALETTE[idx % PALETTE.len()]
}
