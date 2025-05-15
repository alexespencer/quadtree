use crate::{point::Point, region::Region};
use eyre::{OptionExt, Result, bail};
use std::num::NonZero;

pub trait Storable<T, V> {
    fn point(&self) -> Point<T>;

    fn data(&self) -> V;
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

    pub fn insert(&mut self, point: Box<dyn Storable<T, V>>) -> Result<()> {
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
                        Region::new(region),
                        NonZero::new(self.points.capacity()).expect("non-zero capacity"),
                    )
                })
                .collect(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{interval::Interval, point::Point};

    pub struct RandomData(Point<i32>, String);
    impl Storable<i32, String> for RandomData {
        fn point(&self) -> Point<i32> {
            self.0.clone()
        }

        fn data(&self) -> String {
            self.1.clone()
        }
    }

    // Tests to do:
    // Check inserting a point outside the region fails

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
                .insert(Box::new(RandomData(
                    Point::new(vec![i, 0]),
                    "data".to_string(),
                )))
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
                .insert(Box::new(RandomData(
                    Point::new(vec![i, 0]),
                    "data".to_string(),
                )))
                .unwrap();
        }

        assert_eq!(quadtree.points.len(), 4);
        assert!(quadtree.subtrees.is_none());

        // Insert one more point to trigger subdivision
        quadtree
            .insert(Box::new(RandomData(
                Point::new(vec![5, 5]),
                "data_subdivided".to_string(),
            )))
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
                .map(|p| p.data() == "data_subdivided")
                .all(|x| x)
        );
    }
}
