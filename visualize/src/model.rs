extern crate nannou;

use std::collections::HashSet;
use std::num::NonZeroUsize;

use eyre::{Context, Result};

use nannou::Draw;
use nannou::geom::Rect;
use nannou_egui::Egui;
use quadtree::QuadTree;
use quadtree::region::Region;
use quadtree::{interval::Interval, point::Point};
use rand::SeedableRng;
use rand::rngs::StdRng;

use nannou::prelude::*;

const RADIUS: f32 = 100.0;
const DOT_SIZE: f32 = 7.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Technique {
    /// Use comparision of coordinates to determine distance
    Cartesian,
    /// Use a quadtree to determine distance
    Quadtree,
}

pub struct Model {
    pub egui: Egui,
    pub points: Vec<Point<2>>,
    pub mouse_position: Option<Point<2>>,
    pub region: Region<2>,
    rng: StdRng,
}

impl Model {
    pub fn try_new(egui: Egui, rect: Rect) -> Result<Self> {
        // Create a region from the Rect
        let region = Region::new(&[
            Interval::try_new(rect.left() as f64, rect.right() as f64)
                .context("converting rect to Interval")?,
            Interval::try_new(rect.bottom() as f64, rect.top() as f64)
                .context("converting rect to Interval")?,
        ]);
        Ok(Self {
            egui,
            points: Vec::new(),
            mouse_position: None,
            region,
            rng: SeedableRng::seed_from_u64(42),
        })
    }

    pub fn add_point(&mut self, point: Point<2>) {
        self.points.push(point);
    }

    pub fn add_random_points(&mut self, count: usize) {
        for _ in 0..count {
            let point = self.region.sample_point(&mut self.rng);
            self.add_point(point);
        }
    }

    fn points_within_distance(
        &self,
        points: &[Point<2>],
        center: &Point<2>,
        distance: f64,
    ) -> HashSet<Point<2>> {
        let mut qt: QuadTree<2, Point<2>> =
            QuadTree::new(&self.region, NonZeroUsize::new(2).expect("2 is non-zero"));
        // Insert points into the quadtree
        for point in points {
            qt.insert(*point)
                .expect("Inserting point into quadtree succeeds");
        }
        // Query
        let distance_squared = center.to_distance_based_query(distance);
        qt.query(&distance_squared).cloned().collect()
    }

    pub fn draw_app(&self, draw: &Draw, points: &[Point<2>]) {
        // Draw circle around the mouse position if it exists, and find points within that circle.
        let points_inside_query: HashSet<Point<2>> = match &self.mouse_position {
            Some(mouse_pos) => {
                let coords = mouse_pos.dimension_values();
                draw.ellipse()
                    .x_y(coords[0] as f32, coords[1] as f32)
                    .w_h(RADIUS * 2.0, RADIUS * 2.0)
                    .stroke(GREEN)
                    .stroke_weight(2.0)
                    .no_fill();

                self.points_within_distance(points, &Point::new(coords), RADIUS as f64)
            }
            None => HashSet::new(),
        };

        // Draw the points, colouring them based on whether they are inside the query circle.
        for point in points {
            let coords = point.dimension_values();
            let color = if points_inside_query.contains(point) {
                GREEN
            } else {
                RED
            };
            draw.ellipse()
                .x_y(coords[0] as f32, coords[1] as f32)
                .w_h(DOT_SIZE, DOT_SIZE) // Set the size of the point
                .color(color); // Set the color of the point
        }
    }
}
