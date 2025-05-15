use eyre::{Result, ensure};

/// Represents an interval with a start and end value.
/// The interval is inclusive of start and exclusive of end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    start: f64,
    end: f64,
}

impl Interval {
    pub fn try_new(start: f64, end: f64) -> Result<Self> {
        ensure!(start <= end, "Start must be less than or equal to end");
        ensure!(start.is_finite(), "Start must be finite");
        ensure!(end.is_finite(), "End must be finite");
        Ok(Interval { start, end })
    }

    pub fn start(&self) -> &f64 {
        &self.start
    }

    pub fn end(&self) -> &f64 {
        &self.end
    }

    pub fn contains(&self, value: impl Into<f64>) -> bool {
        let value = value.into();
        self.start <= value && value < self.end
    }

    pub fn subdivide(&self) -> Vec<Self> {
        // Split down the middle
        let midpoint = self.start.midpoint(self.end);
        vec![
            Interval {
                start: self.start,
                end: midpoint,
            },
            Interval {
                start: midpoint,
                end: self.end,
            },
        ]
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval() {
        // Valid interval
        let interval = Interval::try_new(1.0, 5.0).unwrap();
        assert_eq!(*interval.start(), 1.0);
        assert_eq!(*interval.end(), 5.0);
        assert!(interval.contains(2.5));
        assert!(interval.contains(1.0));
        assert!(!interval.contains(0.9));
        assert!(!interval.contains(5.0));

        assert!(interval.contains(1));

        // Invalid interval
        let invalid_interval = Interval::try_new(5.0, 1.0);
        assert!(invalid_interval.is_err());
    }

    #[test]
    fn test_interval_subdivide() {
        let interval = Interval::try_new(1.0, 5.0).unwrap();
        let subdivided = interval.subdivide();
        if let [left, right] = subdivided.as_slice() {
            assert_eq!(*left.start(), 1.0);
            assert_eq!(*left.end(), 3.0);
            assert_eq!(*right.start(), 3.0);
            assert_eq!(*right.end(), 5.0);
        } else {
            panic!("Expected exactly two intervals after subdivision");
        }
    }

    #[test]
    fn test_interval_intersects() {
        let interval_a = Interval::try_new(1.0, 5.0).unwrap();
        let interval_b = Interval::try_new(4.0, 6.0).unwrap();
        let interval_c = Interval::try_new(6.0, 8.0).unwrap();
        let interval_d = Interval::try_new(5.0, 7.0).unwrap();

        assert!(interval_a.intersects(&interval_b));
        assert!(!interval_a.intersects(&interval_c));
        assert!(!interval_d.intersects(&interval_a));
    }
}
