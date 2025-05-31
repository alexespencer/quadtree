extern crate nannou;

use eyre::{Context, Result};
use std::{collections::HashSet, num::NonZeroUsize};

use nannou::{
    event::ElementState,
    prelude::*,
    rand::{SeedableRng, rngs::StdRng},
    winit::event::WindowEvent,
};
use nannou_egui::{Egui, egui};
use quadtree::{QuadTree, region::Region};
use quadtree::{interval::Interval, point::Point};

fn main() {
    nannou::app(model).update(update).run();
}

const RADIUS: f32 = 100.0;
const DOT_SIZE: f32 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Technique {
    /// Use comparision of coordinates to determine distance
    Cartesian,
    /// Use a quadtree to determine distance
    Quadtree,
}

struct Settings {
    /// How large the cubes are.
    technique: Technique,
}

struct Model {
    seed: u64,
    settings: Settings,
    egui: Egui,
    points: Vec<Point<2>>,
    mouse_position: Option<Point<2>>,
    region: Region<2>,
}

impl Model {
    fn try_new(seed: u64, settings: Settings, egui: Egui, rect: Rect) -> Result<Self> {
        // Create a region from the Rect
        let region = Region::new(&[
            Interval::try_new(rect.left() as f64, rect.right() as f64)
                .context("converting rect to Interval")?,
            Interval::try_new(rect.bottom() as f64, rect.top() as f64)
                .context("converting rect to Interval")?,
        ]);
        Ok(Self {
            seed,
            settings,
            egui,
            points: Vec::new(),
            mouse_position: None,
            region,
        })
    }

    fn add_point(&mut self, point: Point<2>) {
        self.points.push(point);
    }
}

fn model(app: &App) -> Model {
    // Create window
    let window_id = app
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();

    let egui = Egui::from_window(&window);

    Model::try_new(
        42,
        Settings {
            technique: Technique::Cartesian,
        },
        egui,
        app.window_rect(),
    )
    .expect("valid Rect from app")
}

fn raw_window_event(app: &App, model: &mut Model, event: &WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
    model.mouse_position = None;

    // Handle mouse input for left-click
    if let WindowEvent::MouseInput { state, button, .. } = event {
        if *button == MouseButton::Left {
            let point = Point::new(&[app.mouse.x, app.mouse.y]);
            if *state == ElementState::Pressed {
                model.add_point(point.clone());
            }
            model.mouse_position = Some(point);
        }
    } else if let WindowEvent::CursorMoved { .. } = event {
        let point = Point::new(&[app.mouse.x, app.mouse.y]);
        model.mouse_position = Some(point);
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let settings = &mut model.settings;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.label("Technique:");
        egui::ComboBox::from_label("")
            .selected_text(format!("{:?}", &mut settings.technique))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut settings.technique, Technique::Cartesian, "Cartesian");
                ui.selectable_value(&mut settings.technique, Technique::Quadtree, "Quadtree");
            });
    });
}

impl Model {
    fn points_within_distance(
        &self,
        points: &[Point<2>],
        center: &Point<2>,
        distance: f64,
    ) -> HashSet<Point<2>> {
        match self.settings.technique {
            Technique::Cartesian => points
                .iter()
                .filter(|&point| point.distance(center) <= distance)
                .cloned()
                .collect(),
            Technique::Quadtree => {
                let mut qt: QuadTree<2, Point<2>> =
                    QuadTree::new(&self.region, NonZeroUsize::new(2).expect("2 is non-zero"));
                // Insert points into the quadtree
                for point in points {
                    qt.insert(point.clone())
                        .expect("Inserting point into quadtree succeeds");
                }
                // Query
                let distance_squared = center.to_distance_based_query(distance);
                qt.query(&distance_squared).into_iter().cloned().collect()
            }
        }
    }

    fn draw_app(&self, draw: &Draw, points: &[Point<2>]) {
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

                self.points_within_distance(points, &Point::new(&coords), RADIUS as f64)
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

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(WHITE);

    // Create rng (will be used for random placement of points)
    let mut _main_rng = StdRng::seed_from_u64(model.seed);

    // Prepare to draw.
    let draw = app.draw();

    // Draw the points
    model.draw_app(&draw, &model.points);

    // Draw FPS counter
    let fps = app.fps();
    draw.text(&format!("FPS: {:.0}", fps))
        .x_y(
            app.window_rect().right() - 50.0,
            app.window_rect().top() - 20.0,
        ) // Position in top-right
        .color(BLACK)
        .font_size(16);

    // Write to the window frame and draw the egui menu.
    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
