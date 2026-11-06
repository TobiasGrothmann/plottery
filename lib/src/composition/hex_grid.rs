use std::collections::HashSet;

use crate::{Angle, Layer, Path, Plottable, Rect, V2i, LARGE_EPSILON, V2};

const SQRT_3: f32 = 1.732_050_8;

/// A pointy-top hexagonal cell, see [`HexGrid`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexCell {
    pub center: V2,
    /// Distance from center to a corner.
    pub radius: f32,
    /// Axial grid coordinates `(q, r)`.
    pub axial: V2i,
}

impl HexCell {
    /// Corners counter-clockwise, starting at the lower right.
    pub fn corners(&self) -> [V2; 6] {
        std::array::from_fn(|i| self.corner(i))
    }

    fn corner(&self, i: usize) -> V2 {
        self.center + V2::polar(Angle::from_degrees(-30.0 + 60.0 * i as f32), self.radius)
    }

    /// Closed hexagon outline.
    #[must_use]
    pub fn outline(&self) -> Path {
        Path::new_from(self.corners().to_vec()).closed()
    }

    /// Six closed triangles between the center and adjacent corners.
    pub fn triangles(&self) -> [Path; 6] {
        std::array::from_fn(|i| {
            Path::new_from(vec![self.center, self.corner(i), self.corner((i + 1) % 6)]).closed()
        })
    }
}

/// Pointy-top hexagonal grid covering `bounds`, with cell `(0, 0)` centered at the bottom left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexGrid {
    pub bounds: Rect,
    /// Distance between two parallel edges of a cell.
    pub line_dist: f32,
}

impl HexGrid {
    pub fn new(bounds: Rect, line_dist: f32) -> Self {
        assert!(line_dist > 0.0, "line_dist must be greater than 0");
        Self { bounds, line_dist }
    }

    /// Distance from a cell center to its corners.
    pub fn radius(&self) -> f32 {
        self.line_dist / SQRT_3
    }

    pub fn cell(&self, axial: V2i) -> HexCell {
        let x = self.line_dist * (axial.x as f32 + axial.y as f32 * 0.5);
        let y = self.radius() * 1.5 * axial.y as f32;
        HexCell {
            center: self.bounds.bl() + V2::new(x, y),
            radius: self.radius(),
            axial,
        }
    }

    pub fn neighbors(&self, axial: V2i) -> [V2i; 6] {
        NEIGHBOR_DIRECTIONS.map(|(dq, dr)| V2i::new(axial.x + dq, axial.y + dr))
    }

    /// All cells overlapping `bounds`, row by row.
    pub fn cells(&self) -> impl Iterator<Item = HexCell> + '_ {
        self.axials_overlapping(self.bounds)
            .map(|(q, r)| self.cell(V2i::new(q, r)))
    }

    /// Cell centers inside `bounds`.
    pub fn centers(&self) -> Vec<V2> {
        let bounds = self.bounds_inclusive();
        self.cells()
            .map(|cell| cell.center)
            .filter(|p| bounds.contains_point(*p))
            .collect()
    }

    /// Cell corners inside `bounds`, each once. Together with [`HexGrid::centers`] this is a triangular lattice.
    pub fn vertices(&self) -> Vec<V2> {
        let r = self.radius();
        let expanded = Rect::new(self.bounds.bl() - V2::xy(r), self.bounds.tr() + V2::xy(r));
        let bounds = self.bounds_inclusive();
        self.axials_overlapping(expanded)
            .map(|(q, r)| self.cell(V2i::new(q, r)))
            .flat_map(|cell| [cell.corner(1), cell.corner(4)])
            .filter(|p| bounds.contains_point(*p))
            .collect()
    }

    /// `bounds` grown by an epsilon, so that points on the boundary count as inside.
    fn bounds_inclusive(&self) -> Rect {
        Rect::new(
            self.bounds.bl() - V2::xy(LARGE_EPSILON),
            self.bounds.tr() + V2::xy(LARGE_EPSILON),
        )
    }

    /// Outlines of all cells overlapping `bounds`, every edge drawn once.
    #[must_use]
    pub fn outlines(&self) -> Layer {
        let axials: HashSet<(i32, i32)> = self.axials_overlapping(self.bounds).collect();
        let mut layer = Layer::new();
        for &(q, r) in &axials {
            let cell = self.cell(V2i::new(q, r));
            // own edges: towards the right, upper right and upper left neighbor
            layer.push(Path::new_from(vec![
                cell.corner(0),
                cell.corner(1),
                cell.corner(2),
                cell.corner(3),
            ]));
            // remaining edges, only where the neighbor is not part of the grid
            let mut path = Path::new();
            for (i, (dq, dr)) in NEIGHBOR_DIRECTIONS.into_iter().enumerate().skip(3) {
                if axials.contains(&(q + dq, r + dr)) {
                    if path.get_points_ref().len() > 1 {
                        layer.push(path);
                    }
                    path = Path::new();
                } else {
                    if path.is_empty() {
                        path.push(cell.corner(i));
                    }
                    path.push(cell.corner((i + 1) % 6));
                }
            }
            if path.get_points_ref().len() > 1 {
                layer.push(path);
            }
        }
        layer
    }

    fn axials_overlapping(&self, bounds: Rect) -> impl Iterator<Item = (i32, i32)> + '_ {
        let r = self.radius();
        let origin = self.bounds.bl();
        let row_height = r * 1.5;
        let r_min = ((bounds.bl().y - origin.y - r) / row_height).ceil() as i32;
        let r_max = ((bounds.tr().y - origin.y + r) / row_height).floor() as i32;
        (r_min..=r_max).flat_map(move |row| {
            let row_offset = row as f32 * 0.5 * self.line_dist;
            let half = self.line_dist * 0.5;
            let q_min =
                ((bounds.bl().x - origin.x - row_offset - half) / self.line_dist).ceil() as i32;
            let q_max =
                ((bounds.tr().x - origin.x - row_offset + half) / self.line_dist).floor() as i32;
            (q_min..=q_max).map(move |q| (q, row))
        })
    }
}

/// Axial directions in the order of [`HexCell::corners`] edges: right, upper right, upper left, left, lower left, lower right.
const NEIGHBOR_DIRECTIONS: [(i32, i32); 6] = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];
