extern crate nannou;

use nannou::{
    event::ElementState,
    prelude::*,
    rand::{SeedableRng, rngs::StdRng},
    winit::event::WindowEvent,
};
use nannou_egui::{Egui, egui};
use quadtree::point::Point;

fn main() {
    nannou::app(model).update(update).run();
}

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
}

impl Model {
    fn new(seed: u64, settings: Settings, egui: Egui) -> Self {
        Self {
            seed,
            settings,
            egui,
            points: Vec::new(),
            mouse_position: None,
        }
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

    Model::new(
        42,
        Settings {
            technique: Technique::Cartesian,
        },
        egui,
    )
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

fn draw_app(draw: &Draw, points: &[Point<2>], model: &Model) {
    for point in points {
        let coords = point.dimension_values();
        draw.ellipse()
            .x_y(coords[0] as f32, coords[1] as f32)
            .w_h(5.0, 5.0) // Set the size of the point
            .color(BLACK); // Set the color of the point
    }

    // Draw circle around the mouse position if it exists
    if let Some(mouse_pos) = &model.mouse_position {
        let coords = mouse_pos.dimension_values();
        draw.ellipse()
            .x_y(coords[0] as f32, coords[1] as f32)
            .w_h(200.0, 200.0) // Set the size of the circle
            .stroke(GREEN)
            .stroke_weight(2.0)
            .no_fill();
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(WHITE);
    let _window = app.window_rect();

    // Create rng
    let mut _main_rng = StdRng::seed_from_u64(model.seed);

    // Prepare to draw.
    let draw = app.draw();

    // Draw the points
    draw_app(&draw, &model.points, model);

    // Write to the window frame and draw the egui menu.
    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
