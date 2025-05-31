use std::fmt::Display;

use eyre::{Result, ensure};
use itertools::Itertools;
use ordered_float::OrderedFloat;

use crate::{quadtree::Storable, query::DistanceQuery};

/// Point represents a point in n-dimensional space.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Point<const N: usize>([OrderedFloat<f64>; N]);

impl<const N: usize> Display for Point<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Point({})",
            self.0.iter().map(|x| x.to_string()).join(", ")
        )
    }
}

impl<const N: usize> Default for Point<N> {
    fn default() -> Self {
        Self([OrderedFloat(0.0); N])
    }
}

impl<const N: usize> Point<N> {
    /// Create a new [Point] from a slice of values
    /// Panics if the length of the slice is not equal to N
    pub fn new<T: Copy + Into<f64>>(values: &[T; N]) -> Point<N> {
        Point(
            values
                .iter()
                .map(|value| OrderedFloat((*value).into()))
                .collect_array()
                .expect("same size array"),
        )
    }

    pub fn try_new<T: Copy + Into<f64>>(vec: &Vec<T>) -> Result<Point<N>> {
        ensure!(
            vec.len() == N,
            "cannot create point of size {} from Vec of size {}",
            N,
            &vec.len()
        );

        Ok(Point(
            vec.iter()
                .cloned()
                .map(|value| OrderedFloat((value).into()))
                .collect_array()
                .expect("same sized array"),
        ))
    }

    pub fn dimension_values(&self) -> [f64; N] {
        self.0
            .iter()
            .map(|x| x.0)
            .collect_array()
            .expect("same size array")
    }

    pub fn dimensions(&self) -> usize {
        self.0.len()
    }

    pub fn distance(&self, other: &Point<N>) -> f64 {
        if self.0.len() != other.0.len() {
            panic!("Points must have the same dimension");
        }
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| {
                let diff = a - b;
                (diff * diff).0
            })
            .sum::<f64>()
            .sqrt()
    }

    pub fn to_distance_based_query(&self, distance: f64) -> DistanceQuery<N> {
        DistanceQuery::new(self, distance)
    }
}

/// We can trivialy implement [Storable] for [Point]
impl<const N: usize> Storable<Point<N>, N> for Point<N> {
    fn point(&self) -> &Point<N> {
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
        let point = Point::try_new(&vec![1, 2, 3]).unwrap();
        assert_eq!(point.0, [1.0, 2.0, 3.0]);

        // 3D using floats
        let point_float = Point::try_new(&vec![1.0, 2.0, 3.0]).unwrap();
        assert_eq!(point_float.0, [1.0, 2.0, 3.0]);

        // 2D using integers
        let point_2d = Point::try_new(&vec![4, 5]).unwrap();
        assert_eq!(point_2d.0, [4.0, 5.0]);

        // Try to force an 3D slice into a 2D Point
        let result = Point::<2>::try_new(&vec![4, 5, 6]);
        assert!(result.is_err());
    }

    #[test]
    fn test_point_creation_slices() {
        // 3D using integers
        let point = Point::new(&[1, 2, 3]);
        assert_eq!(point.0, [1.0, 2.0, 3.0]);

        // 3D using floats
        let point_float = Point::new(&[1.0, 2.0, 3.0]);
        assert_eq!(point_float.0, [1.0, 2.0, 3.0]);

        // 2D using integers
        let point_2d = Point::new(&[4, 5]);
        assert_eq!(point_2d.0, [4.0, 5.0]);
    }

    #[test]
    fn test_point_distance() {
        let point_a = Point::new(&[1.0, 2.0, 3.0]);
        let point_b = Point::new(&[4.0, 5.0, 6.0]);
        let distance = point_a.distance(&point_b);
        assert_abs_diff_eq!(distance, 5.2, epsilon = 0.01);
    }

    #[test]
    fn test_point_distance_i32() {
        let point_a = Point::new(&[1, 2, 3]);
        let point_b = Point::new(&[4, 5, 6]);
        let distance = point_a.distance(&point_b);
        assert_abs_diff_eq!(distance, 5.2, epsilon = 0.01);
    }
}
