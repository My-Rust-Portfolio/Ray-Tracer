use std::sync::Arc;

use eframe::egui;
use eframe::egui_wgpu::{self, wgpu};

/// A direct-to-render-pass GPU view used as the foundation for the GPU tracer.
/// It deliberately has no CPU pixel buffer or readback path.
pub struct GpuViewport {
    pipeline: Arc<wgpu::RenderPipeline>,
}

impl GpuViewport {
    pub fn new(render_state: &egui_wgpu::RenderState) -> Self {
        let device = &render_state.device;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gpu-viewport-preview"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gpu_viewport.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gpu-viewport-preview"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(render_state.target_format.into())],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline: Arc::new(pipeline),
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            GpuPreviewCallback {
                pipeline: self.pipeline.clone(),
            },
        ));
    }
}

struct GpuPreviewCallback {
    pipeline: Arc<wgpu::RenderPipeline>,
}

impl egui_wgpu::CallbackTrait for GpuPreviewCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _resources: &egui_wgpu::CallbackResources,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(0..3, 0..1);
    }
}
