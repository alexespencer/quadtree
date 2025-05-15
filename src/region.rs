use crate::{interval::Interval, point::Point};

pub struct Region<T: PartialOrd> {
    intervals: Vec<Interval<T>>,
}

impl<T: PartialOrd> Region<T> {
    pub fn new(intervals: Vec<Interval<T>>) -> Self {
        Region { intervals }
    }

    pub fn intervals(&self) -> &Vec<Interval<T>> {
        &self.intervals
    }

    pub fn contains(&self, point: &Point<T>) -> bool {
        assert!(
            self.intervals.len() == point.dimensions(),
            "Point dimension {} does not match region dimension {}",
            point.dimensions(),
            self.intervals.len()
        );
        self.intervals
            .iter()
            .zip(point.dimension_values())
            .all(|(interval, value)| interval.contains(value))
    }
}

#[cfg(test)]
mod tests {
    use crate::{interval::Interval, point::Point, region::Region};

    #[test]
    fn test_region_1d() {
        let single_axis = Interval::try_new(1, 5).unwrap();
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
        let single_axis = Interval::try_new(1, 5).unwrap();
        let region = Region::new(vec![single_axis]);
        let point = Point::new(vec![3, 4]);
        region.contains(&point);
    }
}
