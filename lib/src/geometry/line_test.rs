#[cfg(test)]
mod test_line {
    #[cfg(test)]
    mod tests {
        use crate::{Line, LineIntersection, PointLineRelation, V2};

        #[test]
        fn point_relation() {
            let line = Line::new(V2::new(0.0, 0.0), V2::new(5.0, 5.0));

            let point_on_line = V2::new(3.0, 3.0);
            assert_eq!(
                line.point_relation(point_on_line),
                PointLineRelation::OnLine
            );

            let point_left = V2::new(-1.0, 2.0);
            assert_eq!(line.point_relation(point_left), PointLineRelation::Left);

            let point_right = V2::new(6.0, 3.0);
            assert_eq!(line.point_relation(point_right), PointLineRelation::Right);
        }

        #[test]
        fn intersection() {
            // non-parallel lines with intersection
            let line1 = Line::new(V2::new(0.0, 0.0), V2::new(5.0, 5.0));
            let line2 = Line::new(V2::new(0.0, 5.0), V2::new(5.0, 0.0));
            assert_eq!(
                line1.intersection(line2),
                LineIntersection::Intersection(V2 { x: 2.5, y: 2.5 })
            );
        }

        #[test]
        fn intersection_colinear() {
            // colinear lines with no intersection
            let line1 = Line::new(V2::new(0.0, 0.0), V2::new(5.0, 5.0));
            let line2 = Line::new(V2::new(6.0, 6.0), V2::new(8.0, 8.0));
            assert_eq!(line1.intersection(line2), LineIntersection::NoIntersection);
        }

        #[test]
        fn intersection_lines_intersect_outside() {
            // lines without intersection
            let line1 = Line::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
            let line2 = Line::new(V2::new(3.5, 0.0), V2::new(0.0, 3.5));
            assert_eq!(line1.intersection(line2), LineIntersection::NoIntersection);

            let line1 = Line::new(V2::new(0.0, 0.0), V2::new(1.0, 0.0));
            let line2 = Line::new(V2::new(1.0, -1.0), V2::new(3.0, 1.0));
            assert_eq!(line1.intersection(line2), LineIntersection::NoIntersection);

            let line1 = Line::new(V2::new(5.0, 5.0), V2::new(5.0, -3.0));
            let line2 = Line::new(V2::new(4.0, -7.0), V2::new(6.0, -5.0));
            assert_eq!(line1.intersection(line2), LineIntersection::NoIntersection);
        }

        #[test]
        fn intersection_lines_end_to_end() {
            // lines end to end
            let line1 = Line::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
            let line2 = Line::new(V2::new(1.0, 1.0), V2::new(2.0, 2.0));
            assert_eq!(
                line1.intersection(line2),
                LineIntersection::Intersection(line1.to)
            );
        }

        #[test]
        fn intersection_lines_t() {
            // lines in a T formation
            let line1 = Line::new(V2::new(0.0, 0.0), V2::new(1.0, 0.0));
            let line2 = Line::new(V2::new(1.0, 1.0), V2::new(1.0, -1.0));
            assert_eq!(
                line1.intersection(line2),
                LineIntersection::Intersection(line1.to)
            );
        }

        #[test]
        fn intersection_lines_t_2() {
            // lines in a diagonal T formation
            let line1 = Line::new(V2::new(1.0, 1.0), V2::new(0.0, 0.0));
            let line2 = Line::new(V2::new(0.0, 2.0), V2::new(2.0, 0.0));
            assert_eq!(
                line1.intersection(line2),
                LineIntersection::Intersection(line1.from)
            );
        }

        #[test]
        fn intersection_inf_lines() {
            // non-parallel lines with intersection
            let line1 = Line::new(V2::xy(0.0), V2::xy(1.0));
            let line2 = Line::new(V2::new(0.0, 5.0), V2::new(1.0, 4.0));
            assert_eq!(
                line1.intersection_as_inf_lines(&line2),
                LineIntersection::Intersection(V2 { x: 2.5, y: 2.5 })
            );
        }

        #[test]
        fn intersection_inf_lines_parallel() {
            // parallel lines
            let line1 = Line::new(V2::xy(0.0), V2::xy(1.0));
            let line2 = Line::new(V2::new(0.0, -1.0), V2::new(1.0, 0.0));
            assert_eq!(
                line1.intersection_as_inf_lines(&line2),
                LineIntersection::NoIntersection
            );
        }

        #[test]
        fn intersection_inf_lines_colinear() {
            // colinear lines
            let line1 = Line::new(V2::xy(0.0), V2::xy(1.0));
            let line2 = Line::new(V2::xy(5.0), V2::xy(4.0));
            assert_eq!(
                line1.intersection_as_inf_lines(&line2),
                LineIntersection::NoIntersection
            );
        }

        #[test]
        fn closest_point() {
            let line = Line::new(V2::new(0.0, 0.0), V2::new(3.0, 3.0));

            let point = V2::new(3.0, 3.0);
            assert_eq!(line.closest_point(point), point);

            let point = V2::new(5.0, 5.0);
            assert_eq!(line.closest_point(point), line.to.clone());

            let point = V2::new(1.5, 1.5);
            assert_eq!(line.closest_point(point), point);

            let point = V2::new(0.0, 3.0);
            assert_eq!(line.closest_point(point), V2::new(1.5, 1.5));

            let point = V2::new(3.0, 0.0);
            assert_eq!(line.closest_point(point), V2::new(1.5, 1.5));

            let point = V2::new(100.0, 0.0);
            assert_eq!(line.closest_point(point), V2::new(3.0, 3.0));

            let point = V2::new(0.0, 5.0);
            assert_eq!(line.closest_point(point), V2::new(2.5, 2.5));

            let point = V2::new(5.0, 0.0);
            assert_eq!(line.closest_point(point), V2::new(2.5, 2.5));
        }

        #[test]
        fn closest_point_line_points_coincident() {
            // special case where line starts and ends at the same point
            let line = Line::new(V2::new(0.0, 0.0), V2::new(0.0, 0.0));

            let point = V2::new(1.0, 1.0);
            assert_eq!(line.closest_point(point), V2::new(0.0, 0.0));
        }

        #[test]
        fn project() {
            let line = Line::new(V2::new(0.0, 1.0), V2::new(1.0, 2.0));

            let point = V2::new(0.0, 2.0);
            assert_eq!(line.project(point), V2::new(0.5, 1.5));
        }

        #[test]
        fn intersect_multiple_sorted() {
            let a = Line::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));

            let i1 = Line::new(V2::new(0.0, 0.25), V2::new(1.0, 0.25));
            let i2 = Line::new(V2::new(0.0, 0.5), V2::new(1.0, 0.5));
            let i3 = Line::new(V2::new(0.0, 0.75), V2::new(1.0, 0.75));

            let order1 = vec![i1, i2, i3];
            let order2 = vec![i1, i3, i2];
            let order3 = vec![i2, i1, i3];
            let order4 = vec![i2, i3, i1];
            let order5 = vec![i3, i1, i2];
            let order6 = vec![i3, i2, i1];

            for order in [order1, order2, order3, order4, order5, order6] {
                let r = a.intersect_multiple_sorted_by_dist(&order);
                assert_eq!(r.len(), 3);
                assert_eq!(r[0], V2::xy(0.25));
                assert_eq!(r[1], V2::xy(0.5));
                assert_eq!(r[2], V2::xy(0.75));
            }
        }
    }
}
