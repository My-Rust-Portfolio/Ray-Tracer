use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

use crate::cpu_view::CpuView;

pub struct App {
    window: Option<Arc<Window>>,
    cpu_view: Option<CpuView>,
    width: u32,
    height: u32,
}

impl App {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            window: None,
            cpu_view: None,
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
            .with_title("Ray Tracer")
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.width as f64,
                self.height as f64,
            ));

        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.width = window.inner_size().width;
        self.height = window.inner_size().height;

        let cpu_view = CpuView::new(window.clone(), self.width, self.height);
        self.cpu_view = Some(cpu_view);
        self.window = Some(window);

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                if let Some(cpu_view) = &mut self.cpu_view {
                    cpu_view.resize(width, height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(cpu_view) = &mut self.cpu_view {
                    cpu_view.render_frame();
                }

                if let Some(window) = &self.window {
                    window.pre_present_notify();
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
