use std::num::NonZeroU64;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use eframe::egui;
use eframe::egui_wgpu::{self, wgpu};

use super::world::RenderWorld;

const MAX_SPHERES: usize = 256;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuParams {
    camera_origin: [f32; 4],
    camera_forward: [f32; 4],
    camera_right: [f32; 4],
    camera_up: [f32; 4],
    viewport: [f32; 4],
    scene: [u32; 4],
    light_dir: [f32; 4],
    plane_point: [f32; 4],
    plane_normal: [f32; 4],
    plane_base_ambient: [f32; 4],
    plane_properties: [f32; 4],
    plane_shadow_ambient: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuSphere {
    center_radius: [f32; 4],
    base_ambient: [f32; 4],
    properties: [f32; 4],
    shadow_ambient: [f32; 4],
}

/// GPU ray tracer that writes directly into egui's wgpu render pass.
/// Rendered pixels never leave GPU memory.
pub struct GpuViewport {
    pipeline: Arc<wgpu::RenderPipeline>,
    bind_group: Arc<wgpu::BindGroup>,
    uniform_buffer: Arc<wgpu::Buffer>,
    sphere_buffer: Arc<wgpu::Buffer>,
}

impl GpuViewport {
    pub fn new(render_state: &egui_wgpu::RenderState) -> Self {
        let device = &render_state.device;
        let uniform_size = std::mem::size_of::<GpuParams>() as u64;
        let sphere_size = (std::mem::size_of::<GpuSphere>() * MAX_SPHERES) as u64;
        let uniform_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ray-tracer-gpu-params"),
            size: uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        let sphere_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ray-tracer-gpu-spheres"),
            size: sphere_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        let ground_texture = create_texture(
            device,
            &render_state.queue,
            "ray-tracer-ground-texture",
            include_bytes!("../../assets/ground.jpg"),
            true,
        );
        let sky_texture = create_texture(
            device,
            &render_state.queue,
            "ray-tracer-sky-texture",
            include_bytes!("../../assets/sky.png"),
            true,
        );
        let image_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ray-tracer-image-sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ray-tracer-gpu-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gpu_viewport.wgsl").into()),
        });
        let pipeline = Arc::new(
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ray-tracer-gpu-pipeline"),
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
            }),
        );

        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ray-tracer-gpu-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &sphere_buffer,
                        offset: 0,
                        size: NonZeroU64::new(sphere_size),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&ground_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&sky_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&image_sampler),
                },
            ],
        }));

        Self {
            pipeline,
            bind_group,
            uniform_buffer,
            sphere_buffer,
        }
    }

    pub fn show(
        &self,
        ui: &mut egui::Ui,
        world: &mut RenderWorld,
        settings: super::RenderSettings,
    ) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        let pixels_per_point = ui.ctx().pixels_per_point();
        world.resize(
            (rect.width() * pixels_per_point).round() as u32,
            (rect.height() * pixels_per_point).round() as u32,
        );
        response.request_focus();
        if response.dragged() {
            let delta = ui.input(|input| input.pointer.delta());
            world.mouse_delta(delta.x, delta.y);
        }
        if response.has_focus() || response.hovered() {
            ui.input(|input| {
                for (key, code) in [
                    (egui::Key::W, winit::keyboard::KeyCode::KeyW),
                    (egui::Key::A, winit::keyboard::KeyCode::KeyA),
                    (egui::Key::S, winit::keyboard::KeyCode::KeyS),
                    (egui::Key::D, winit::keyboard::KeyCode::KeyD),
                    (egui::Key::Space, winit::keyboard::KeyCode::Space),
                ] {
                    world.set_key(code, input.key_down(key));
                }
                world.set_key(winit::keyboard::KeyCode::ControlLeft, input.modifiers.ctrl);
            });
        }
        let camera = world.camera();
        let scene = world.scene();
        let plane_material = scene.plane.material;
        let params = GpuParams {
            camera_origin: extend(camera.origin, 0.0),
            camera_forward: extend(camera.forward, 0.0),
            camera_right: extend(camera.right, 0.0),
            camera_up: extend(camera.up, 0.0),
            viewport: [
                rect.min.x * pixels_per_point,
                rect.min.y * pixels_per_point,
                rect.width() * pixels_per_point,
                rect.height() * pixels_per_point,
            ],
            scene: [
                scene.spheres.len().min(MAX_SPHERES) as u32,
                u32::from(settings.shadows_enabled),
                settings.samples_per_axis.clamp(1, 4) as u32,
                0,
            ],
            light_dir: extend([0.4662524, 0.8392543, 0.2797515], 0.0),
            plane_point: extend(scene.plane.point, 0.0),
            plane_normal: extend(scene.plane.normal, 0.0),
            plane_base_ambient: [
                plane_material.base_color[0],
                plane_material.base_color[1],
                plane_material.base_color[2],
                plane_material.ambient,
            ],
            plane_properties: [
                plane_material.diffuse,
                plane_material.specular,
                plane_material.shininess,
                plane_material.reflectivity,
            ],
            plane_shadow_ambient: [plane_material.shadow_ambient, 0.0, 0.0, 0.0],
        };
        let spheres = scene
            .spheres
            .iter()
            .take(MAX_SPHERES)
            .map(|sphere| {
                let material = sphere.material;
                GpuSphere {
                    center_radius: extend(sphere.center, sphere.radius),
                    base_ambient: [
                        material.base_color[0],
                        material.base_color[1],
                        material.base_color[2],
                        material.ambient,
                    ],
                    properties: [
                        material.diffuse,
                        material.specular,
                        material.shininess,
                        material.reflectivity,
                    ],
                    shadow_ambient: [material.shadow_ambient, 0.0, 0.0, 0.0],
                }
            })
            .collect::<Vec<_>>();

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            GpuRaytraceCallback {
                pipeline: self.pipeline.clone(),
                bind_group: self.bind_group.clone(),
                uniform_buffer: self.uniform_buffer.clone(),
                sphere_buffer: self.sphere_buffer.clone(),
                params,
                spheres,
            },
        ));
    }
}

struct GpuRaytraceCallback {
    pipeline: Arc<wgpu::RenderPipeline>,
    bind_group: Arc<wgpu::BindGroup>,
    uniform_buffer: Arc<wgpu::Buffer>,
    sphere_buffer: Arc<wgpu::Buffer>,
    params: GpuParams,
    spheres: Vec<GpuSphere>,
}

impl egui_wgpu::CallbackTrait for GpuRaytraceCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.params));
        if !self.spheres.is_empty() {
            queue.write_buffer(&self.sphere_buffer, 0, bytemuck::cast_slice(&self.spheres));
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _resources: &egui_wgpu::CallbackResources,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.bind_group.as_ref(), &[]);
        render_pass.draw(0..3, 0..1);
    }
}

fn extend(vector: [f32; 3], fourth: f32) -> [f32; 4] {
    [vector[0], vector[1], vector[2], fourth]
}

fn create_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    bytes: &[u8],
    srgb: bool,
) -> wgpu::TextureView {
    let image = image::load_from_memory(bytes)
        .expect("failed to decode bundled GPU texture")
        .to_rgba8();
    let (width, height) = image.dimensions();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: if srgb {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        },
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        image.as_raw(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    texture.create_view(&Default::default())
}
