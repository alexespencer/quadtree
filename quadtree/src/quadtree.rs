use crate::{point::Point, query::Query, region::Region};
use eyre::{OptionExt, Result, bail};
use std::num::NonZero;

pub trait Storable<V, const N: usize> {
    fn point(&self) -> &Point<N>;
    fn item(&self) -> &V;
}

/// Technically an 'orthree' this QuadTree struct is actually a generalised version
/// of a quadtree that can be used for any number of dimensions.
/// See <https://en.wikipedia.org/wiki/Quadtree> for more information.
/// Motivation: We want to be able to store points and query regions efficiently.
/// ```rust
/// # use eyre::OptionExt;
/// # fn main() -> eyre::Result<()> {
/// # use quadtree::{interval::Interval, point::Point, quadtree::QuadTree, region::Region};
/// # use std::num::NonZero;
/// // Create a region, the bounds of the quadtree
/// let region = Region::new(&[
///     Interval::try_new(0.0, 10.0)?, // X-axis
///     Interval::try_new(0.0, 10.0)?, // Y-axis
/// ]);
///
/// // Initialise the QuadTree with this region and the maximum number of points each individual node
/// // should store. You can store any Struct in the QuadTree as long as it implements the Storable trait.
/// // Here we're deferring the type of the QuadTree to the compiler,
/// // inferred from the first insert
/// let mut quadtree = QuadTree::new(&region, NonZero::new(4).ok_or_eyre("value must be > 0")?);
///
/// // Insert points into the QuadTree
/// for i in 0..4 {
///     quadtree.insert(Point::new(&[i, 0]))?;
/// }
///
/// // To query the QuadTree, provide a region, or anything that implements the Query trait
/// let query_region = Region::new(&[
///     Interval::try_new(0.0, 2.0)?,
///     Interval::try_new(0.0, 10.0)?,
/// ]);
///
/// let results: Vec<_> = quadtree.query(&query_region).collect();
/// assert_eq!(results.len(), 2);
///
/// // Alternatively, search around a point using a DistanceQuery
/// let distance_query = Point::new(&[5.0, 5.0]).to_distance_based_query(3.0);
/// let results: Vec<_> = quadtree.query(&distance_query).collect();
/// #
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct QuadTree<const N: usize, V> {
    region: Region<N>,
    subtrees: Option<Vec<QuadTree<N, V>>>,
    points: Vec<V>,
}

impl<const N: usize, V: Storable<V, N>> QuadTree<N, V> {
    /// Create a new [QuadTree] with the given region and maximum number of points.
    pub fn new(region: &Region<N>, max_points: NonZero<usize>) -> Self {
        QuadTree {
            region: region.clone(),
            subtrees: None,
            points: Vec::with_capacity(max_points.into()),
        }
    }

    /// Try to insert a point into the [QuadTree]. If the point is outside the quadtree's region, an error is returned.
    /// All points must be [Storable] and of the type set in the [QuadTree].
    pub fn insert(&mut self, point: V) -> Result<()> {
        if self.points.len() < self.points.capacity() {
            if !self.region.contains(&point.point()) {
                bail!("Point is outside the region");
            }
            self.points.push(point);
            return Ok(());
        }

        if self.subtrees.is_none() {
            self.subdivide();
        }

        for subtree in self
            .subtrees
            .as_mut()
            .ok_or_eyre("subtrees not created, this is a bug")?
        {
            if subtree.region.contains(&point.point()) {
                return subtree.insert(point);
            }
        }

        // If we get here, the point was not inserted, which should not happen
        bail!("Point not inserted into any subtree");
    }

    fn subdivide(&mut self) {
        let subregions = self.region.subdivide();
        self.subtrees = Some(
            subregions
                .into_iter()
                .map(|region| {
                    QuadTree::new(
                        &Region::new(&region),
                        NonZero::new(self.points.capacity()).expect("non-zero capacity"),
                    )
                })
                .collect(),
        );
    }

