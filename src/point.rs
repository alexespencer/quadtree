use nonempty::NonEmpty;

/// Point represents a point in n-dimensional space.
pub struct Point<T>(NonEmpty<T>);

impl<T> Point<T> {
    pub fn new(vec: NonEmpty<T>) -> Point<T> {
        Point(vec)
    }

    pub fn distance(&self, other: &Point<T>) -> f64
    where
        T: Copy + Into<f64>,
    {
        if self.0.len() != other.0.len() {
            panic!("Points must have the same dimension");
        }
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| {
                let diff = (*a).into() - (*b).into();
                diff * diff
            })
            .sum::<f64>()
            .sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use nonempty::nonempty;

    #[test]
    fn test_point_creation() {
        // 3D using integers
        let point = Point::new(nonempty![1, 2, 3]);
        assert_eq!(point.0, nonempty![1, 2, 3]);

        // 3D using floats
        let point_float = Point::new(nonempty![1.0, 2.0, 3.0]);
        assert_eq!(point_float.0, nonempty![1.0, 2.0, 3.0]);

        // 2D using integers
        let point_2d = Point::new(nonempty![4, 5]);
        assert_eq!(point_2d.0, nonempty![4, 5]);
    }

    #[test]
    fn test_point_distance() {
        let point_a = Point::new(nonempty![1.0, 2.0, 3.0]);
        let point_b = Point::new(nonempty![4.0, 5.0, 6.0]);
        let distance = point_a.distance(&point_b);
        assert_abs_diff_eq!(distance, 5.2, epsilon = 0.01);
    }

    #[test]
    fn test_point_distance_i32() {
        let point_a = Point::new(nonempty![1, 2, 3]);
        let point_b = Point::new(nonempty![4, 5, 6]);
        let distance = point_a.distance(&point_b);
        assert_abs_diff_eq!(distance, 5.2, epsilon = 0.01);
    }
}
