use crate::quadtree::Storable;

/// Point represents a point in n-dimensional space.
#[derive(Debug, Clone, PartialEq)]
pub struct Point(Vec<f64>);

impl Point {
    pub fn new<T: Into<f64>>(vec: Vec<T>) -> Point {
        Point(vec.into_iter().map(Into::into).collect::<Vec<_>>())
    }

    pub fn dimension_values(&self) -> &[f64] {
        &self.0
    }

    pub fn dimensions(&self) -> usize {
        self.0.len()
    }

    pub fn distance(&self, other: &Point) -> f64 {
        if self.0.len() != other.0.len() {
            panic!("Points must have the same dimension");
        }
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| {
                let diff = a - b;
                diff * diff
            })
            .sum::<f64>()
            .sqrt()
    }
}

/// We can trivially implement [Storable] for [Point]
impl Storable<Point> for Point {
    fn point(&self) -> &Point {
        self
    }

    fn item(&self) -> &Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_point_creation() {
        // 3D using integers
        let point = Point::new(vec![1, 2, 3]);
        assert_eq!(point.0, vec![1.0, 2.0, 3.0]);

        // 3D using floats
        let point_float = Point::new(vec![1.0, 2.0, 3.0]);
        assert_eq!(point_float.0, vec![1.0, 2.0, 3.0]);

        // 2D using integers
        let point_2d = Point::new(vec![4, 5]);
        assert_eq!(point_2d.0, vec![4.0, 5.0]);
    }

    #[test]
    fn test_point_distance() {
        let point_a = Point::new(vec![1.0, 2.0, 3.0]);
        let point_b = Point::new(vec![4.0, 5.0, 6.0]);
        let distance = point_a.distance(&point_b);
        assert_abs_diff_eq!(distance, 5.2, epsilon = 0.01);
    }

    #[test]
    fn test_point_distance_i32() {
        let point_a = Point::new(vec![1, 2, 3]);
        let point_b = Point::new(vec![4, 5, 6]);
        let distance = point_a.distance(&point_b);
        assert_abs_diff_eq!(distance, 5.2, epsilon = 0.01);
    }
}
