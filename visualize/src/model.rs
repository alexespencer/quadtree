extern crate nannou;

use std::collections::HashSet;
use std::num::NonZeroUsize;

use eyre::Result;

use nannou::Draw;
use nannou::geom::Rect;
use nannou_egui::Egui;
use quadtree::QuadTree;
use quadtree::point::Point;
use quadtree::region::Region;
use rand::SeedableRng;
use rand::rngs::StdRng;

use nannou::prelude::*;

const RADIUS: f32 = 100.0;
const DOT_SIZE: f32 = 7.5;
type Point2D = Point<2>;

pub struct Model {
    pub egui: Egui,
    points: Vec<Point2D>,
    mouse_position: Option<Point2D>,
    region: Region<2>,
    rng: StdRng,
}

impl Model {
    pub fn try_new(egui: Egui, rect: Rect) -> Result<Self> {
        // Create a region from the Rect
        let region: Region<2> = rect.into();
        Ok(Self {
            egui,
            points: Vec::new(),
            mouse_position: None,
            region,
            rng: SeedableRng::seed_from_u64(42),
        })
    }

    pub fn set_mouse_position(&mut self, position: Option<Point2D>) {
        self.mouse_position = position;
    }

    pub fn add_point(&mut self, point: Point2D) {
        self.points.push(point);
    }

    pub fn add_random_points(&mut self, count: usize) {
        for _ in 0..count {
            let point = self.region.sample_point(&mut self.rng);
            self.add_point(point);
        }
    }

    fn quadtree(&self) -> QuadTree<2, Point2D> {
        // Create a quadtree with a capacity of 2 points per region
        let mut qt = QuadTree::new(&self.region, NonZeroUsize::new(2).expect("2 is non-zero"));
        for point in &self.points {
            // Insert points into the quadtree
            qt.insert(*point)
                .expect("Inserting point into quadtree succeeds");
        }
        qt
    }

    pub fn draw_app(&self, draw: &Draw) {
        let qt = self.quadtree();
        // Draw circle around the mouse position if it exists, and find points within that circle.
        let points_inside_query = match &self.mouse_position {
            Some(mouse_pos) => {
                let coords = mouse_pos.dimension_values();
                draw.ellipse()
                    .x_y(coords[0] as f32, coords[1] as f32)
                    .w_h(RADIUS * 2.0, RADIUS * 2.0)
                    .stroke(GREEN)
                    .stroke_weight(2.0)
                    .no_fill();

                let distance_query = &Point::new(coords).to_distance_based_query(RADIUS as f64);
                qt.query(distance_query).cloned().collect()
            }
            None => HashSet::new(),
        };

        // Draw the points, colouring them based on whether they are inside the query circle.
        for point in &self.points {
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

        // Draw the quadtree regions
        for region in qt.regions().iter() {
            let rect: Rect = region.into();
            draw.rect()
                .xy(rect.xy())
                .wh(rect.wh())
                .stroke(BLUE)
                .stroke_weight(1.0)
                .no_fill();
        }
    }
}
