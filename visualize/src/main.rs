extern crate nannou;

use nannou::{
    prelude::*,
    rand::{SeedableRng, rngs::StdRng},
};
use nannou_egui::{Egui, egui};

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

    Model {
        seed: 42,
        settings: Settings {
            technique: Technique::Cartesian,
        },
        egui,
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
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

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(WHITE);
    let _window = app.window_rect();

    // Create rng
    let mut _main_rng = StdRng::seed_from_u64(model.seed);

    // Prepare to draw.
    let draw = app.draw();

    // Write to the window frame and draw the egui menu.
    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
