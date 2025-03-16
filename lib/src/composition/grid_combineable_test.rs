#[cfg(test)]
mod test_grid_combineable {
    use crate::{
        composition::{
            grid::Grid,
            grid_comineable::{CellState, GridCombineable},
        },
        Rect, V2,
    };

    fn create_test_grid() -> Grid {
        Grid::new(
            Rect::new(V2::new(0.0, 0.0), V2::new(100.0, 100.0)),
            5,
            5,
            2.0,
        )
    }

    #[test]
    fn test_initialization() {
        let grid = create_test_grid();
        let combineable = GridCombineable::new_from_grid(grid);

        for y in 0..5 {
            for x in 0..5 {
                match combineable.get_cell_state(x, y).unwrap() {
                    CellState::Cell { width, height } => {
                        assert_eq!(*width, 1);
                        assert_eq!(*height, 1);
                    }
                    _ => panic!("Expected all cells to be initial Cell state"),
                }
            }
        }
    }

    #[test]
    fn test_can_combine() {
        let grid = create_test_grid();
        let mut combineable = GridCombineable::new_from_grid(grid);

        // Valid combinations
        assert!(combineable.can_combine(0, 0, 2, 2));
        assert!(combineable.can_combine(3, 3, 2, 2));

        // Invalid combinations (out of bounds)
        assert!(!combineable.can_combine(4, 4, 2, 2));
        assert!(!combineable.can_combine(0, 0, 6, 2));

        // Create a combination and test overlap
        assert!(combineable.set_size(0, 0, 2, 2).is_ok());
        assert!(!combineable.can_combine(1, 1, 2, 2));
        assert!(!combineable.can_combine(1, 1, 1, 1));
        assert!(combineable.can_combine(2, 2, 2, 2));
    }

    #[test]
    fn test_set_size() {
        let grid = create_test_grid();
        let mut combineable = GridCombineable::new_from_grid(grid);

        // Combine a 2x2 cell at (0,0)
        assert!(combineable.set_size(0, 0, 2, 2).is_ok());

        // Check states after combination
        match combineable.get_cell_state(0, 0).unwrap() {
            CellState::Cell { width, height } => {
                assert_eq!(*width, 2);
                assert_eq!(*height, 2);
            }
            _ => panic!("Expected (0,0) to be a Cell"),
        }

        match combineable.get_cell_state(1, 0).unwrap() {
            CellState::PartOf { x, y } => {
                assert_eq!(*x, 0);
                assert_eq!(*y, 0);
            }
            _ => panic!("Expected (1,0) to be PartOf (0,0)"),
        }

        match combineable.get_cell_state(0, 1).unwrap() {
            CellState::PartOf { x, y } => {
                assert_eq!(*x, 0);
                assert_eq!(*y, 0);
            }
            _ => panic!("Expected (0,1) to be PartOf (0,0)"),
        }

        match combineable.get_cell_state(1, 1).unwrap() {
            CellState::PartOf { x, y } => {
                assert_eq!(*x, 0);
                assert_eq!(*y, 0);
            }
            _ => panic!("Expected (1,1) to be PartOf (0,0)"),
        }

        match combineable.get_cell_state(2, 2).unwrap() {
            CellState::Cell { width, height } => {
                assert_eq!(*width, 1);
                assert_eq!(*height, 1);
            }
            _ => panic!("Expected (2,2) to be Empty"),
        }

        // Try to combine cells that overlap with existing combination (should fail)
        assert!(combineable.set_size(1, 1, 2, 2).is_err());
    }

    #[test]
    fn test_get_cell() {
        let grid = create_test_grid();
        let mut combineable = GridCombineable::new_from_grid(grid);

        // Get original single cell
        let single_cell = combineable.get_cell(2, 2);
        let original_size = single_cell.tr() - single_cell.bl();

        // Combine a 2x2 cell and check its dimensions
        assert!(combineable.set_size(0, 0, 2, 2).is_ok());

        let combined_cell = combineable.get_cell(0, 0);
        let combined_size = combined_cell.tr() - combined_cell.bl();

        // The width and height should be approximately double the original cell
        // (minus one margin that's now internal)
        let expected_width = 2.0 * original_size.x + 2.0; // 2.0 is the margin
        let expected_height = 2.0 * original_size.y + 2.0;

        assert!((combined_size.x - expected_width).abs() < 0.001);
        assert!((combined_size.y - expected_height).abs() < 0.001);

        // Get cell using coordinates of a part cell
        let from_part = combineable.get_cell(1, 1);
        assert_eq!(from_part.bl(), combined_cell.bl());
        assert_eq!(from_part.tr(), combined_cell.tr());
    }

