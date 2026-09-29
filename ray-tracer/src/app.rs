use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

pub struct App {
    window: Option<Window>,
    width: u32,
    height: u32,
}

impl App {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            window: None,
            width,
            height,
        }
    }

    pub fn run(self) -> Result<(), winit::error::EventLoopError> {
        use winit::event_loop::EventLoop;

        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut app = self;
        event_loop.run_app(&mut app)?;

        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("Rust Ray Tracer")
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.width as f64,
                self.height as f64,
            ));

        self.window = Some(event_loop.create_window(attrs).unwrap());

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
