use crate::{interval::Interval, point::Point, query::Query};
use eyre::{Result, ensure};
use itertools::Itertools;
use rand::{Rng, distr::uniform::SampleRange};

/// A region in n-dimensional space defined by a Vec of intervals.
#[derive(Debug, Clone, PartialEq)]
pub struct Region<const N: usize>([Interval; N]);

impl<const N: usize> Region<N> {
    pub fn new(intervals: &[Interval; N]) -> Self {
        Region(*intervals)
    }

    pub fn try_new(intervals: &[Interval]) -> Result<Self> {
        ensure!(
            intervals.len() == N,
            "cannot create region of size {} from Vec of size {}",
            N,
            intervals.len()
        );
        Ok(Region(
            intervals
                .iter()
                .cloned()
                .collect_array()
                .expect("same sized array"),
        ))
    }

    pub fn intervals(&self) -> &[Interval; N] {
        &self.0
    }

    pub fn contains(&self, point: &Point<N>) -> bool {
        self.intervals()
            .iter()
            .zip(point.dimension_values())
            .all(|(interval, value)| interval.contains(value))
    }

    pub fn subdivide(&self) -> Vec<[Interval; N]> {
        let iterators = self
            .intervals()
            .iter()
            .map(|interval| interval.subdivide())
            .collect::<Vec<_>>();

        iterators
            .iter()
            .map(|v| v.iter().cloned())
            .multi_cartesian_product()
            .map(|product| {
                product
                    .into_iter()
                    .collect_array()
                    .expect("same sized array")
            })
            .collect::<Vec<_>>()
    }

    pub fn intersects(&self, other: &Region<N>) -> bool {
        self.intervals()
            .iter()
            .zip(other.intervals().iter())
            .all(|(a, b)| a.intersects(b))
    }

    pub fn sample_point(&self, rng: &mut impl Rng) -> Point<N> {
        let values: Vec<f64> = self
            .intervals()
            .iter()
            .map(|interval| interval.sample_single(rng).unwrap())
            .collect();
        Point::try_new(&values).expect("should be same size as N")
    }
}

/// We can trivially implement [Query] for [Region]
/// This allows us to use Region in a QuadTree query
impl<const N: usize> Query<N> for Region<N> {
    fn region(&self) -> &Region<N> {
        self
    }

    fn contains(&self, point: &Point<N>) -> bool {
        self.contains(point)
    }
}

#[cfg(feature = "nannou")]
impl From<&Region<2>> for nannou::geom::Rect {
    fn from(region: &Region<2>) -> Self {
        // Convert the region to a Rect
        nannou::geom::Rect::from_corners(
            nannou::geom::pt2(
                *region.intervals()[0].start() as f32,
                *region.intervals()[1].start() as f32,
            ),
            nannou::geom::pt2(
                *region.intervals()[0].end() as f32,
                *region.intervals()[1].end() as f32,
            ),
        )
    }
}

#[cfg(feature = "nannou")]
impl From<nannou::geom::Rect> for Region<2> {
    fn from(rect: nannou::geom::Rect) -> Self {
        Region::new(&[
            Interval::try_new(rect.left() as f64, rect.right() as f64)
                .expect("valid Rect produces valid x-axis"),
            Interval::try_new(rect.bottom() as f64, rect.top() as f64)
                .expect("valid Rect produces valid y-axis"),
        ])
    }
}

/// Demonstrates region containment with correct and incorrect point dimensions.
///
/// This compiles:
/// ```
/// use quadtree::{interval::Interval, point::Point, region::Region};
/// let axis = Interval::try_new(1.0, 5.0).unwrap();
/// let region = Region::new(&[axis]);
/// let point = Point::new(&[3.0]);
/// assert!(region.contains(&point));
/// ```
///
/// This fails to compile due to a dimension mismatch:
/// ```compile_fail
/// use quadtree::{interval::Interval, point::Point, region::Region};
/// let axis = Interval::try_new(1.0, 5.0).unwrap();
/// let region = Region::new(&[axis]);
/// let point = Point::new(&[3.0, 4.0]); // 2D point for 1D region
/// region.contains(&point);
/// ```
#[allow(dead_code)]
fn test_compile_fail_different_dimensions() {}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use itertools::Itertools;

    use crate::{interval::Interval, point::Point, region::Region};

    #[test]
    fn test_region_1d() {
        let single_axis = Interval::try_new(1.0, 5.0).unwrap();
        let region = Region::new(&[single_axis]);
        assert_eq!(region.intervals().len(), 1);

        let point = Point::new(&[3]);
        assert!(region.contains(&point));
        let point_outside = Point::new(&[6]);
        assert!(!region.contains(&point_outside));
    }

    #[test]
    fn test_region_2d_subdivide() {
        let x_axis = Interval::try_new(1.0, 5.0).unwrap();
        let y_axis = Interval::try_new(20.0, 60.0).unwrap();
        let region = Region::new(&[x_axis, y_axis]);
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
        let region = Region::new(&[x_axis, y_axis, z_axis]);

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

    #[test]
    fn test_intersects() {
        let x_axis = Interval::try_new(1.0, 5.0).unwrap();
        let y_axis = Interval::try_new(20.0, 60.0).unwrap();
        let region_a = Region::new(&[x_axis, y_axis]);
        let region_b = Region::new(&[Interval::try_new(4.0, 6.0).unwrap(), y_axis]);

        assert!(region_a.intersects(&region_b));

        let region_c = Region::new(&[Interval::try_new(6.0, 8.0).unwrap(), y_axis]);
        assert!(!region_a.intersects(&region_c));
    }

    #[test]
    fn test_sample_point() {
        let x_axis = Interval::try_new(1.0, 5.0).unwrap();
        let y_axis = Interval::try_new(20.0, 60.0).unwrap();
        let region = Region::new(&[x_axis, y_axis]);

        use rand::{SeedableRng, rngs::StdRng};
        let mut rng = StdRng::seed_from_u64(42);

        let mut points_seen = HashSet::new();
        for _ in 0..100 {
            let point = region.sample_point(&mut rng);
            points_seen.insert(point);
            assert!(region.contains(&point));
        }
        // Ensure we sampled multiple unique points
        assert_eq!(points_seen.len(), 100);
    }
}
