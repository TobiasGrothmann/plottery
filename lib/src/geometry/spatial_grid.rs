use std::cmp::Ordering;

use rustc_hash::FxHashMap;

use crate::V2;

type Cell = (i64, i64);

#[derive(Debug, Clone)]
struct Entry<T> {
    point: V2,
    value: T,
    // insertion number, breaks distance ties in `nearest`
    order: u64,
}

/// Points with values in square cells for fast neighbour queries; `collect()` picks the cell size from the density.
#[derive(Debug, Clone)]
pub struct SpatialGrid<T> {
    cell_size: f32,
    cells: FxHashMap<Cell, Vec<Entry<T>>>,
    // bounds of all cells ever filled, they limit the search rings of `nearest`
    min_cell: Cell,
    max_cell: Cell,
    len: usize,
    inserted: u64,
}

impl<T> SpatialGrid<T> {
    /// Queries are fastest when `cell_size` is about the query radius or the point spacing.
    pub fn new(cell_size: f32) -> Self {
        assert!(
            cell_size > 0.0,
            "cell_size must be positive, got {cell_size}"
        );
        Self {
            cell_size,
            cells: FxHashMap::default(),
            min_cell: (i64::MAX, i64::MAX),
            max_cell: (i64::MIN, i64::MIN),
            len: 0,
            inserted: 0,
        }
    }

    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn insert(&mut self, point: V2, value: T) {
        let cell = self.cell_of(point);
        self.min_cell = (self.min_cell.0.min(cell.0), self.min_cell.1.min(cell.1));
        self.max_cell = (self.max_cell.0.max(cell.0), self.max_cell.1.max(cell.1));
        self.cells.entry(cell).or_default().push(Entry {
            point,
            value,
            order: self.inserted,
        });
        self.len += 1;
        self.inserted += 1;
    }

    /// Removes one entry with exactly this point and value, returns whether there was one.
    pub fn remove(&mut self, point: V2, value: &T) -> bool
    where
        T: PartialEq,
    {
        let cell = self.cell_of(point);
        let Some(entries) = self.cells.get_mut(&cell) else {
            return false;
        };
        let Some(index) = entries
            .iter()
            .position(|entry| entry.point == point && entry.value == *value)
        else {
            return false;
        };
        entries.remove(index);
        if entries.is_empty() {
            self.cells.remove(&cell);
        }
        self.len -= 1;
        true
    }

    /// Entries at most `radius` away from `point`, in no particular distance order.
    pub fn within(&self, point: V2, radius: f32) -> impl Iterator<Item = (V2, &T)> + '_ {
        let from = self.cell_of(point - V2::xy(radius));
        let to = self.cell_of(point + V2::xy(radius));
        let (x_from, x_to) = (from.0.max(self.min_cell.0), to.0.min(self.max_cell.0));
        let (y_from, y_to) = (from.1.max(self.min_cell.1), to.1.min(self.max_cell.1));
        let radius_squared = radius * radius;

        (x_from..=x_to)
            .flat_map(move |x| (y_from..=y_to).map(move |y| (x, y)))
            .filter_map(|cell| self.cells.get(&cell))
            .flatten()
            .filter(move |entry| entry.point.dist_squared(point) <= radius_squared)
            .map(|entry| (entry.point, &entry.value))
    }

    pub fn any_within(&self, point: V2, radius: f32) -> bool {
        self.within(point, radius).next().is_some()
    }

    /// Closest entry to `point`; on a tie the earliest inserted.
    pub fn nearest(&self, point: V2) -> Option<(V2, &T)> {
        if self.is_empty() {
            return None;
        }
        let center = self.cell_of(point);
        let last_ring = [
            center.0.saturating_sub(self.min_cell.0),
            self.max_cell.0.saturating_sub(center.0),
            center.1.saturating_sub(self.min_cell.1),
            self.max_cell.1.saturating_sub(center.1),
        ]
        .into_iter()
        .max()
        .unwrap_or(0)
        .max(0);

        let mut best: Option<(f32, &Entry<T>)> = None;
        for ring in 0..=last_ring {
            if let Some((best_dist_squared, _)) = best {
                // `point` can sit at the edge of its cell, so ring cells are at least ring - 1 cells away
                let gap = (ring - 1) as f32 * self.cell_size;
                if best_dist_squared < gap * gap {
                    break;
                }
            }
            let ring_best = self
                .ring_cells(center, ring)
                .filter_map(|cell| self.cells.get(&cell))
                .flatten()
                .map(|entry| (entry.point.dist_squared(point), entry))
                .min_by(closer_then_earlier);
            best = best
                .into_iter()
                .chain(ring_best)
                .min_by(closer_then_earlier);
        }
        best.map(|(_, entry)| (entry.point, &entry.value))
    }

    fn cell_of(&self, point: V2) -> Cell {
        (
            (point.x / self.cell_size).floor() as i64,
            (point.y / self.cell_size).floor() as i64,
        )
    }

    /// Cells exactly `ring` steps from `center` (Chebyshev distance) that lie inside the filled bounds.
    fn ring_cells(&self, center: Cell, ring: i64) -> impl Iterator<Item = Cell> {
        let (min, max) = (self.min_cell, self.max_cell);
        let low = (center.0.saturating_sub(ring), center.1.saturating_sub(ring));
        let high = (center.0.saturating_add(ring), center.1.saturating_add(ring));
        let both_sides = |a: i64, b: i64| std::iter::once(a).chain((a != b).then_some(b));

        let row_xs = low.0.max(min.0)..=high.0.min(max.0);
        let rows = both_sides(low.1, high.1)
            .filter(move |y| (min.1..=max.1).contains(y))
            .flat_map(move |y| row_xs.clone().map(move |x| (x, y)));

        let column_ys = low.1.saturating_add(1).max(min.1)..=high.1.saturating_sub(1).min(max.1);
        let columns = both_sides(low.0, high.0)
            .filter(move |x| (min.0..=max.0).contains(x))
            .flat_map(move |x| column_ys.clone().map(move |y| (x, y)));

        rows.chain(columns)
    }
}

impl<T> FromIterator<(V2, T)> for SpatialGrid<T> {
    fn from_iter<I: IntoIterator<Item = (V2, T)>>(iter: I) -> Self {
        let entries: Vec<(V2, T)> = iter.into_iter().collect();
        let mut grid = Self::new(mean_spacing(&entries));
        grid.extend(entries);
        grid
    }
}

impl<T> Extend<(V2, T)> for SpatialGrid<T> {
    fn extend<I: IntoIterator<Item = (V2, T)>>(&mut self, iter: I) {
        iter.into_iter()
            .for_each(|(point, value)| self.insert(point, value));
    }
}

fn closer_then_earlier<T>(a: &(f32, &Entry<T>), b: &(f32, &Entry<T>)) -> Ordering {
    a.0.total_cmp(&b.0).then(a.1.order.cmp(&b.1.order))
}

/// About one point per cell, for points spread over an area or along a line; 1 when there is no spread.
fn mean_spacing<T>(entries: &[(V2, T)]) -> f32 {
    let Some((first, _)) = entries.first() else {
        return 1.0;
    };
    let (min, max) = entries.iter().fold((*first, *first), |(min, max), (p, _)| {
        (min.min(*p), max.max(*p))
    });
    let size = max - min;
    let count = entries.len() as f32;
    let spacing = (size.x * size.y / count)
        .sqrt()
        .max(size.max_axis() / count);
    if spacing.is_finite() && spacing > 0.0 {
        spacing
    } else {
        1.0
    }
}
