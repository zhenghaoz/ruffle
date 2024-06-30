use crate::filters::{FilterVertex, Filters};
use crate::layouts::BindLayouts;
use crate::pipelines::VERTEX_BUFFERS_DESCRIPTION_POS;
use crate::shaders::Shaders;
use crate::{
    create_buffer_with_data, BitmapSamplers, Pipelines, PosColorVertex, PosVertex,
    TextureTransforms, DEFAULT_COLOR_ADJUSTMENTS,
};
use fnv::FnvHashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

pub struct Descriptors {
    pub wgpu_instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub limits: wgpu::Limits,
    pub queue: wgpu::Queue,
    pub bitmap_samplers: BitmapSamplers,
    pub bind_layouts: BindLayouts,
    pub quad: Quad,
    copy_pipeline: Mutex<FnvHashMap<(u32, wgpu::TextureFormat), Arc<wgpu::RenderPipeline>>>,
    copy_srgb_pipeline: Mutex<FnvHashMap<(u32, wgpu::TextureFormat), Arc<wgpu::RenderPipeline>>>,
    pub shaders: Shaders,
    pipelines: Mutex<FnvHashMap<(u32, wgpu::TextureFormat), Arc<Pipelines>>>,
    pub default_color_bind_group: wgpu::BindGroup,
    pub filters: Filters,
}

impl Debug for Descriptors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Descriptors")
    }
}

impl Descriptors {
    pub fn new(
        instance: wgpu::Instance,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Self {
        let limits = device.limits();
        let bind_layouts = BindLayouts::new(&device);
        let bitmap_samplers = BitmapSamplers::new(&device);
        let shaders = Shaders::new(&device);
        let quad = Quad::new(&device);
        let default_color_transform = create_buffer_with_data(
            &device,
            bytemuck::cast_slice(&[DEFAULT_COLOR_ADJUSTMENTS]),
            wgpu::BufferUsages::UNIFORM,
            create_debug_label!("Default colors"),
        );
        let default_color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: create_debug_label!("Default colors").as_deref(),
            layout: &bind_layouts.color_transforms,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: default_color_transform.as_entire_binding(),
            }],
        });
        let filters = Filters::new(&device);

        Self {
            wgpu_instance: instance,
            adapter,
            device,
            limits,
            queue,
            bitmap_samplers,
            bind_layouts,
            quad,
            copy_pipeline: Default::default(),
            copy_srgb_pipeline: Default::default(),
            shaders,
            pipelines: Default::default(),
            default_color_bind_group,
            filters,
        }
    }

    pub fn copy_srgb_pipeline(
        &self,
        format: wgpu::TextureFormat,
        msaa_sample_count: u32,
    ) -> Arc<wgpu::RenderPipeline> {
        let mut pipelines = self
            .copy_srgb_pipeline
            .lock()
            .expect("Pipelines should not be already locked");
        pipelines
            .entry((msaa_sample_count, format))
            .or_insert_with(|| {
                let copy_texture_pipeline_layout =
                    &self
                        .device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: create_debug_label!("Copy sRGB pipeline layout").as_deref(),
                            bind_group_layouts: &[
                                &self.bind_layouts.globals,
                                &self.bind_layouts.transforms,
                                &self.bind_layouts.bitmap,
                            ],
                            push_constant_ranges: &[],
                        });
                Arc::new(
                    self.device
                        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: create_debug_label!("Copy sRGB pipeline").as_deref(),
                            layout: Some(copy_texture_pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: &self.shaders.copy_srgb_shader,
                                entry_point: "main_vertex",
                                buffers: &VERTEX_BUFFERS_DESCRIPTION_POS,
                                compilation_options: Default::default(),
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &self.shaders.copy_srgb_shader,
                                entry_point: "main_fragment",
                                targets: &[Some(wgpu::ColorTargetState {
                                    format,
                                    // All of our blending has been done by now, so we want
                                    // to overwrite the target pixels without any blending
                                    blend: Some(wgpu::BlendState::REPLACE),
                                    write_mask: Default::default(),
                                })],
                                compilation_options: Default::default(),
                            }),
                            primitive: wgpu::PrimitiveState {
                                topology: wgpu::PrimitiveTopology::TriangleList,
                                strip_index_format: None,
                                front_face: wgpu::FrontFace::Ccw,
                                cull_mode: None,
                                polygon_mode: wgpu::PolygonMode::default(),
                                unclipped_depth: false,
                                conservative: false,
                            },
                            depth_stencil: None,
                            multisample: wgpu::MultisampleState {
                                count: msaa_sample_count,
                                mask: !0,
                                alpha_to_coverage_enabled: false,
                            },
                            multiview: None,
                        }),
                )
            })
            .clone()
    }

    pub fn copy_pipeline(
        &self,
        format: wgpu::TextureFormat,
        msaa_sample_count: u32,
    ) -> Arc<wgpu::RenderPipeline> {
        let mut pipelines = self
            .copy_pipeline
            .lock()
            .expect("Pipelines should not be already locked");
        pipelines
            .entry((msaa_sample_count, format))
            .or_insert_with(|| {
                let copy_texture_pipeline_layout =
                    &self
                        .device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: create_debug_label!("Copy pipeline layout").as_deref(),
                            bind_group_layouts: &[
                                &self.bind_layouts.globals,
                                &self.bind_layouts.transforms,
                                &self.bind_layouts.bitmap,
                            ],
                            push_constant_ranges: &[],
                        });
                Arc::new(
                    self.device
                        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: create_debug_label!("Copy pipeline").as_deref(),
                            layout: Some(copy_texture_pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: &self.shaders.copy_shader,
                                entry_point: "main_vertex",
                                buffers: &VERTEX_BUFFERS_DESCRIPTION_POS,
                                compilation_options: Default::default(),
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &self.shaders.copy_shader,
                                entry_point: "main_fragment",
                                targets: &[Some(wgpu::ColorTargetState {
                                    format,
                                    // All of our blending has been done by now, so we want
                                    // to overwrite the target pixels without any blending
                                    blend: Some(wgpu::BlendState::REPLACE),
                                    write_mask: Default::default(),
                                })],
                                compilation_options: Default::default(),
                            }),
                            primitive: wgpu::PrimitiveState {
                                topology: wgpu::PrimitiveTopology::TriangleList,
                                strip_index_format: None,
                                front_face: wgpu::FrontFace::Ccw,
                                cull_mode: None,
                                polygon_mode: wgpu::PolygonMode::default(),
                                unclipped_depth: false,
                                conservative: false,
                            },
                            depth_stencil: None,
                            multisample: wgpu::MultisampleState {
                                count: msaa_sample_count,
                                mask: !0,
                                alpha_to_coverage_enabled: false,
                            },
                            multiview: None,
                        }),
                )
            })
            .clone()
    }

    pub fn pipelines(&self, msaa_sample_count: u32, format: wgpu::TextureFormat) -> Arc<Pipelines> {
        let mut pipelines = self
            .pipelines
            .lock()
            .expect("Pipelines should not be already locked");
        pipelines
            .entry((msaa_sample_count, format))
            .or_insert_with(|| {
                Arc::new(Pipelines::new(
                    &self.device,
                    &self.shaders,
                    format,
                    msaa_sample_count,
                    &self.bind_layouts,
                ))
            })
            .clone()
    }
}

