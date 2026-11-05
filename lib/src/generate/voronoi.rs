use std::collections::HashSet;

use itertools::Itertools;
use voronoice::{BoundingBox, ClipBehavior, Point, VoronoiBuilder};

use crate::{Layer, Line, Path, Rect, Shape, V2};

#[derive(Debug, Clone, PartialEq)]
pub struct VoronoiCell {
    pub site: V2,
    /// Corners of the clipped cell, counter-clockwise.
    pub vertices: Vec<V2>,
    /// Indices of the neighboring cells.
    pub neighbors: Vec<usize>,
}

impl VoronoiCell {
    pub fn outline(&self) -> Path {
        let mut path = Path::new_from(self.vertices.clone());
        path.close();
        path
    }

    pub fn area(&self) -> f32 {
        self.signed_area().abs()
    }

    pub fn centroid(&self) -> V2 {
        let area = self.signed_area();
        if area.abs() < 1e-12 {
            return self.site;
        }
        let sum = self
            .vertices
            .iter()
            .circular_tuple_windows()
            .map(|(a, b)| (a + b) * (a.x * b.y - b.x * a.y))
            .sum::<V2>();
        sum / (6.0 * area)
    }

    /// Centroid weighted by `density`, sampled on a grid inside the cell.
    pub fn centroid_weighted<F: Fn(V2) -> f32>(&self, density: F) -> V2 {
        let area = self.area();
        if area < 1e-12 {
            return self.site;
        }
        let step = (area / 40.0).sqrt();
        let (min, max) = self
            .vertices
            .iter()
            .fold((V2::xy(f32::MAX), V2::xy(f32::MIN)), |(min, max), p| {
                (min.min(*p), max.max(*p))
            });
        let mut weight_sum = 0.0;
        let mut weighted = V2::zero();
        let mut y = min.y + step * 0.5;
        while y < max.y {
            let mut x = min.x + step * 0.5;
            while x < max.x {
                let p = V2::new(x, y);
                if self.contains(p) {
                    let w = density(p).max(0.0);
                    weight_sum += w;
                    weighted += p * w;
                }
                x += step;
            }
            y += step;
        }
        if weight_sum <= 0.0 {
            return self.centroid();
        }
        weighted / weight_sum
    }

    fn signed_area(&self) -> f32 {
        self.vertices
            .iter()
            .circular_tuple_windows()
            .map(|(a, b)| a.x * b.y - b.x * a.y)
            .sum::<f32>()
            * 0.5
    }

    fn contains(&self, p: V2) -> bool {
        // cells are convex, so p is inside if it is on the same side of every edge
        let sign = self.signed_area().signum();
        self.vertices
            .iter()
            .circular_tuple_windows()
            .all(|(a, b)| ((b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)) * sign >= 0.0)
    }
}

/// Voronoi diagram of a set of sites, clipped to `bounds`. Sites outside of `bounds` are dropped.
#[derive(Debug, Clone, PartialEq)]
pub struct Voronoi {
    pub cells: Vec<VoronoiCell>,
    /// Every edge once.
    pub edges: Vec<Line>,
    pub bounds: Rect,
}

impl Voronoi {
    pub fn new(sites: &[V2], bounds: Rect) -> Self {
        let center = bounds.center();
        let built = VoronoiBuilder::default()
            .set_sites(sites.iter().map(to_point).collect())
            .set_bounding_box(BoundingBox::new(
                to_point(&center),
                bounds.width() as f64,
                bounds.height() as f64,
            ))
            .set_clip_behavior(ClipBehavior::Clip)
            .build();
        let Some(diagram) = built else {
            return Self {
                cells: vec![],
                edges: vec![],
                bounds,
            };
        };

        let vertices: Vec<V2> = diagram.vertices().iter().map(from_point).collect();
        let cells = diagram
            .iter_cells()
            .map(|cell| VoronoiCell {
                site: from_point(cell.site_position()),
                vertices: cell.iter_vertices().map(from_point).collect(),
                neighbors: cell.iter_neighbors().collect(),
            })
            .collect();

        let mut seen = HashSet::new();
        let mut edges = vec![];
        for cell in diagram.cells() {
            for (&a, &b) in cell.iter().circular_tuple_windows() {
                if seen.insert((a.min(b), a.max(b))) {
                    edges.push(Line::new(vertices[a], vertices[b]));
                }
            }
        }

        Self {
            cells,
            edges,
            bounds,
        }
    }

    pub fn sites(&self) -> Vec<V2> {
        self.cells.iter().map(|cell| cell.site).collect()
    }

    pub fn edges_layer(&self) -> Layer {
        self.edges
            .iter()
            .map(|line| Path::new_shape_from(vec![line.from, line.to]))
            .collect()
    }

    pub fn cells_layer(&self) -> Layer {
        self.cells
            .iter()
            .map(|cell| Shape::Path(cell.outline()))
            .collect()
    }

    /// Lloyd relaxation, moves every site to the centroid of its cell.
    pub fn relax(&self, iterations: usize) -> Self {
        self.relax_by(iterations, |cell| cell.centroid())
    }

    /// Lloyd relaxation weighted by `density`, sites end up denser where `density` is high.
    pub fn relax_weighted<F: Fn(V2) -> f32>(&self, iterations: usize, density: F) -> Self {
        self.relax_by(iterations, |cell| cell.centroid_weighted(&density))
    }

    fn relax_by<F: Fn(&VoronoiCell) -> V2>(&self, iterations: usize, target: F) -> Self {
        let mut voronoi = self.clone();
        for _ in 0..iterations {
            let sites: Vec<V2> = voronoi.cells.iter().map(&target).collect();
            voronoi = Voronoi::new(&sites, self.bounds);
        }
        voronoi
    }
}

fn to_point(p: &V2) -> Point {
    Point {
        x: p.x as f64,
        y: p.y as f64,
    }
}

fn from_point(p: &Point) -> V2 {
    V2::new(p.x as f32, p.y as f32)
}
