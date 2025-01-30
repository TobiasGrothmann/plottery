use crate::{Rect, V2};

pub struct Grid {
    pub rows: usize,
    pub cols: usize,
    pub margin: f32,
    pub bounding_box: Rect,
    cell_size: V2,
}

impl Grid {
    pub fn new(bounding_box: Rect, rows: usize, cols: usize, margin: f32) -> Self {
        assert!(rows > 0, "Number of rows must be greater than 0");
        assert!(cols > 0, "Number of columns must be greater than 0");
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
            rows,
            cols,
            margin,
            bounding_box,
            cell_size,
        }
    }

    pub fn get_rect(&self, row: usize, col: usize) -> Rect {
        let bl = V2::new(self.margin * col as f32, self.margin * row as f32)
            + self.cell_size * V2::new(col as f32, row as f32)
            + self.bounding_box.bl();
        Rect::new(bl, bl + self.cell_size)
    }

    pub fn iter(&self) -> GridIterator {
        GridIterator {
            grid: self,
            current_row: 0,
            current_col: 0,
        }
    }
}

// Iterator struct for Grid
pub struct GridIterator<'a> {
    grid: &'a Grid,
    current_row: usize,
    current_col: usize,
}

impl Iterator for GridIterator<'_> {
    type Item = Rect;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.grid.rows {
            return None;
        }
        let rect = self.grid.get_rect(self.current_row, self.current_col);
        self.current_col += 1;
        if self.current_col >= self.grid.cols {
            self.current_col = 0;
            self.current_row += 1;
        }
        Some(rect)
    }
}