    #[test]
    fn test_reset_cell() {
        let grid = create_test_grid();
        let mut combineable = GridCombineable::new_from_grid(grid);

        // Create a combined cell
        assert!(combineable.set_size(0, 0, 2, 2).is_ok());

        // Reset from the main cell
        assert!(combineable.reset_cell(0, 0).is_ok());

        // Check that all cells are individual now
        for y in 0..2 {
            for x in 0..2 {
                match combineable.get_cell_state(x, y).unwrap() {
                    CellState::Cell { width, height } => {
                        assert_eq!(*width, 1);
                        assert_eq!(*height, 1);
                    }
                    _ => panic!("Expected all cells to be reset to 1x1"),
                }
            }
        }

        // Create another combined cell and reset from a part cell
        assert!(combineable.set_size(2, 2, 2, 2).is_ok());
        assert!(combineable.reset_cell(3, 3).is_ok());

        // Check all cells are individual
        for y in 2..4 {
            for x in 2..4 {
                match combineable.get_cell_state(x, y).unwrap() {
                    CellState::Cell { width, height } => {
                        assert_eq!(*width, 1);
                        assert_eq!(*height, 1);
                    }
                    _ => panic!("Expected all cells to be reset to 1x1"),
                }
            }
        }
    }

    #[test]
    fn test_helper_methods() {
        let grid = create_test_grid();
        let mut combineable = GridCombineable::new_from_grid(grid);

        // Test before combination
        assert!(combineable.is_main_cell(2, 2));
        assert_eq!(combineable.find_main_cell(2, 2), (2, 2));

        // Create a combined cell
        assert!(combineable.set_size(0, 0, 2, 2).is_ok());

        // Test is_main_cell
        assert!(combineable.is_main_cell(0, 0));
        assert!(!combineable.is_main_cell(1, 0));
        assert!(!combineable.is_main_cell(0, 1));
        assert!(!combineable.is_main_cell(1, 1));

        // Test find_main_cell
        assert_eq!(combineable.find_main_cell(0, 0), (0, 0));
        assert_eq!(combineable.find_main_cell(1, 0), (0, 0));
        assert_eq!(combineable.find_main_cell(0, 1), (0, 0));
        assert_eq!(combineable.find_main_cell(1, 1), (0, 0));
    }

    #[test]
    fn test_complex_combinations() {
        let grid = create_test_grid();
        let mut combineable = GridCombineable::new_from_grid(grid);

        // Create multiple combined cells with different sizes
        assert!(combineable.set_size(0, 0, 2, 2).is_ok());
        assert!(combineable.set_size(2, 0, 3, 1).is_ok());
        assert!(combineable.set_size(0, 2, 1, 3).is_ok());
        assert!(combineable.set_size(2, 2, 2, 2).is_ok());

        // Check if all combinations were properly set
        assert_eq!(
            combineable.get_cell(0, 0).width(),
            combineable.get_cell(1, 1).width()
        );
        assert_eq!(
            combineable.get_cell(2, 0).width(),
            combineable.get_cell(4, 0).width()
        );
        assert_eq!(
            combineable.get_cell(0, 2).height(),
            combineable.get_cell(0, 4).height()
        );

        // Check that cells outside combinations remain individual
        assert_eq!(combineable.find_main_cell(4, 2), (4, 2));
        assert_eq!(combineable.find_main_cell(4, 4), (4, 4));
    }

    #[test]
    fn test_iterator() {
        let grid = create_test_grid();
        let mut combineable = GridCombineable::new_from_grid(grid);

        // Create various combined cells
        assert!(combineable.set_size(0, 0, 2, 2).is_ok());
        assert!(combineable.set_size(3, 1, 2, 2).is_ok());

        // Count the cells we expect to visit
        let mut expected_cells = 0;

        // 2x2 cell at (0,0)
        expected_cells += 1;

        // 2x2 cell at (3,1)
        expected_cells += 1;

        // Remaining individual cells: total cells minus combined cells
        expected_cells += 5 * 5 - (2 * 2) - (2 * 2);

        // Count the cells the iterator actually visits
        let visited_count = combineable.iter().count();

        assert_eq!(visited_count, expected_cells);

        // Check that only main cells are visited for combined areas
        let mut main_cells_count = 0;
        for cell_info in combineable.iter() {
            if (cell_info.position.x == 0 && cell_info.position.y == 0)
                || (cell_info.position.x == 3 && cell_info.position.y == 1)
            {
                main_cells_count += 1;

                // Verify size is correct
                assert_eq!(cell_info.size.x, 2);
                assert_eq!(cell_info.size.y, 2);
            }
        }
        assert_eq!(main_cells_count, 2);

        // Verify that non-main parts of combined cells aren't visited
        let mut part_cells_count = 0;
        for cell_info in combineable.iter() {
            #[allow(clippy::nonminimal_bool)]
            if !(cell_info.position.x != 1 && cell_info.position.y != 1
                || cell_info.position.y != 0 && cell_info.position.y != 1
                || cell_info.position.x != 1
                    && cell_info.position.x != 0
                    && cell_info.position.x != 4)
            {
                part_cells_count += 1;
            }
        }
        assert_eq!(part_cells_count, 0);

        // Test that the cell rectangles are correct
        for cell_info in combineable.iter() {
            // For combined cells, verify the rectangle spans the correct area
            if cell_info.size.x > 1 || cell_info.size.y > 1 {
                let expected_rect = combineable
                    .get_cell(cell_info.position.x as usize, cell_info.position.y as usize);
                assert_eq!(cell_info.cell, expected_rect);
            }
        }
    }
}
