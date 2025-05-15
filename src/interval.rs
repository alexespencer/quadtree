use eyre::{Result, bail};

/// Represents an interval with a start and end value.
/// The interval is inclusive of start and exclusive of end.
pub struct Interval<T: PartialOrd> {
    start: T,
    end: T,
}

impl<T: PartialOrd> Interval<T> {
    pub fn try_new(start: T, end: T) -> Result<Self> {
        if start > end {
            bail!("Start must be less than or equal to end");
        }
        Ok(Interval { start, end })
    }

    pub fn start(&self) -> &T {
        &self.start
    }

    pub fn end(&self) -> &T {
        &self.end
    }

    pub fn contains(&self, value: &T) -> bool {
        self.start <= *value && *value < self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_creation() {
        // Valid interval
        let interval = Interval::try_new(1, 5).unwrap();
        assert_eq!(*interval.start(), 1);
        assert_eq!(*interval.end(), 5);

        // Invalid interval
        let invalid_interval = Interval::try_new(5, 1);
        assert!(invalid_interval.is_err());
    }
}
