use crate::{point::Point, region::Region};

/// A trait for querying a region with a point.
/// This is trivial for the existing [Region] struct,
/// but can be extended for other types of queries.
///
/// For example, a circle query could be implemented
/// by creating a new struct that implements this trait.
/// The region should be the bounding box of the circle
/// and the `contains` method would check if the point
/// is within the circle.
pub trait Query {
    fn region(&self) -> &Region;
    fn contains<T: Copy + Into<f64>>(&self, point: &Point<T>) -> bool;
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use approx::assert_abs_diff_eq;
    use rand::{Rng, SeedableRng};

    use super::*;
    use crate::{
        interval::Interval,
        quadtree::{QuadTree, Storable},
    };

    struct CircleQuery {
        center: Point<f64>,
        radius: f64,
        region: Region,
    }

    impl CircleQuery {
        fn new(center: Point<f64>, radius: f64) -> Self {
            let center_f64 = center.dimension_values();
            let intervals = vec![
                Interval::try_new(center_f64[0] - radius, center_f64[0] + radius).unwrap(),
                Interval::try_new(center_f64[1] - radius, center_f64[1] + radius).unwrap(),
            ];
            let region = Region::new(intervals);
            CircleQuery {
                center,
                radius,
                region,
            }
        }
    }

    impl Query for CircleQuery {
        fn region(&self) -> &Region {
            &self.region
        }

        fn contains<T: Copy + Into<f64>>(&self, point: &Point<T>) -> bool {
            let distance = self.center.distance(&point.to_f64_point());
            distance <= self.radius
        }
    }

    pub struct TestStruct(Point<i32>);
    impl Storable<TestStruct> for TestStruct {
        fn point(&self) -> Point<f64> {
            self.0.to_f64_point()
        }

        fn item(&self) -> &Self {
            self
        }
    }

    #[test]
    fn test_circle_query() {
        let center = Point::new(vec![5.0, 5.0]);
        let radius = 3.0;
        let circle_query = CircleQuery::new(center.clone(), radius);

        assert_eq!(
            circle_query.region(),
            &Region::new(vec![
                Interval::try_new(2.0, 8.0).unwrap(),
                Interval::try_new(2.0, 8.0).unwrap(),
            ])
        );

        let point_inside = Point::new(vec![6.0, 6.0]);
        assert!(circle_query.contains(&point_inside));

        let point_outside = Point::new(vec![9.0, 9.0]);
        assert!(!circle_query.contains(&point_outside));

        // Test with quadtree (we can assert the value is almost PI!)
        let region = Region::new(vec![
            Interval::try_new(0.0, 100.0).unwrap(),
            Interval::try_new(0.0, 100.0).unwrap(),
        ]);

        // Capacity of 100 will ensure lots of subdivision when inserting 100,000 items
        let mut quadtree = QuadTree::new(region, NonZero::new(100).unwrap());
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);

        for _ in 0..100_000 {
            // Choose random x and y coordinates
            let x = rng.random_range(0..100);
            let y = rng.random_range(0..100);
            quadtree.insert(TestStruct(Point::new(vec![x, y]))).unwrap();
        }

        // Construct query region
        let square_region = Region::new(vec![
            Interval::try_new(0.0, 100.0).unwrap(),
            Interval::try_new(0.0, 100.0).unwrap(),
        ]);
        let square_count = quadtree.query(&square_region).collect::<Vec<_>>().len();

        // Construct circle query
        let circle_query = CircleQuery::new(Point::new(vec![50.0, 50.0]), 50.0);
        let circle_count = quadtree.query(&circle_query).collect::<Vec<_>>().len();
        assert_ne!(circle_count, square_count);
        assert_abs_diff_eq!(
            circle_count as f64 / square_count as f64,
            std::f64::consts::PI / 4.0,
            epsilon = 0.01
        );
    }
}
