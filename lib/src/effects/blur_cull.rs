use std::collections::HashMap;

use crate::{Line, Path, Shape, V2};

type Cell = (i64, i64);

/// Items bucketed by position. Anything within one cell size of a point is in its 3×3 neighbourhood.
struct Grid<T> {
    cell_size: f32,
    cells: HashMap<Cell, Vec<T>>,
}

impl<T> Grid<T> {
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    fn cell(&self, point: V2) -> Cell {
        (
            (point.x / self.cell_size).floor() as i64,
            (point.y / self.cell_size).floor() as i64,
        )
    }

    fn near(&self, point: V2) -> impl Iterator<Item = &T> {
        let (x, y) = self.cell(point);
        (-1..=1)
            .flat_map(move |dx| (-1..=1).map(move |dy| (x + dx, y + dy)))
            .filter_map(|cell| self.cells.get(&cell))
            .flatten()
    }
}

/// Drops dots closer than `min_distance` to a path or to an earlier kept dot. Paths are kept.
pub(super) fn cull_dots(shapes: Vec<Shape>, min_distance: f32, cell_size: f32) -> Vec<Shape> {
    let cell_size = cell_size.max(min_distance);
    let mut segments: Grid<Line> = Grid::new(cell_size);
    let lines = shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::Path(path) => Some(path),
            _ => None,
        })
        .flat_map(Path::segments);
    for line in lines {
        let (x_from, y_from) = segments.cell(line.from.min(line.to));
        let (x_to, y_to) = segments.cell(line.from.max(line.to));
        for cell in (x_from..=x_to).flat_map(|x| (y_from..=y_to).map(move |y| (x, y))) {
            segments.cells.entry(cell).or_default().push(line);
        }
    }

    let min_distance_squared = min_distance * min_distance;
    let mut dots: Grid<V2> = Grid::new(cell_size);
    shapes
        .into_iter()
        .filter(|shape| {
            let Shape::Circle(dot) = shape else {
                return true;
            };
            let point = dot.center;
            let near_ink = segments
                .near(point)
                .any(|line| line.closest_point(point).dist_squared(point) < min_distance_squared)
                || dots
                    .near(point)
                    .any(|kept| kept.dist_squared(point) < min_distance_squared);
            if !near_ink {
                let cell = dots.cell(point);
                dots.cells.entry(cell).or_default().push(point);
            }
            !near_ink
        })
        .collect()
}
