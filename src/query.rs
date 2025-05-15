use crate::{point::Point, region::Region};

pub trait Query<T> {
    fn region(&self) -> &Region;
    fn contains(&self, point: &Point<T>) -> bool;
}
