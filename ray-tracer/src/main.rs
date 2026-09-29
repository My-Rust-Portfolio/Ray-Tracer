mod app;
mod camera;
mod cpu_view;
mod math;
mod objects;
mod renderer;
mod scene;

fn main() -> Result<(), winit::error::EventLoopError> {
    let width = 800;
    let height = 600;

    let app = app::App::new(width, height);
    app.run()
}
