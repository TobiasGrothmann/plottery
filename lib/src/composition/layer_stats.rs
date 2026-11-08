use crate::{BoundingBox, Layer, Plottable, Rect, Shape, V2};

/// Size and pen movement of a [`Layer`], see [`Layer::plot_stats`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotStats {
    /// Shapes in all layers.
    pub shapes: usize,
    /// Layers including the root.
    pub layers: usize,
    /// Pen-down length in cm.
    pub draw_length: f32,
    /// Pen-up length in cm, from the origin through all shapes and back.
    pub travel_length: f32,
    pub bounds: Option<Rect>,
}

impl Layer {
    /// Counts shapes and lengths in plotting order ([`Layer::iter`]), the way the plotter walks them.
    pub fn plot_stats(&self) -> PlotStats {
        let (travel_length, end) =
            self.iter().fold(
                (0.0, V2::zero()),
                |(travel, pos), shape| match entry_and_exit(shape, pos) {
                    Some((entry, exit)) => (travel + pos.dist(entry), exit),
                    None => (travel, pos),
                },
            );
        PlotStats {
            shapes: self.len(),
            layers: self.layer_count(),
            draw_length: self.iter().map(Shape::length).sum(),
            travel_length: travel_length + end.dist(V2::zero()),
            bounds: self.bounding_box(),
        }
    }

    fn layer_count(&self) -> usize {
        1 + self.sublayers.iter().map(Layer::layer_count).sum::<usize>()
    }
}

/// Where the plotter puts the pen down and lifts it when coming from `pos`, see `get_points_from`.
fn entry_and_exit(shape: &Shape, pos: V2) -> Option<(V2, V2)> {
    match shape {
        Shape::Path(path) => {
            let points = path.get_points_ref();
            let (first, last) = (*points.first()?, *points.last()?);
            if points.len() < 2 {
                None
            } else if pos.dist(last) < pos.dist(first) {
                Some((last, first))
            } else {
                Some((first, last))
            }
        }
        Shape::Circle(circle) => {
            let direction = if pos == circle.center {
                V2::new(1.0, 0.0)
            } else {
                (pos - circle.center).normalize()
            };
            let point = circle.center + direction * circle.radius;
            Some((point, point))
        }
        Shape::Rect(rect) => Some((rect.bl(), rect.bl())),
    }
}