    /// Query the [QuadTree] with a region (any type that implements the [Query] trait).
    pub fn query<'a, Q>(&'a self, query: &'a Q) -> Box<dyn Iterator<Item = &'a V> + 'a>
    where
        Q: Query<N> + 'a,
    {
        let my_iter = self
            .points
            .iter()
            .filter_map(move |point| query.contains(&point.point()).then_some(point.item()));

        let subtree_iter = self.subtrees.iter().flat_map(|subtrees| {
            subtrees
                .iter()
                .filter(|subtree| subtree.region.intersects(query.region()))
                .flat_map(|subtree| subtree.query(query))
        });

        Box::new(my_iter.chain(subtree_iter))
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rand::{Rng, SeedableRng};

    use super::*;
    use crate::{interval::Interval, point::Point, query::DistanceQuery};

    pub struct TestStruct(Point<2>, String);
    impl Storable<TestStruct, 2> for TestStruct {
        fn point(&self) -> &Point<2> {
            &self.0
        }

        fn item(&self) -> &Self {
            self
        }
    }

    #[test]
    fn test_quadtree_insert_outside_region() {
        let region = Region::new(&[
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        let mut quadtree = QuadTree::new(&region, NonZero::new(4).unwrap());

        let point_outside = TestStruct(Point::new(&[11, 5]), "data".to_string());
        assert!(quadtree.insert(point_outside).is_err());
    }

    #[test]
    fn test_quadtree_initialise() {
        let region = Region::new(&[
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        let _: QuadTree<2, TestStruct> = QuadTree::new(&region, NonZero::new(4).unwrap());
    }

    #[test]
    fn test_quadtree_insert_below_capacity() {
        let region = Region::new(&[
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        let mut quadtree = QuadTree::new(&region, NonZero::new(4).unwrap());

        for i in 0..4 {
            quadtree
                .insert(TestStruct(Point::new(&[i, 0]), "data".to_string()))
                .unwrap();
        }

        assert_eq!(quadtree.points.len(), 4);
        assert!(quadtree.subtrees.is_none());
    }

    #[test]
    fn test_quadtree_insert_above_capacity() {
        let region = Region::new(&[
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        let mut quadtree = QuadTree::new(&region, NonZero::new(4).unwrap());

        for i in 0..4 {
            quadtree
                .insert(TestStruct(Point::new(&[i, 0]), "data".to_string()))
                .unwrap();
        }

        assert_eq!(quadtree.points.len(), 4);
        assert!(quadtree.subtrees.is_none());

        // Insert one more point to trigger subdivision
        quadtree
            .insert(TestStruct(
                Point::new(&[5, 5]),
                "data_subdivided".to_string(),
            ))
            .unwrap();

        // Check that the quadtree has subdivided
        assert!(quadtree.subtrees.is_some());
        let subtrees = quadtree.subtrees.as_ref().unwrap();
        assert_eq!(subtrees.len(), 4);

        // Assert the point went into only 1 subtree
        let subtree_total_points: usize = subtrees.iter().map(|st| st.points.len()).sum();
        assert_eq!(subtree_total_points, 1);
        assert!(
            subtrees
                .iter()
                .flat_map(|subtree| subtree.points.iter())
                .map(|p| p.item().1 == "data_subdivided")
                .all(|x| x)
        );
    }

    #[test]
    fn test_quadtree_query() {
        let region = Region::new(&[
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        let mut quadtree = QuadTree::new(&region, NonZero::new(4).unwrap());

        for i in 0..4 {
            quadtree
                .insert(TestStruct(Point::new(&[i, 0]), "data".to_string()))
                .unwrap();
        }

        // Construct query region that should only contain only the first two points
        let query_region = Region::new(&[
            Interval::try_new(0.0, 2.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);

        let results: Vec<_> = quadtree.query(&query_region).collect();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_quadtree_query_subdivided() {
        let region = Region::new(&[
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        // Capacity of 2 will ensure lots of subdivision when inserting 10 items
        let mut quadtree = QuadTree::new(&region, NonZero::new(2).unwrap());

        for i in 0..10 {
            quadtree
                .insert(TestStruct(Point::new(&[i, 0]), "data".to_string()))
                .unwrap();
        }

        // Construct query region
        let query_region = Region::new(&[
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);

        let results: Vec<_> = quadtree.query(&query_region).collect();

        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_quadtree_many_points() {
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
            quadtree
                .insert(TestStruct(Point::new(&[x, y]), "data".to_string()))
                .unwrap();
        }

        // Construct query region
        let query_region = Region::new(&[
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);

        let results: Vec<_> = quadtree.query(&query_region).collect();
        dbg!(&results.len());
        assert!((900..=1100).contains(&results.len()));
    }

    #[test]
    fn test_quadtree_single_point_interval() {
        const COUNT: usize = 10;
        // Create a quadtree where the region is a single point
        let region = Region::new(&[
            Interval::try_new(1.0, 1.0 + f64::EPSILON).unwrap(),
            Interval::try_new(1.0, 1.0 + f64::EPSILON).unwrap(),
        ]);
        let mut quadtree = QuadTree::new(&region, NonZero::new(1).unwrap());

        // Insert 10 points into the quadtree
        for i in 0..COUNT {
            quadtree
                .insert(TestStruct(Point::new(&[1.0, 1.0]), format!("P{}", i)))
                .unwrap();
        }

        // Query the quadtree
        let results: Vec<_> = quadtree.query(&region).collect();
        assert_eq!(results.len(), 10);
        // Assert there are 10 unique strings
        let unique_results: Vec<_> = results.iter().map(|item| item.item().1.clone()).collect();
        assert_eq!(
            unique_results
                .into_iter()
                .unique()
                .collect::<Vec<_>>()
                .len(),
            COUNT
        );
    }

    #[test]
    fn perf_smoke_test_neighbours() {
        const POINT_COUNT: usize = 2000;
        // Create a Vec of random points
        let mut rng = rand::rng();
        let points: Vec<Point<2>> = (0..POINT_COUNT)
            .map(|_| Point::new(&[rng.random_range(0.0..1000.0), rng.random_range(0.0..1000.0)]))
            .collect();

        // Create a QuadTree with a region that covers the points
        let region = Region::new(&[
            Interval::try_new(0.0, 1000.0).unwrap(),
            Interval::try_new(0.0, 1000.0).unwrap(),
        ]);
        let mut quadtree = QuadTree::new(&region, NonZero::new(10).unwrap());
        for point in &points {
            quadtree.insert(point.clone()).unwrap();
        }

        // Loop over the points and count how many points have a neighbour within a distance of 10.0
        // not using the quadtree, first - but a double loop
        let start = std::time::Instant::now();
        let count_non_quadtree = points
            .iter()
            .filter(|point| {
                points
                    .iter()
                    .filter(|other| point != other && point.distance(other) < 10.0)
                    .count()
                    > 0
            })
            .count();
        let elapsed_non_quadtree = start.elapsed();
        assert_ne!(
            POINT_COUNT, count_non_quadtree,
            "All points are close-by neighbours?"
        );

        // Now use the quadtree to count how many points have a neighbour within a distance of 10.0
        // Reset the timer
        let start = std::time::Instant::now();
        let count_quadtree = points
            .iter()
            .filter(|&point| {
                let query_region = DistanceQuery::new(point, 10.0);
                quadtree
                    .query(&query_region)
                    .filter(|other_point| *other_point != point)
                    .count()
                    > 0
            })
            .count();
        let elapsed_quadtree = start.elapsed();

        dbg!(elapsed_non_quadtree);
        dbg!(elapsed_quadtree);
        dbg!(count_quadtree);

        // Quad tree should be faster than non-quadtree but find the same number of points
        assert_eq!(count_quadtree, count_non_quadtree);
        assert!(elapsed_quadtree < elapsed_non_quadtree);
    }

    #[test]
    fn perf_smoke_test_region() {
        const POINT_COUNT: usize = 100000;
        // Create a Vec of random points
        let mut rng = rand::rng();
        let points: Vec<Point<2>> = (0..POINT_COUNT)
            .map(|_| Point::new(&[rng.random_range(0.0..1000.0), rng.random_range(0.0..1000.0)]))
            .collect();

        // Create a QuadTree with a region that covers the points
        let region = Region::new(&[
            Interval::try_new(0.0, 1000.0).unwrap(),
            Interval::try_new(0.0, 1000.0).unwrap(),
        ]);
        let mut quadtree = QuadTree::new(&region, NonZero::new(10).unwrap());
        for point in &points {
            quadtree.insert(point.clone()).unwrap();
        }

        let search_region = Region::new(&[
            Interval::try_new(550.0, 600.0).unwrap(),
            Interval::try_new(0.0, 200.0).unwrap(),
        ]);

        // Loop over the points and count how many points are in the random region
        // not using the quadtree, first - but a double loop
        let start = std::time::Instant::now();
        let count_non_quadtree = points
            .iter()
            .filter(|point| search_region.contains(&point.point()))
            .count();
        let elapsed_non_quadtree = start.elapsed();
        assert_ne!(
            POINT_COUNT, count_non_quadtree,
            "All points are in search region?"
        );

        // Now use the quadtree to count how many points have a neighbour within a distance of 10.0
        // Reset the timer
        let start = std::time::Instant::now();
        let count_quadtree = quadtree.query(&search_region).count();
        let elapsed_quadtree = start.elapsed();

        dbg!(elapsed_non_quadtree);
        dbg!(elapsed_quadtree);
        dbg!(count_quadtree);

        // Quad tree should be faster than non-quadtree but find the same number of points
        assert_eq!(count_quadtree, count_non_quadtree);
        assert!(elapsed_quadtree < elapsed_non_quadtree);
    }
}
