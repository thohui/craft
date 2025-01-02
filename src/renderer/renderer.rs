use std::{borrow::Cow, sync::Arc};

use bytemuck::Pod;
use cgmath::Vector2;
use wgpu::{
    BindGroupLayoutDescriptor, CommandEncoder, RenderPass, RenderPassDescriptor, SamplerDescriptor,
    Texture,
};
use winit::window::Window;

use crate::camera::{self, CameraUniform};

use super::{
    block::{BlockVertex, TerrainMesh},
    buffer,
};

pub struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    device: Arc<wgpu::Device>,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    resolution: Vector2<u32>,

    camera_buffer: buffer::DynamicBuffer<camera::CameraUniform>,

    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,

    depth_texture: super::texture::Texture,

    terrain_pipeline: TerrainPipeline,
    terrain_texture: super::texture::Texture,
    terrain_bind_group_layout: wgpu::BindGroupLayout,
    terrain_bind_group: wgpu::BindGroup,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });
        let surface = instance
            .create_surface(window)
            .expect("Failed to create surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let texture_format = surface_caps.formats[0];

        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_configuration);

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Uniform,
                    },
                    count: None,
                }],
            });
        let camera_buffer = buffer::DynamicBuffer::new(&device, 1, wgpu::BufferUsages::UNIFORM);

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.buf().buf.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });

        let terrain_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let terrain_atlas = include_bytes!("../../assets/terrain.png");

        let terrain_texture = crate::renderer::texture::Texture::from_bytes(
            &device,
            &queue,
            terrain_atlas,
            "Terrain Texture",
        )
        .unwrap();

        let terrain_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &terrain_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&terrain_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&terrain_texture.sampler),
                },
            ],
            label: Some("Texture Bind Group"),
        });

        let terrain_pipeline = TerrainPipeline::new(
            &BindGroups {
                camera: &camera_bind_group,
                terrain: &terrain_bind_group,
            },
            &BindGroupLayouts {
                camera: &camera_bind_group_layout,
                terrain: &terrain_bind_group_layout,
            },
            &device,
            texture_format,
        );

        let depth_texture = super::texture::Texture::create_depth_texture(
            &device,
            &surface_configuration,
            "Depth texture",
        );

        Self {
            surface,
            queue,
            surface_config: surface_configuration,
            size,
            terrain_pipeline,
            resolution: Vector2::new(size.width, size.height),
            camera_buffer,
            device: Arc::new(device),

            depth_texture,

            camera_bind_group_layout,
            camera_bind_group,
            terrain_texture,
            terrain_bind_group_layout,
            terrain_bind_group,
        }
    }

    pub fn on_resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.size = size;
        self.resolution = Vector2::new(size.width, size.height);
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);

        self.depth_texture = super::texture::Texture::create_depth_texture(
            &self.device,
            &self.surface_config,
            "Depth texture",
        );
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn update_camera_uniform(&mut self, camera: CameraUniform) {
        self.camera_buffer.update(&self.queue, &[camera], 0);
    }

    pub fn camera_buffer(&self) -> &wgpu::Buffer {
        &self.camera_buffer.buf().buf
    }

    pub fn bind_group_layouts(&self) -> BindGroupLayouts {
        BindGroupLayouts {
            camera: &self.camera_bind_group_layout,
            terrain: &self.terrain_bind_group_layout,
        }
    }

    pub fn bind_groups(&self) -> BindGroups {
        BindGroups {
            camera: &self.camera_bind_group,
            terrain: &self.terrain_bind_group,
        }
    }

    pub fn draw_terrain(&mut self, mesh: &TerrainMesh) -> anyhow::Result<()> {
        let surface = self.surface.get_current_texture()?;

        let surface_view = surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Terrain Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        let bind_groups = self.bind_groups();
        render_pass.set_bind_group(0, bind_groups.camera, &[]);
        render_pass.set_bind_group(1, bind_groups.terrain, &[]);
        render_pass.set_pipeline(&self.terrain_pipeline.pipeline);

        let vertices = mesh.vertices();
        let indices = mesh.indices();

        let vertex = super::buffer::Buffer::new(&self.device, wgpu::BufferUsages::VERTEX, vertices);

        let index = super::buffer::Buffer::new(&self.device, wgpu::BufferUsages::INDEX, indices);

        render_pass.set_vertex_buffer(0, vertex.buf.slice(..));
        render_pass.set_index_buffer(index.buf.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);

        drop(render_pass);
        let buffer = encoder.finish();
        self.queue.submit(std::iter::once(buffer));
        surface.present();

        Ok(())
    }
}

#[derive(Debug)]
pub struct BindGroupLayouts<'a> {
    pub camera: &'a wgpu::BindGroupLayout,
    pub terrain: &'a wgpu::BindGroupLayout,
}

#[derive(Debug)]
pub struct BindGroups<'a> {
    pub camera: &'a wgpu::BindGroup,
    pub terrain: &'a wgpu::BindGroup,
}

#[derive(Debug)]
pub struct TerrainPipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl TerrainPipeline {
    pub fn new(
        bind_groups: &BindGroups,
        bind_group_layouts: &BindGroupLayouts,
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let shader_src = include_str!("../../assets/shaders/terrain.wgsl");

        let vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Terrain vertex shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_src)),
        });

        let fragment = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Terrain fragment shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_src)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Terrain Pipeline Layout"),
            bind_group_layouts: &[bind_group_layouts.camera, bind_group_layouts.terrain],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            vertex: wgpu::VertexState {
                module: &vertex,
                entry_point: Some("vs_main"),
                buffers: &[BlockVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            cache: None,
            label: Some("Terrain Pipeline"),
            layout: Some(&pipeline_layout),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            multiview: None,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
        });

        Self { pipeline }
    }
}
