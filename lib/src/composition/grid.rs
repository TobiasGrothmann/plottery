use crate::{Rect, V2i, V2};

use super::grid_combineable::GridCombineable;

/// A 2D grid layout that divides a rectangular area into equal-sized cells with a margin between them.
///
/// Cells are addressed by `(x, y)`, counting from the bottom left.
///
/// ### Example
/// ```rust
/// # use plottery_lib::{Rect, V2, composition::grid::Grid};
/// # let bounding_box = Rect::new(V2::new(0.0, 0.0), V2::new(100.0, 100.0));
/// // 4 columns, 3 rows, 2.0 margin between cells
/// let grid = Grid::new(bounding_box, 4, 3, 2.0);
///
/// let rect = grid.cell(1, 2);
///
/// for cell in grid.cells() {
///     let checkered = (cell.x + cell.y) % 2 == 0;
///     // cell.rect ...
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grid {
    /// Number of cells in x.
    pub cols: usize,
    /// Number of cells in y.
    pub rows: usize,
    pub margin: f32,
    pub bounding_box: Rect,
    cell_size: V2,
}

/// One cell of a [`Grid`] with its index.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridCell {
    pub x: usize,
    pub y: usize,
    pub rect: Rect,
}

impl Grid {
    /// Creates a grid of `cols` × `rows` cells filling `bounding_box`.
    pub fn new(bounding_box: Rect, cols: usize, rows: usize, margin: f32) -> Self {
        assert!(cols > 0, "Number of columns must be greater than 0");
        assert!(rows > 0, "Number of rows must be greater than 0");
        let cell_size = V2::new(
            (bounding_box.width() - (cols - 1) as f32 * margin) / cols as f32,
            (bounding_box.height() - (rows - 1) as f32 * margin) / rows as f32,
        );
        assert!(
            cell_size.x > 0.0,
            "Cell width must be greater than 0. Select a larger bounding box or smaller margin."
        );
        assert!(
            cell_size.y > 0.0,
            "Cell height must be greater than 0. Select a larger bounding box or smaller margin."
        );
        Self {
            cols,
            rows,
            margin,
            bounding_box,
            cell_size,
        }
    }

    /// Creates a grid of square cells, shrinking the bounding box around its center as needed.
    pub fn new_square_cells(bounding_box: Rect, cols: usize, rows: usize, margin: f32) -> Self {
        let side = Self::new(bounding_box, cols, rows, margin)
            .cell_size
            .min_axis();
        Self::fitted(bounding_box, V2::xy(side), cols, rows, margin)
    }

    /// Creates a grid with as many square cells of `cell_size` as fit, centered in `bounding_box`.
    pub fn with_cell_size(bounding_box: Rect, cell_size: f32, margin: f32) -> Self {
        let fit =
            |length: f32| (((length + margin) / (cell_size + margin)).floor() as usize).max(1);
        let cols = fit(bounding_box.width());
        let rows = fit(bounding_box.height());
        Self::fitted(bounding_box, V2::xy(cell_size), cols, rows, margin)
    }

    /// Grid with exactly `cell_size` cells, centered in `bounding_box`.
    fn fitted(bounding_box: Rect, cell_size: V2, cols: usize, rows: usize, margin: f32) -> Self {
        let size = cell_size * V2::new(cols as f32, rows as f32)
            + V2::new((cols - 1) as f32, (rows - 1) as f32) * margin;
        let bl = bounding_box.center() - size * 0.5;
        Self::new(Rect::new(bl, bl + size), cols, rows, margin)
    }

    /// The rectangle of the cell at `(x, y)`.
    pub fn cell(&self, x: usize, y: usize) -> Rect {
        let index = V2::new(x as f32, y as f32);
        let bl = self.bounding_box.bl() + (self.cell_size + V2::xy(self.margin)) * index;
        Rect::new(bl, bl + self.cell_size)
    }

    pub fn cell_size(&self) -> V2 {
        self.cell_size
    }

    /// Number of cells as `(cols, rows)`.
    pub fn num_cells(&self) -> V2i {
        V2i::new(self.cols as i32, self.rows as i32)
    }

    /// All cell rectangles, row by row from the bottom left.
    pub fn iter(&self) -> impl Iterator<Item = Rect> + '_ {
        self.cells().map(|cell| cell.rect)
    }

    /// All cells with their `(x, y)` index, row by row from the bottom left.
    pub fn cells(&self) -> impl Iterator<Item = GridCell> + '_ {
        (0..self.rows).flat_map(move |y| {
            (0..self.cols).map(move |x| GridCell {
                x,
                y,
                rect: self.cell(x, y),
            })
        })
    }

    /// Converts this grid to a [`GridCombineable`] which allows combining cells.
    pub fn to_combineable(self) -> GridCombineable {
        GridCombineable::new_from_grid(self)
    }
}
