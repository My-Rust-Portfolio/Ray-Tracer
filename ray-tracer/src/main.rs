mod app;
mod camera;
mod controller;
mod cpu_renderer;
mod cpu_view;
mod math;
mod objects;
mod scene;

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
        Box::new(|_creation_context| Ok(Box::<app::App>::default())),
    )
}
