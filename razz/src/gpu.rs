use crate::{basic_scene_01, RenderData, State};

use rand::thread_rng;
use razz_lib::Scene;
use winit::{event::*, window::Window};

struct ComputeData {
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group_layout: wgpu::BindGroupLayout,
    compute_bind_groups: [wgpu::BindGroup; 2],
}

pub struct GpuState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,

    render_data: RenderData,
    compute_data: ComputeData,

    _scene: Scene,
    frame_number: u32,
}

// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-swapchain/
impl GpuState {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let (render_pipeline, render_bind_group_layout) =
            Self::make_render_pipeline(&device, &sc_desc);

        let new_texture_data = Self::make_render_textures(&device, &size);
        let render_textures = new_texture_data.0;
        let render_texture_views = new_texture_data.1;

        let render_bind_groups = [
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("render_bind_group_0"),
                layout: &render_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&render_texture_views[0]),
                }],
            }),
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("render_bind_group_1"),
                layout: &render_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&render_texture_views[1]),
                }],
            }),
        ];

        let render_data = RenderData {
            render_pipeline,
            render_bind_group_layout,
            render_bind_groups,
            render_textures,
            render_texture_views,
        };

        let (compute_pipeline, compute_bind_group_layout) = Self::make_compute_pipeline(&device);
        // let buffer_bytes = [0.0f32, 0.0, 0.0, 1.0]
        //     .iter()
        //     .map(|x| x.to_ne_bytes())
        //     .flatten()
        //     .collect::<Vec<_>>();
        // let compute_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("compute_buffer"),
        //     contents: &buffer_bytes,
        //     usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
        // });
        // let compute_buffer_bind_group_entries = [wgpu::BindGroupEntry {
        //     binding: 2,
        //     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        //         buffer: &compute_buffer,
        //         offset: 0,
        //         size: None,
        //     }),
        // }];

        dbg!("Making compute bind groups.");
        let compute_bind_groups = [
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("gpu_bind_group"),
                layout: &compute_bind_group_layout,
                entries: &[
                    // Output texture, goes to the render texture
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &render_data.render_texture_views[0],
                        ),
                    },
                    // Input texture, from previous iteration
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &render_data.render_texture_views[1],
                        ),
                    },
                ],
            }),
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("gpu_bind_group"),
                layout: &compute_bind_group_layout,
                entries: &[
                    // Output texture, goes to the render texture
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &render_data.render_texture_views[1],
                        ),
                    },
                    // Input texture, from previous iteration
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &render_data.render_texture_views[0],
                        ),
                    },
                ],
            }),
        ];

        let compute_data = ComputeData {
            compute_pipeline,
            compute_bind_group_layout,
            compute_bind_groups,
        };

        let _scene = basic_scene_01();

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_data,
            compute_data,
            _scene,
            frame_number: 0,
        }
    }

    fn make_render_textures(
        device: &wgpu::Device,
        size: &winit::dpi::PhysicalSize<u32>,
    ) -> ([wgpu::Texture; 2], [wgpu::TextureView; 2]) {
        let textures = [
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Image"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsage::STORAGE
                    | wgpu::TextureUsage::COPY_DST
                    | wgpu::TextureUsage::COPY_SRC,
            }),
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Image"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsage::STORAGE
                    | wgpu::TextureUsage::COPY_DST
                    | wgpu::TextureUsage::COPY_SRC,
            }),
        ];
        let texture_views = [
            textures[0].create_view(&wgpu::TextureViewDescriptor::default()),
            textures[1].create_view(&wgpu::TextureViewDescriptor::default()),
        ];

        (textures, texture_views)
    }

    fn make_render_pipeline(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Render"),
            flags: wgpu::ShaderFlags::all(),
            source: wgpu::ShaderSource::Wgsl(include_str!("render.wgsl").into()),
        });

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        format: wgpu::TextureFormat::Rgba32Float,
                    },
                    count: None,
                }],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        (render_pipeline, render_bind_group_layout)
    }

    fn make_compute_pipeline(
        device: &wgpu::Device,
    ) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Compute"),
            flags: wgpu::ShaderFlags::all(),
            source: wgpu::ShaderSource::Wgsl(include_str!("compute.wgsl").into()),
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            format: wgpu::TextureFormat::Rgba32Float,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadOnly,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            format: wgpu::TextureFormat::Rgba32Float,
                        },
                        count: None,
                    },
                ],
            });

        dbg!("Making compute pipeline.");
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("gpu_pipeline"),
            module: &shader,
            entry_point: "main",
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("gpu_pipeline_layout"),
                    bind_group_layouts: &[&compute_bind_group_layout],
                    push_constant_ranges: &[],
                }),
            ),
        });

        (compute_pipeline, compute_bind_group_layout)
    }
}

impl State for GpuState {
    fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        let new_texture_data = Self::make_render_textures(&self.device, &self.size);
        self.render_data.render_textures = new_texture_data.0;
        self.render_data.render_texture_views = new_texture_data.1;

        self.render_data.render_bind_groups = [
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("render_bind_group_0"),
                layout: &self.render_data.render_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.render_data.render_texture_views[0],
                    ),
                }],
            }),
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("render_bind_group_1"),
                layout: &self.render_data.render_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.render_data.render_texture_views[1],
                    ),
                }],
            }),
        ];

        self.compute_data.compute_bind_groups = [
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("gpu_bind_group"),
                layout: &self.compute_data.compute_bind_group_layout,
                entries: &[
                    // Output texture, goes to the render texture
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &self.render_data.render_texture_views[0],
                        ),
                    },
                    // Input texture, from previous iteration
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &self.render_data.render_texture_views[1],
                        ),
                    },
                ],
            }),
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("gpu_bind_group"),
                layout: &self.compute_data.compute_bind_group_layout,
                entries: &[
                    // Output texture, goes to the render texture
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &self.render_data.render_texture_views[1],
                        ),
                    },
                    // Input texture, from previous iteration
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &self.render_data.render_texture_views[0],
                        ),
                    },
                ],
            }),
        ]
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        if self.frame_number % 10 == 0 {
            println!("Frame number: {}", self.frame_number);
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut _rng = thread_rng();
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });
            compute_pass.set_pipeline(&self.compute_data.compute_pipeline);
            compute_pass.set_bind_group(
                0,
                &self.compute_data.compute_bind_groups[(self.frame_number % 2) as usize],
                &[],
            );
            compute_pass.dispatch((self.size.width + 31) / 32, (self.size.height + 31) / 32, 1);
        }

        let frame = self.swap_chain.get_current_frame()?.output;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_data.render_pipeline);
            render_pass.set_bind_group(
                0,
                &self.render_data.render_bind_groups[(self.frame_number % 2) as usize],
                &[],
            );
            render_pass.draw(0..3, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        self.frame_number += 1;

        Ok(())
    }
}
