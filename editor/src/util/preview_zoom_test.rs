#[cfg(test)]
mod tests {
    use plottery_lib::V2;

    use crate::util::preview_zoom::PreviewZoom;

    fn close(a: V2, b: V2) -> bool {
        (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3
    }

    #[test]
    fn relative_zoom_keeps_a_stage_picture_on_the_paper() {
        let paper = V2::new(600.0, 800.0);
        let rendered = PreviewZoom {
            scale: 4.0,
            pan: V2::new(120.0, -80.0),
        };
        let current = PreviewZoom {
            scale: 10.0,
            pan: V2::new(-300.0, 45.0),
        };
        let moved = current.relative_to(rendered);

        // Whatever the picture shows at a stage point has to stay on the same paper point.
        for at in [V2::zero(), V2::new(250.0, -130.0), V2::new(-400.0, 310.0)] {
            let before = rendered.stage_to_paper(at, paper);
            let after = current.stage_to_paper(moved.pan + at * moved.scale, paper);
            assert!(close(before, after), "{before:?} != {after:?}");
        }

        assert_eq!(current.relative_to(current), PreviewZoom::default());
    }
}
