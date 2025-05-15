use std::num::NonZero;

use crate::{point::WithPoint, region::Region};

pub struct QuadTree<D: PartialOrd> {
    region: Region<D>,
    subtrees: Option<Vec<QuadTree<D>>>,
    points: Vec<Box<dyn WithPoint<D>>>,
}

impl<D: PartialOrd> QuadTree<D> {
    pub fn new(region: Region<D>, max_points: NonZero<usize>) -> Self {
        QuadTree {
            region,
            subtrees: None,
            points: Vec::with_capacity(max_points.into()),
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
    fn test_quadtree() {
        let region = Region::new(vec![
            Interval::try_new(0, 10).unwrap(),
            Interval::try_new(0, 10).unwrap(),
        ]);
        let mut quadtree = QuadTree::new(region, NonZero::new(4).unwrap());

        // for i in 0..10 {
        //     for j in 0..10 {
        //         quadtree.insert(Box::new(RandomData(
        //             Point::new(vec![i, j]),
        //             "data".to_string(),
        //         )));
        //     }
        // }

        // assert_eq!(quadtree.points.len(), 100);
    }
}
