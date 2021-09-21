use std::env::args;

use rand::thread_rng;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use razz_lib::*;

fn main() {
    let use_gpu = args().any(|a| a == "--gpu");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = pollster::block_on(State::new(&window, use_gpu));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(_) => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // Recreate the swap_chain if lost
                Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually request it.
            window.request_redraw();
        }
        _ => {}
    });
}

struct RenderData {
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group_layout: wgpu::BindGroupLayout,
    render_bind_groups: [wgpu::BindGroup; 2],
    render_textures: [wgpu::Texture; 2],
    render_texture_views: [wgpu::TextureView; 2],
}

enum RenderDevice {
    Cpu {
        renderer: ProgressiveRenderer,
    },
    Gpu {
        compute_pipeline: wgpu::ComputePipeline,
        compute_bind_group_layout: wgpu::BindGroupLayout,
        compute_bind_groups: [wgpu::BindGroup; 2],
    },
}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,

    render_data: RenderData,
    render_device: RenderDevice,

    scene: Scene,
    frame_number: u32,
}

// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-swapchain/
impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window, use_gpu: bool) -> Self {
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

        let (render_pipeline, render_bind_group_layout) = make_render_pipeline(&device, &sc_desc);

        let new_texture_data = Self::create_render_textures(&device, &size);
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

        let render_device = if use_gpu {
            let (compute_pipeline, compute_bind_group_layout) = make_compute_pipeline(&device);
            let buffer_bytes = [0.0f32, 0.0, 0.0, 1.0]
                .iter()
                .map(|x| x.to_ne_bytes())
                .flatten()
                .collect::<Vec<_>>();
            let compute_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("compute_buffer"),
                contents: &buffer_bytes,
                usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            });
            let compute_buffer_bind_group_entries = [wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &compute_buffer,
                    offset: 0,
                    size: None,
                }),
            }];

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

            RenderDevice::Gpu {
                compute_pipeline,
                compute_bind_group_layout,
                compute_bind_groups,
            }
        } else {
            RenderDevice::Cpu {
                renderer: ProgressiveRenderer::new(size.width as usize, size.height as usize, 5),
            }
        };

        let scene = basic_scene();

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_data,
            scene,
            render_device,
            frame_number: 0,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        let new_texture_data = Self::create_render_textures(&self.device, &self.size);
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

        match &mut self.render_device {
            RenderDevice::Cpu { ref mut renderer } => {
                *renderer =
                    ProgressiveRenderer::new(self.size.width as usize, self.size.height as usize, 5)
            }
            RenderDevice::Gpu {
                compute_bind_group_layout,
                ref mut compute_bind_groups,
                ..
            } => {
                *compute_bind_groups = [
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("gpu_bind_group"),
                        layout: &compute_bind_group_layout,
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
                        layout: &compute_bind_group_layout,
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
        }
    }

    fn create_render_textures(
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

    fn create_compute_buffers() {}

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        dbg!(self.frame_number);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut rng = thread_rng();
        match &mut self.render_device {
            RenderDevice::Cpu { renderer } => {
                self.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &self.render_data.render_textures
                            [(self.frame_number % 2) as usize],
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                    },
                    renderer.render(&self.scene, &mut rng).as_bytes(),
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new(4 * 4 * self.size.width),
                        rows_per_image: std::num::NonZeroU32::new(self.size.height),
                    },
                    wgpu::Extent3d {
                        width: self.size.width,
                        height: self.size.height,
                        depth_or_array_layers: 1,
                    },
                );
            }
            RenderDevice::Gpu {
                compute_pipeline,
                compute_bind_groups,
                ..
            } => {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute Pass"),
                });
                compute_pass.set_pipeline(&compute_pipeline);
                compute_pass.set_bind_group(
                    0,
                    &compute_bind_groups[(self.frame_number % 2) as usize],
                    &[],
                );
                compute_pass.dispatch((self.size.width + 31) / 32, (self.size.height + 31) / 32, 1);
            }
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

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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

fn make_compute_pipeline(device: &wgpu::Device) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
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

fn basic_scene() -> Scene {
    let aspect_ratio = 16.0 / 9.0;
    let camera = Camera::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -1.0),
        90.0,
        aspect_ratio,
        0.0,
        1.0,
    );

    let mut world = World::default();
    let texture = world.push_texture(Texture::default());
    let material = world.push_material(Material::Lambertian { albedo: texture });
    let _ground = world.push_hittable(Primative::Sphere {
        center: Vec3::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material,
    });
    let _sphere = world.push_hittable(Primative::Sphere {
        center: Vec3::new(0.0, 0.0, -1.0),
        radius: 0.5,
        material,
    });

    let scene: Scene = Scene::new(world, camera);

    scene
}
