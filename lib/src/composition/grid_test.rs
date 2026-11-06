#[cfg(test)]
mod test_grid {
    use crate::{Grid, Rect, V2};

    #[test]
    fn xy_order_and_cell_size() {
        let grid = Grid::new(Rect::new(V2::zero(), V2::new(40.0, 20.0)), 4, 2, 0.0);
        assert_eq!(grid.cell(3, 0).bl(), V2::new(30.0, 0.0));
        assert_eq!(grid.cell(0, 1).bl(), V2::new(0.0, 10.0));
        let last = grid.cells().last().unwrap();
        assert_eq!((last.x, last.y), (3, 1));
        assert_eq!(grid.cells().count(), 8);

        let fitted = Grid::with_cell_size(Rect::new(V2::zero(), V2::new(10.0, 7.0)), 3.0, 0.5);
        assert_eq!(fitted.num_cells().x, 3);
        assert_eq!(fitted.num_cells().y, 2);
        assert_eq!(fitted.cell_size(), V2::xy(3.0));
        assert_eq!(fitted.bounding_box.center(), V2::new(5.0, 3.5));
    }
}
