use eyre::{Result, ensure};
use rand::distr::uniform::SampleRange;

/// Represents an interval with a start and end value.
/// The interval is inclusive of start and exclusive of end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    start: f64,
    end: f64,
}

impl Interval {
    pub fn try_new(start: f64, end: f64) -> Result<Self> {
        ensure!(start < end, "Start must be less to end");
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

    pub fn contains(&self, value: &f64) -> bool {
        self.start <= *value && *value < self.end
    }

    /// Subdivides the Interval at the mid-point
    pub fn subdivide(&self) -> Vec<Self> {
        let midpoint = self.start.midpoint(self.end);
        if self.start == midpoint {
            vec![*self]
        } else {
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
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

impl SampleRange<f64> for Interval {
    fn sample_single<R: rand::RngCore + ?Sized>(
        self,
        rng: &mut R,
    ) -> std::result::Result<f64, rand::distr::uniform::Error> {
        // Generate a random value within the interval
        (self.start..=self.end).sample_single(rng)
    }

    fn is_empty(&self) -> bool {
        // By construction, an Interval cannot be empty
        false
    }
}

#[cfg(test)]
mod tests {
    use core::f64;

    use super::*;

    #[test]
    fn test_interval() {
        // Valid interval
        let interval = Interval::try_new(1.0, 5.0).unwrap();
        assert_eq!(*interval.start(), 1.0);
        assert_eq!(*interval.end(), 5.0);
        assert!(interval.contains(&2.5));
        assert!(interval.contains(&1.0));
        assert!(!interval.contains(&0.9));
        assert!(!interval.contains(&5.0));

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
    fn test_interval_subdivide_epsilon() {
        // Trying to subdivide something that cannot be divided should only yield itself
        let interval = Interval::try_new(1.0, 1.0 + f64::EPSILON).unwrap();
        let subdivided = interval.subdivide();
        assert_eq!(subdivided.len(), 1);
        let first = subdivided.first().unwrap();
        assert_eq!(first, &interval);
    }

    #[test]
    fn test_interval_intersects() {
        let interval_a = Interval::try_new(1.0, 5.0).unwrap();
        // b intersects with a
        let interval_b = Interval::try_new(4.0, 6.0).unwrap();
        // c should not intersect with b's end
        let interval_c = Interval::try_new(6.0, 8.0).unwrap();
        // d should not intersect with a
        let interval_d = Interval::try_new(5.0, 7.0).unwrap();

        assert!(interval_a.intersects(&interval_b));
        assert!(!interval_a.intersects(&interval_c));
        assert!(!interval_b.intersects(&interval_c));
        assert!(!interval_d.intersects(&interval_a));
    }

    #[test]
    fn test_interval_sample_range() {
        let interval = Interval::try_new(1.0, 5.0).unwrap();
        let mut rng = rand::rng();
        for _ in 0..100 {
            let sample = interval.sample_single(&mut rng).unwrap();
            assert!(interval.contains(&sample));
        }
    }
}
