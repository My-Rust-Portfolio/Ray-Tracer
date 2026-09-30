mod app;
mod camera;
mod controller;
mod cpu_renderer;
mod cpu_view;
mod math;
mod objects;
mod scene;

fn main() -> Result<(), winit::error::EventLoopError> {
    let width = 800;
    let height = 600;

    let app = app::App::new(width, height);
    app.run()
}