pub struct Quad {
    pub vertices_pos: wgpu::Buffer,
    pub vertices_pos_color: wgpu::Buffer,
    pub filter_vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub texture_transforms: wgpu::Buffer,
}

impl Quad {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertices_pos = [
            PosVertex {
                position: [0.0, 0.0],
            },
            PosVertex {
                position: [1.0, 0.0],
            },
            PosVertex {
                position: [1.0, 1.0],
            },
            PosVertex {
                position: [0.0, 1.0],
            },
        ];
        let vertices_pos_color = [
            PosColorVertex {
                position: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            PosColorVertex {
                position: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            PosColorVertex {
                position: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            PosColorVertex {
                position: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];
        let filter_vertices = [
            FilterVertex {
                position: [0.0, 0.0],
                uv: [0.0, 0.0],
            },
            FilterVertex {
                position: [1.0, 0.0],
                uv: [1.0, 0.0],
            },
            FilterVertex {
                position: [1.0, 1.0],
                uv: [1.0, 1.0],
            },
            FilterVertex {
                position: [0.0, 1.0],
                uv: [0.0, 1.0],
            },
        ];
        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        let vbo_pos = create_buffer_with_data(
            device,
            bytemuck::cast_slice(&vertices_pos),
            wgpu::BufferUsages::VERTEX,
            create_debug_label!("Quad vbo (pos)"),
        );

        let vbo_pos_color = create_buffer_with_data(
            device,
            bytemuck::cast_slice(&vertices_pos_color),
            wgpu::BufferUsages::VERTEX,
            create_debug_label!("Quad vbo (pos & color)"),
        );

        let vbo_filter = create_buffer_with_data(
            device,
            bytemuck::cast_slice(&filter_vertices),
            wgpu::BufferUsages::VERTEX,
            create_debug_label!("Quad vbo (filter)"),
        );

        let ibo = create_buffer_with_data(
            device,
            bytemuck::cast_slice(&indices),
            wgpu::BufferUsages::INDEX,
            create_debug_label!("Quad ibo"),
        );

        let tex_transforms = create_buffer_with_data(
            device,
            bytemuck::cast_slice(&[TextureTransforms {
                u_matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            }]),
            wgpu::BufferUsages::UNIFORM,
            create_debug_label!("Quad tex transforms"),
        );

        Self {
            vertices_pos: vbo_pos,
            vertices_pos_color: vbo_pos_color,
            filter_vertices: vbo_filter,
            indices: ibo,
            texture_transforms: tex_transforms,
        }
    }
}
