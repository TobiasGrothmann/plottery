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
/// # use plottery_lib::{Rect, V2, composition::grid::Grid};
/// # let bounding_box = Rect::new(V2::new(0.0, 0.0), V2::new(100.0, 100.0));
/// let grid = Grid::new(bounding_box, 5, 5, 2.0);
/// let mut combineable_grid = grid.to_combineable();
///
/// // Combine a 2x2 area of cells
/// combineable_grid.set_size(0, 0, 2, 2).unwrap();
///
/// // Get the rectangle for the combined cell
/// let rect = combineable_grid.get_cell(0, 0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GridCombineable {
    grid: Grid,
    cell_states: Vec<Vec<CellState>>,
}

impl GridCombineable {
    pub fn new_from_grid(grid: Grid) -> Self {
        // Initialize all cells as standard 1x1 cells
        let mut cell_states = Vec::with_capacity(grid.rows);
        for _ in 0..grid.rows {
            let mut row = Vec::with_capacity(grid.cols);
            for _ in 0..grid.cols {
                row.push(CellState::Cell {
                    width: 1,
                    height: 1,
                });
            }
            cell_states.push(row);
        }

        GridCombineable { grid, cell_states }
    }

    /// Checks if cells starting at (x, y) can be combined into a cell of size (width, height)
    pub fn can_combine(&self, x: usize, y: usize, width: usize, height: usize) -> bool {
        // Check bounds
        if x + width > self.grid.cols || y + height > self.grid.rows {
            return false;
        }

        // Check if any cell in the target area is already part of a combined cell
        for row in y..(y + height) {
            for col in x..(x + width) {
                match self.cell_states[row][col] {
                    CellState::PartOf { .. } => return false, // Already part of another cell
                    CellState::Cell {
                        width: w,
                        height: h,
                    } => {
                        // If this is a main cell with size > 1, check if it overlaps our target area
                        if (w > 1 || h > 1)
                            && col + w > x
                            && col < x + width
                            && row + h > y
                            && row < y + height
                        {
                            return false;
                        }
                    }
                }
            }
        }

        true
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

        // Set the bottom-left cell to the new size
        self.cell_states[y][x] = CellState::Cell { width, height };

        // Mark all other cells in the combined area as part of this cell
        for row in y..(y + height) {
            for col in x..(x + width) {
                if row != y || col != x {
                    // Skip the bottom-left cell
                    self.cell_states[row][col] = CellState::PartOf { x, y };
                }
            }
        }

        Ok(())
    }

    /// Gets the rectangle for a cell, taking into account cell combinations
    pub fn get_cell(&self, x: usize, y: usize) -> Rect {
        // Get the bottom-left cell of the combined area
        let (bl_x, bl_y, width, height) = match self.cell_states[y][x] {
            CellState::Cell { width, height } => (x, y, width, height),
            CellState::PartOf {
                x: main_x,
                y: main_y,
            } => match self.cell_states[main_y][main_x] {
                CellState::Cell { width, height } => (main_x, main_y, width, height),
                _ => panic!("Invalid cell state: PartOf points to non-Cell"),
            },
        };

        // Get the bottom-left cell rect
        let bl_rect = self.grid.get_cell(bl_y, bl_x);

        // Calculate the top-right cell rect
        let tr_rect = self.grid.get_cell(bl_y + height - 1, bl_x + width - 1);

        // Create a rect that spans from bl to tr
        Rect::new(bl_rect.bl(), tr_rect.tr())
    }

    /// Get the underlying grid
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Get the cell state at a specific position
    pub fn get_cell_state(&self, x: usize, y: usize) -> Option<&CellState> {
        self.cell_states.get(y)?.get(x)
    }

    /// Reset a combined cell back to individual cells
    pub fn reset_cell(&mut self, x: usize, y: usize) -> Result<(), &'static str> {
        let (main_x, main_y, width, height) = match self.cell_states[y][x] {
            CellState::Cell { width, height } if width > 1 || height > 1 => (x, y, width, height),
            CellState::PartOf {
                x: main_x,
                y: main_y,
            } => match self.cell_states[main_y][main_x] {
                CellState::Cell { width, height } => (main_x, main_y, width, height),
                _ => return Err("Invalid cell state: PartOf points to non-Cell"),
            },
            _ => return Err("Cell is not part of a combined area"),
        };

        // Reset all cells in the combined area to individual cells
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
            CellState::PartOf {
                x: main_x,
                y: main_y,
            } => (main_x, main_y),
        }
    }

    /// Check if a position is a main cell (bottom-left of a possibly combined cell)
    pub fn is_main_cell(&self, x: usize, y: usize) -> bool {
        matches!(self.cell_states[y][x], CellState::Cell { .. })
    }

    /// Returns an iterator that yields information about each cell
    /// For combined cells, only the main (bottom-left) cell is visited
    pub fn iter(&self) -> GridCombineableIterator {
        GridCombineableIterator {
            grid_combineable: self,
            current_row: 0,
            current_col: 0,
            visited: vec![vec![false; self.grid.cols]; self.grid.rows],
        }
    }

    pub fn get_num_cells(&self) -> V2i {
        self.grid.get_num_cells()
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
    current_row: usize,
    current_col: usize,
    visited: Vec<Vec<bool>>, // Tracks visited cells
}

impl Iterator for GridCombineableIterator<'_> {
    type Item = CellInfo;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_row < self.grid_combineable.grid.rows {
            // Skip if we've already visited this cell
            if self.visited[self.current_row][self.current_col] {
                self.advance_position();
                continue;
            }

            // Get the main cell coordinates for the current position
            let (main_x, main_y) = self
                .grid_combineable
                .find_main_cell(self.current_col, self.current_row);

            // If this is a "PartOf" cell and not the main cell, skip it
            if main_x != self.current_col || main_y != self.current_row {
                self.visited[self.current_row][self.current_col] = true;
                self.advance_position();
                continue;
            }

            // Get the width and height of the cell
            let (width, height) = match self.grid_combineable.cell_states[main_y][main_x] {
                CellState::Cell { width, height } => {
                    // Mark all cells in this combined area as visited
                    for r in main_y..(main_y + height) {
                        for c in main_x..(main_x + width) {
                            self.visited[r][c] = true;
                        }
                    }
                    (width, height)
                }
                _ => {
                    self.visited[main_y][main_x] = true;
                    (1, 1)
                }
            };

            let rect = self.grid_combineable.get_cell(main_x, main_y);
            let result = CellInfo {
                cell: rect,
                position: V2i::new(main_x as i32, main_y as i32),
                size: V2i::new(width as i32, height as i32),
            };

            // Advance to the next position
            self.advance_position();

            return Some(result);
        }

        None
    }
}

impl GridCombineableIterator<'_> {
    // Helper method to advance position
    fn advance_position(&mut self) {
        self.current_col += 1;
        if self.current_col >= self.grid_combineable.grid.cols {
            self.current_col = 0;
            self.current_row += 1;
        }
    }
}
