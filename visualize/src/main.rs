extern crate nannou;

use nannou::{event::ElementState, prelude::*, winit::event::WindowEvent};
use nannou_egui::Egui;
use quadtree::point::Point;
use visualize::Model;

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

    Model::try_new(egui, app.window_rect()).expect("valid Rect from app")
}

fn raw_window_event(app: &App, model: &mut Model, event: &WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
    model.set_mouse_position(None);

    // Get the egui context
    let ctx = model.egui.ctx();

    // Handle mouse input
    if let WindowEvent::MouseInput { state, button, .. } = event {
        let point = Point::new(&[app.mouse.x, app.mouse.y]);
        let wants_pointer_input = ctx.wants_pointer_input();
        model.set_mouse_position(Some(point));

        // Only add points if egui is not handling the pointer input
        if !wants_pointer_input {
            // Left-click (adding point manually)
            if *button == MouseButton::Left && *state == ElementState::Pressed {
                model.add_point(point);
            }
            // Right-click (adding random points)
            if *button == MouseButton::Right && *state == ElementState::Pressed {
                model.add_random_points(500);
            }
        }
    } else if let WindowEvent::CursorMoved { .. } = event {
        let point = Point::new(&[app.mouse.x, app.mouse.y]);
        model.set_mouse_position(Some(point));
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    let egui = &mut model.egui;
    let _ctx = egui.begin_frame();
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(WHITE);

    // Prepare to draw.
    let draw = app.draw();

    // Draw the points, mouse query circle and QuadTree rectangles
    model.draw_app(&draw);

    // Draw FPS counter
    let fps = app.fps();
    draw.text(&format!("FPS: {:.0}", fps))
        .x_y(
            app.window_rect().right() - 50.0,
            app.window_rect().top() - 20.0,
        ) // Position in top-right
        .color(BLACK)
        .font_size(20);

    // Add help instructions
    draw.text("Left-click to add a point, Right-click to add random points")
        .x_y(0.0, app.window_rect().bottom() + 20.0)
        .width(500.0)
        .color(BLACK)
        .font_size(14);

    // Write to the window frame and draw the egui menu.
    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
