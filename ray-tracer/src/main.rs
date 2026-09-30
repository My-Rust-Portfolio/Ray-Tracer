mod app;
mod camera;
mod controller;
mod material;
mod math;
mod objects;
mod renderer;
mod scene;
mod ui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Ray Tracer")
            .with_inner_size([1100.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Ray Tracer",
        options,
        Box::new(|creation_context| Ok(Box::new(app::App::new(creation_context)))),
    )
}
