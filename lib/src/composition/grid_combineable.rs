use crate::{composition::grid::Grid, Rect, V2i};

/// Represents the state of a cell in the grid
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellState {
    /// A cell with specific dimensions (either a standalone cell or the bottom-left of a combined cell)
    Cell { width: usize, height: usize },
    /// A cell that is part of a combined cell, with coordinates pointing to the bottom-left cell
    PartOf { x: usize, y: usize },
}

/// A grid structure that allows combining adjacent cells into larger rectangular areas.
///
/// `GridCombineable` wraps a basic `Grid` and adds the ability to combine multiple cells
/// into a single larger cell. This is useful for creating complex layouts where some
/// elements need to span multiple grid cells.
///
/// # Features
/// - Combine multiple cells into a single rectangular area
/// - Query the state and dimensions of combined cells
/// - Iterate over logical cells (where combined cells are visited only once)
/// - Reset combined cells back to individual cells
///
/// # Example
/// ```rust
/// # use plottery_lib::{Grid, Rect, V2};
/// # let bounding_box = Rect::new(V2::new(0.0, 0.0), V2::new(100.0, 100.0));
/// let grid = Grid::new(bounding_box, 5, 5, 2.0);
/// let mut combineable_grid = grid.to_combineable();
///
/// // Combine a 2x2 area of cells
/// combineable_grid.set_size(0, 0, 2, 2).unwrap();
///
/// // Get the rectangle for the combined cell
/// let rect = combineable_grid.cell(0, 0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GridCombineable {
    grid: Grid,
    cell_states: Vec<Vec<CellState>>,
}

impl GridCombineable {
    pub fn new_from_grid(grid: Grid) -> Self {
        let single = CellState::Cell {
            width: 1,
            height: 1,
        };
        let cell_states = vec![vec![single; grid.cols]; grid.rows];
        GridCombineable { grid, cell_states }
    }

    /// Checks if cells starting at (x, y) can be combined into a cell of size (width, height)
    pub fn can_combine(&self, x: usize, y: usize, width: usize, height: usize) -> bool {
        x + width <= self.grid.cols
            && y + height <= self.grid.rows
            && self.cell_states[y..y + height].iter().all(|row| {
                row[x..x + width].iter().all(|state| {
                    *state
                        == CellState::Cell {
                            width: 1,
                            height: 1,
                        }
                })
            })
    }

    /// Sets the size of a cell starting at (x, y) to the given width and height
    pub fn set_size(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<(), &'static str> {
        if !self.can_combine(x, y, width, height) {
            return Err("Cannot combine cells: out of bounds or cells already combined");
        }
        for row in y..(y + height) {
            for col in x..(x + width) {
                self.cell_states[row][col] = CellState::PartOf { x, y };
            }
        }
        self.cell_states[y][x] = CellState::Cell { width, height };
        Ok(())
    }

    /// Bottom left position and size of the possibly combined cell that covers (x, y).
    fn main_cell(&self, x: usize, y: usize) -> (usize, usize, usize, usize) {
        let (main_x, main_y) = self.find_main_cell(x, y);
        match self.cell_states[main_y][main_x] {
            CellState::Cell { width, height } => (main_x, main_y, width, height),
            // `set_size` only points `PartOf` at a `Cell`
            CellState::PartOf { .. } => (main_x, main_y, 1, 1),
        }
    }

    /// Gets the rectangle for a cell, taking into account cell combinations
    pub fn cell(&self, x: usize, y: usize) -> Rect {
        let (x, y, width, height) = self.main_cell(x, y);
        let top_right = self.grid.cell(x + width - 1, y + height - 1);
        Rect::new(self.grid.cell(x, y).bl(), top_right.tr())
    }

    /// Get the underlying grid
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Get the cell state at a specific position
    pub fn cell_state(&self, x: usize, y: usize) -> Option<&CellState> {
        self.cell_states.get(y)?.get(x)
    }

    /// Reset a combined cell back to individual cells
    pub fn reset_cell(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
        let (main_x, main_y, width, height) = self.main_cell(x, y);
        if width == 1 && height == 1 {
            return Err("Cell is not part of a combined area");
        }
        for row in main_y..(main_y + height) {
            for col in main_x..(main_x + width) {
                self.cell_states[row][col] = CellState::Cell {
                    width: 1,
                    height: 1,
                };
            }
        }
        Ok(())
    }

    /// Find the main cell coordinates for any position
    pub fn find_main_cell(&self, x: usize, y: usize) -> (usize, usize) {
        match self.cell_states[y][x] {
            CellState::Cell { .. } => (x, y),
            CellState::PartOf { x, y } => (x, y),
        }
    }

    /// Check if a position is a main cell (bottom-left of a possibly combined cell)
    pub fn is_main_cell(&self, x: usize, y: usize) -> bool {
        matches!(self.cell_states[y][x], CellState::Cell { .. })
    }

    /// Returns an iterator that yields information about each cell
    /// For combined cells, only the main (bottom-left) cell is visited
    pub fn iter(&self) -> GridCombineableIterator<'_> {
        GridCombineableIterator {
            grid_combineable: self,
            index: 0,
        }
    }

    pub fn num_cells(&self) -> V2i {
        self.grid.num_cells()
    }
}

/// Contains information about a cell in the grid
#[derive(Debug, Clone)]
pub struct CellInfo {
    /// The rectangle representing the cell
    pub cell: Rect,
    /// The position of the cell (column, row)
    pub position: V2i,
    /// The size of the cell (width, height)
    pub size: V2i,
}

/// Iterator for GridCombineable that visits each logical cell once
pub struct GridCombineableIterator<'a> {
    grid_combineable: &'a GridCombineable,
    /// Row-major position, bottom row first; a combined cell's bottom left comes before its other cells.
    index: usize,
}

impl Iterator for GridCombineableIterator<'_> {
    type Item = CellInfo;

    fn next(&mut self) -> Option<Self::Item> {
        let grid = &self.grid_combineable.grid;
        while self.index < grid.cols * grid.rows {
            let (x, y) = (self.index % grid.cols, self.index / grid.cols);
            self.index += 1;
            if let CellState::Cell { width, height } = self.grid_combineable.cell_states[y][x] {
                return Some(CellInfo {
                    cell: self.grid_combineable.cell(x, y),
                    position: V2i::new(x as i32, y as i32),
                    size: V2i::new(width as i32, height as i32),
                });
            }
        }
        None
    }
}
