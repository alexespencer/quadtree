use itertools::Itertools;

use crate::{interval::Interval, point::Point, region::Region};

/// [Query] is trait that allows for querying a region in n-dimensional space.
/// This is trivial for the existing [Region] struct, but can be extended for other types of queries.
///
/// For example, [DistanceQuery] is implemented by creating a new struct that implements this trait.
/// The region should be the bounding box of the 'circle' (or sphere, or n-dimensional shape) and the
/// `contains` method would check if the point is within the circle.
pub trait Query<const N: usize> {
    fn region(&self) -> &Region<N>;
    fn contains(&self, point: &Point<N>) -> bool;
}

pub struct DistanceQuery<const N: usize> {
    center: Point<N>,
    radius: f64,
    region: Region<N>,
}

impl<const N: usize> DistanceQuery<N> {
    pub fn new(center: Point<N>, radius: f64) -> Self {
        let center_f64 = center.dimension_values();
        let intervals = center_f64
            .iter()
            .map(|&c| Interval::try_new(c - radius, c + radius).unwrap())
            .collect_array()
            .expect("same sized array");
        let region = Region::new(&intervals);
        DistanceQuery {
            center,
            radius,
            region,
        }
    }
}

impl<const N: usize> Query<N> for DistanceQuery<N> {
    fn region(&self) -> &Region<N> {
        &self.region
    }

    fn contains(&self, point: &Point<N>) -> bool {
        let distance = self.center.distance(point);
        distance <= self.radius
    }
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

    pub struct TestStruct(Point<2>);
    impl Storable<TestStruct, 2> for TestStruct {
        fn point(&self) -> &Point<2> {
            &self.0
        }

        fn item(&self) -> &Self {
            self
        }
    }

    #[test]
    fn test_circle_query() {
        let center = Point::new(&[5.0, 5.0]);
        let radius = 3.0;
        let circle_query = DistanceQuery::new(center.clone(), radius);

        assert_eq!(
            circle_query.region(),
            &Region::new(&[
                Interval::try_new(2.0, 8.0).unwrap(),
                Interval::try_new(2.0, 8.0).unwrap(),
            ])
        );

        let point_inside = Point::new(&[6.0, 6.0]);
        assert!(circle_query.contains(&point_inside));

        let point_outside = Point::new(&[9.0, 9.0]);
        assert!(!circle_query.contains(&point_outside));

        // Test with quadtree (we can assert the value is almost PI!)
        let region = Region::new(&[
            Interval::try_new(0.0, 100.0).unwrap(),
            Interval::try_new(0.0, 100.0).unwrap(),
        ]);

        // Capacity of 100 will ensure lots of subdivision when inserting 100,000 items
        let mut quadtree = QuadTree::new(&region, NonZero::new(100).unwrap());
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);

        for _ in 0..100_000 {
            // Choose random x and y coordinates
            let x = rng.random_range(0..100);
            let y = rng.random_range(0..100);
            quadtree.insert(TestStruct(Point::new(&[x, y]))).unwrap();
        }

        // Construct query region
        let square_region = Region::new(&[
            Interval::try_new(0.0, 100.0).unwrap(),
            Interval::try_new(0.0, 100.0).unwrap(),
        ]);
        let square_count = quadtree.query(&square_region).collect::<Vec<_>>().len();

        // Construct circle query
        let circle_query = DistanceQuery::new(Point::new(&[50.0, 50.0]), 50.0);
        let circle_count = quadtree.query(&circle_query).collect::<Vec<_>>().len();
        assert_ne!(circle_count, square_count);
        assert_abs_diff_eq!(
            circle_count as f64 / square_count as f64,
            std::f64::consts::PI / 4.0,
            epsilon = 0.01
        );
    }
}
