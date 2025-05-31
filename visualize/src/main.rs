extern crate nannou;

use nannou::{
    event::ElementState,
    prelude::*,
    rand::{SeedableRng, rngs::StdRng},
    winit::event::WindowEvent,
};
use nannou_egui::{Egui, egui};
use quadtree::point::Point;
use visualize::{Model, Settings, Technique};

fn main() {
    nannou::app(create_model).update(update).run();
}

fn create_model(app: &App) -> Model {
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

    // Get the egui context
    let ctx = model.egui.ctx();

    // Handle mouse input for left-click
    if let WindowEvent::MouseInput { state, button, .. } = event {
        let point = Point::new(&[app.mouse.x, app.mouse.y]);
        if *button == MouseButton::Left && *state == ElementState::Pressed {
            // Only add points if egui is not handling the pointer input
            if !ctx.wants_pointer_input() {
                model.add_point(point);
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

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(WHITE);

    // Create rng (will be used for random placement of points)
    let mut _main_rng = StdRng::seed_from_u64(42);

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
