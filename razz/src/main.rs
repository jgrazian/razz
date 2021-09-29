mod cpu;
mod gpu;

use cpu::CpuState;
use gpu::GpuState;

use std::env::args;

use razz_lib::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = match args().any(|a| a == "--gpu") {
        true => StateType::Gpu(pollster::block_on(GpuState::new(&window))),
        false => StateType::Cpu(pollster::block_on(CpuState::new(&window))),
    };

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
                Err(wgpu::SwapChainError::Lost) => state.resize(state.size()),
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

trait State {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    fn input(&mut self, event: &WindowEvent) -> bool;
    fn update(&mut self);
    fn render(&mut self) -> Result<(), wgpu::SwapChainError>;
    fn size(&self) -> winit::dpi::PhysicalSize<u32>;
}

struct RenderData {
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group_layout: wgpu::BindGroupLayout,
    render_bind_groups: [wgpu::BindGroup; 2],
    render_textures: [wgpu::Texture; 2],
    render_texture_views: [wgpu::TextureView; 2],
}

enum StateType {
    Cpu(CpuState),
    Gpu(GpuState),
}

impl State for StateType {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        match self {
            StateType::Cpu(state) => state.resize(new_size),
            StateType::Gpu(state) => state.resize(new_size),
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match self {
            StateType::Cpu(state) => state.input(event),
            StateType::Gpu(state) => state.input(event),
        }
    }

    fn update(&mut self) {
        match self {
            StateType::Cpu(state) => state.update(),
            StateType::Gpu(state) => state.update(),
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        match self {
            StateType::Cpu(state) => state.render(),
            StateType::Gpu(state) => state.render(),
        }
    }

    fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        match self {
            StateType::Cpu(state) => state.size(),
            StateType::Gpu(state) => state.size(),
        }
    }
}

fn basic_scene() -> Scene {
    let aspect_ratio = 16.0 / 9.0;
    let camera = Camera::new(
        Vec3A::new(0.0, 0.0, 0.0),
        Vec3A::new(0.0, 0.0, -1.0),
        90.0,
        aspect_ratio,
        0.0,
        1.0,
    );

    let mut world_builder = WorldBuilder::default();
    let texture = world_builder.push_texture(Texture::default());
    let material_key = world_builder.push_material(Material::Lambertian { albedo: texture });
    let _ground = world_builder.push_hittable(Primative::Sphere {
        center: Vec3A::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material_key,
    });
    let _sphere = world_builder.push_hittable(Primative::Sphere {
        center: Vec3A::new(0.0, 0.0, -1.0),
        radius: 0.5,
        material_key,
    });
    let _tris = Triangle::vec_from(&vec![
        // Triangle 1
        [-2.0, 0.0, -2.0],
        [2.0, 0.0, -2.0],
        [2.0, 2.0, -2.0],
        // Triangle 2
        [2.0, 2.0, -2.001],
        [-2.0, 2.0, -2.0],
        [-2.0, 0.0, -2.0],
    ]);
    let _mesh = Primative::mesh(_tris, material_key);
    let _mesh = world_builder.push_hittable(_mesh);

    let scene: Scene = Scene::new(world_builder.into(), camera);

    scene
}
