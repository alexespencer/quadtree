use crate::{point::WithPoint, region::Region};
use eyre::{OptionExt, Result, bail};
use std::num::NonZero;

pub struct QuadTree<T> {
    region: Region,
    subtrees: Option<Vec<QuadTree<T>>>,
    points: Vec<Box<dyn WithPoint<T>>>,
}

impl<T: Copy + Into<f64>> QuadTree<T> {
    pub fn new(region: Region, max_points: NonZero<usize>) -> Self {
        QuadTree {
            region,
            subtrees: None,
            points: Vec::with_capacity(max_points.into()),
        }
    }

    pub fn insert(&mut self, point: Box<dyn WithPoint<T>>) -> Result<()> {
        if self.points.len() < self.points.capacity() {
            self.points.push(point);
            Ok(())
        } else {
            if self.subtrees.is_none() {
                // self.subdivide();
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{interval::Interval, point::Point};

    pub struct RandomData(Point<i32>, String);
    impl WithPoint<i32> for RandomData {
        fn point(&self) -> Point<i32> {
            self.0.clone()
        }
    }

    #[test]
    fn test_quadtree_initialise() {
        let region = Region::new(vec![
            Interval::try_new(0.0, 10.0).unwrap(),
            Interval::try_new(0.0, 10.0).unwrap(),
        ]);
        let _: QuadTree<u32> = QuadTree::new(region, NonZero::new(4).unwrap());
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
}
