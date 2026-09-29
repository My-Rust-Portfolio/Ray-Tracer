use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::CursorGrabMode;
use winit::window::{Window, WindowAttributes, WindowId};

use crate::cpu_view::CpuView;

pub struct App {
    window: Option<Arc<Window>>,
    cpu_view: Option<CpuView>,
    width: u32,
    height: u32,
    last_frame: Option<Instant>,
}

impl App {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            window: None,
            cpu_view: None,
            width,
            height,
            last_frame: None,
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
        window.set_cursor_visible(false);
        if let Err(e) = window.set_cursor_grab(CursorGrabMode::Locked) {
            eprintln!("Failed to grab cursor: {e:?}");
        }
        self.last_frame = Some(Instant::now());

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
                let now = Instant::now();
                let dt = self
                    .last_frame
                    .map(|last| last.elapsed().as_secs_f32())
                    .unwrap_or(1.0 / 60.0);
                self.last_frame = Some(now);

                if let Some(cpu_view) = &mut self.cpu_view {
                    cpu_view.render_frame(dt);
                }

                if let Some(window) = &self.window {
                    window.pre_present_notify();
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(cpu_view) = &mut self.cpu_view {
                    let pressed = event.state.is_pressed();
                    if let PhysicalKey::Code(code) = event.physical_key {
                        cpu_view.handle_key(code, pressed);
                    }
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        use winit::event::DeviceEvent;
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            if let Some(cpu_view) = &mut self.cpu_view {
                let sensitivity = 0.003;
                cpu_view.handle_mouse_delta(dx as f32, dy as f32, sensitivity);
            }
        }
    }
}
