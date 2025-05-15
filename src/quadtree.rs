use crate::{point::Point, query::Query, region::Region};
use eyre::{OptionExt, Result, bail};
use std::num::NonZero;

pub trait Storable<T, V> {
    fn point(&self) -> Point<T>;
    fn item(&self) -> &V;
}

pub struct QuadTree<T, V> {
    region: Region,
    subtrees: Option<Vec<QuadTree<T, V>>>,
    points: Vec<Box<dyn Storable<T, V>>>,
}

impl<T: Copy + Into<f64>, V> QuadTree<T, V> {
    pub fn new(region: Region, max_points: NonZero<usize>) -> Self {
        QuadTree {
            region,
            subtrees: None,
            points: Vec::with_capacity(max_points.into()),
        }
    }

    pub fn insert(&mut self, point: impl Storable<T, V> + 'static) -> Result<()> {
        let point = Box::new(point);

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
                return subtree.insert(*point);
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
                        Region::new(region),
                        NonZero::new(self.points.capacity()).expect("non-zero capacity"),
                    )
                })
                .collect(),
        );
    }

    pub fn query<'a, Q>(&'a self, query: &'a Q) -> Box<dyn Iterator<Item = &'a V> + 'a>
    where
        Q: Query<T> + 'a,
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
    use rand::{Rng, SeedableRng};

    use super::*;
    use crate::{interval::Interval, point::Point};

    pub struct TestStruct(Point<i32>, String);
    impl Storable<i32, TestStruct> for TestStruct {
        fn point(&self) -> Point<i32> {
            self.0.clone()
        }

        fn item(&self) -> &Self {
            self
        }
    }

    #[test]
    fn test_quadtree_insert_outside_region() {
        let region = Region::new(vec![
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        let mut quadtree = QuadTree::new(region, NonZero::new(4).unwrap());

        let point_outside = TestStruct(Point::new(vec![11, 5]), "data".to_string());
        assert!(quadtree.insert(point_outside).is_err());
    }

    #[test]
    fn test_quadtree_initialise() {
        let region = Region::new(vec![
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        let _: QuadTree<u32, String> = QuadTree::new(region, NonZero::new(4).unwrap());
    }

    #[test]
    fn test_quadtree_insert_below_capacity() {
        let region = Region::new(vec![
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        let mut quadtree = QuadTree::new(region, NonZero::new(4).unwrap());

        for i in 0..4 {
            quadtree
                .insert(TestStruct(Point::new(vec![i, 0]), "data".to_string()))
                .unwrap();
        }

        assert_eq!(quadtree.points.len(), 4);
        assert!(quadtree.subtrees.is_none());
    }

    #[test]
    fn test_quadtree_insert_above_capacity() {
        let region = Region::new(vec![
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        let mut quadtree = QuadTree::new(region, NonZero::new(4).unwrap());

        for i in 0..4 {
            quadtree
                .insert(TestStruct(Point::new(vec![i, 0]), "data".to_string()))
                .unwrap();
        }

        assert_eq!(quadtree.points.len(), 4);
        assert!(quadtree.subtrees.is_none());

        // Insert one more point to trigger subdivision
        quadtree
            .insert(TestStruct(
                Point::new(vec![5, 5]),
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
        let region = Region::new(vec![
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        let mut quadtree = QuadTree::new(region, NonZero::new(4).unwrap());

        for i in 0..4 {
            quadtree
                .insert(TestStruct(Point::new(vec![i, 0]), "data".to_string()))
                .unwrap();
        }

        // Construct query region that should only contain only the first two points
        let query_region = Region::new(vec![
            Interval::try_new(0.0, 2.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);

        let results: Vec<_> = quadtree.query(&query_region).collect();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_quadtree_query_subdivided() {
        let region = Region::new(vec![
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        // Capacity of 2 will ensure lots of subdivision when inserting 10 items
        let mut quadtree = QuadTree::new(region, NonZero::new(2).unwrap());

        for i in 0..10 {
            quadtree
                .insert(TestStruct(Point::new(vec![i, 0]), "data".to_string()))
                .unwrap();
        }

        // Construct query region
        let query_region = Region::new(vec![
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);

        let results: Vec<_> = quadtree.query(&query_region).collect();

        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_quadtree_many_points() {
        let region = Region::new(vec![
            Interval::try_new(0.0, 100.0).unwrap(),
            Interval::try_new(0.0, 100.0).unwrap(),
        ]);

        // Capacity of 100 will ensure lots of subdivision when inserting 100,000 items
        let mut quadtree = QuadTree::new(region, NonZero::new(100).unwrap());
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);

        for _ in 0..100_000 {
            // Choose random x and y coordinates
            let x = rng.random_range(0..100);
            let y = rng.random_range(0..100);
            quadtree
                .insert(TestStruct(Point::new(vec![x, y]), "data".to_string()))
                .unwrap();
        }

        // Construct query region
        let query_region = Region::new(vec![
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);

        let results: Vec<_> = quadtree.query(&query_region).collect();
        dbg!(&results.len());
        assert!((900..=1100).contains(&results.len()));
    }
}
