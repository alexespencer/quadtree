use crate::{interval::Interval, point::Point, query::Query};
use itertools::Itertools;

/// A region in n-dimensional space defined by a set of intervals.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    intervals: Vec<Interval>,
}

impl Region {
    pub fn new(intervals: Vec<Interval>) -> Self {
        Region { intervals }
    }

    pub fn intervals(&self) -> &Vec<Interval> {
        &self.intervals
    }

    pub fn contains<T: Copy + Into<f64>>(&self, point: &Point<T>) -> bool {
        assert!(
            self.intervals.len() == point.dimensions(),
            "Point dimension {} does not match region dimension {}",
            point.dimensions(),
            self.intervals.len()
        );
        self.intervals
            .iter()
            .zip(point.dimension_values())
            .all(|(interval, value)| interval.contains(*value))
    }

    pub fn subdivide(&self) -> Vec<Vec<Interval>> {
        let iterators = self
            .intervals
            .iter()
            .map(|interval| interval.subdivide())
            .collect::<Vec<_>>();

        iterators
            .iter()
            .map(|v| v.iter().cloned())
            .multi_cartesian_product()
            .map(|product| product.into_iter().collect())
            .collect::<Vec<_>>()
    }

    pub fn intersects(&self, other: &Region) -> bool {
        assert!(
            self.intervals.len() == other.intervals.len(),
            "Regions must have the same number of dimensions"
        );
        self.intervals
            .iter()
            .zip(other.intervals.iter())
            .all(|(a, b)| a.intersects(b))
    }
}

impl Query for Region {
    fn region(&self) -> &Region {
        self
    }

    fn contains<T: Copy + Into<f64>>(&self, point: &Point<T>) -> bool {
        self.contains(point)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{interval::Interval, point::Point, region::Region};

    #[test]
    fn test_region_1d() {
        let single_axis = Interval::try_new(1.0, 5.0).unwrap();
        let region = Region::new(vec![single_axis]);
        assert_eq!(region.intervals().len(), 1);

        let point = Point::new(vec![3]);
        assert!(region.contains(&point));
        let point_outside = Point::new(vec![6]);
        assert!(!region.contains(&point_outside));
    }

    #[test]
    #[should_panic(expected = "Point dimension 2 does not match region dimension 1")]
    fn test_region_1d_panic() {
        let single_axis = Interval::try_new(1.0, 5.0).unwrap();
        let region = Region::new(vec![single_axis]);
        let point = Point::new(vec![3, 4]);
        region.contains(&point);
    }

    #[test]
    fn test_region_2d_subdivide() {
        let x_axis = Interval::try_new(1.0, 5.0).unwrap();
        let y_axis = Interval::try_new(20.0, 60.0).unwrap();
        let region = Region::new(vec![x_axis, y_axis]);
        assert_eq!(region.intervals().len(), 2);

        // Assert there are 4 unique intervals after subdivision
        let subdivided_intervals = region.subdivide();
        assert_eq!(subdivided_intervals.len(), 4);
        for interval in subdivided_intervals.iter() {
            assert_eq!(interval.len(), 2);
        }

        // Assert that the intervals are unique
        let unique_intervals: Vec<_> = subdivided_intervals
            .iter()
            .unique_by(|interval| {
                interval
                    .iter()
                    .map(|i| format!("{} to {}", i.start(), i.end()))
                    .collect::<Vec<String>>()
            })
            .collect();
        assert_eq!(unique_intervals.len(), 4);
    }

    #[test]
    fn test_region_3d_subdivide() {
        let x_axis = Interval::try_new(1.0, 5.0).unwrap();
        let y_axis = Interval::try_new(20.0, 60.0).unwrap();
        let z_axis = Interval::try_new(100.0, 200.0).unwrap();
        let region = Region::new(vec![x_axis, y_axis, z_axis]);

        // Assert there are 8 unique intervals after subdivision
        let subdivided_intervals = region.subdivide();
        assert_eq!(subdivided_intervals.len(), 8);
        for interval in subdivided_intervals.iter() {
            assert_eq!(interval.len(), 3);
        }

        // Assert that the intervals are unique
        let unique_intervals: Vec<_> = subdivided_intervals
            .iter()
            .unique_by(|interval| {
                interval
                    .iter()
                    .map(|i| format!("{} to {}", i.start(), i.end()))
                    .collect::<Vec<String>>()
            })
            .collect();
        assert_eq!(unique_intervals.len(), 8);
    }
}
