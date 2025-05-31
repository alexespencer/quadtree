use eyre::OptionExt;
use eyre::Result;
use quadtree::{interval::Interval, point::Point, quadtree::QuadTree, region::Region};

use std::num::NonZero;

fn main() -> Result<()> {
    // Create a region (the bounds of the quadtree)
    let region = Region::new(&[
        Interval::try_new(0.0, 10.0)?, // X-axis
        Interval::try_new(0.0, 10.0)?, // Y-axis
    ]);

    // Initialise the QuadTree
    let mut quadtree = QuadTree::new(&region, NonZero::new(4).ok_or_eyre("value must be > 0")?);

    // Insert points into the QuadTree
    for i in 0..4 {
        quadtree.insert(Point::new(&[i, 0]))?;
    }

    // Query quadtree
    let query_region = Region::new(&[Interval::try_new(0.0, 2.0)?, Interval::try_new(0.0, 10.0)?]);
    let results: Vec<_> = quadtree.query(&query_region).collect();

    println!(
        "There are {} points within the distance query",
        results.len()
    );
    Ok(())
}
