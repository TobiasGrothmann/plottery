#[cfg(test)]
mod tests {
    use plottery_lib::{Rect, V2};

    use crate::util::preview_coords::preview_click_to_layer;

    fn close(a: V2, b: V2) -> bool {
        (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3
    }

    #[test]
    fn corners_map_with_flipped_y_and_letterbox() {
        // 20 x 10 cm content shown in a 400 x 400 px square: scale 20, bars of 100 px top/bottom
        let bbox = Rect::new(V2::new(1.0, 2.0), V2::new(21.0, 12.0));
        let element = V2::new(400.0, 400.0);

        let top_left = preview_click_to_layer(V2::new(0.0, 100.0), element, bbox).unwrap();
        assert!(close(top_left, V2::new(1.0, 12.0)), "{top_left:?}");

        let bottom_right = preview_click_to_layer(V2::new(400.0, 300.0), element, bbox).unwrap();
        assert!(close(bottom_right, V2::new(21.0, 2.0)), "{bottom_right:?}");

        let center = preview_click_to_layer(V2::new(200.0, 200.0), element, bbox).unwrap();
        assert!(close(center, V2::new(11.0, 7.0)), "{center:?}");

        assert!(preview_click_to_layer(V2::new(200.0, 50.0), element, bbox).is_none());
    }
}
