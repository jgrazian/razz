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

fn basic_scene_01() -> Scene {
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
    let _ground = world_builder.push_hittable(Primative::sphere(
        Vec3A::new(0.0, -100.5, -1.0),
        100.0,
        material_key,
    ));
    let _sphere = world_builder.push_hittable(Primative::sphere(
        Vec3A::new(0.0, 0.0, -1.0),
        0.5,
        material_key,
    ));
    let _tris = Triangle::vec_from(
        &vec![
            // Triangle 1
            [-2.0, 0.0, -2.0],
            [2.0, 0.0, -2.0],
            [2.0, 2.0, -2.0],
            // Triangle 2
            [2.0, 2.0, -2.001],
            [-2.0, 2.0, -2.0],
            [-2.0, 0.0, -2.0],
        ],
        material_key,
    );
    dbg!(&world_builder);
    dbg!(&_tris);
    let _mesh = Primative::mesh(_tris);
    let _mesh = world_builder.push_hittable(_mesh);

    let scene: Scene = Scene::new(world_builder.into(), camera);

    scene
}

fn basic_scene_02() -> Scene {
    let mut world_builder = WorldBuilder::default();

    let (camera, mesh) = cornell_box(&mut world_builder);
    world_builder.push_hittable(mesh);

    let blue_texture = world_builder.push_texture(Texture::Solid {
        color: Rgba::new(0.2, 0.2, 0.6, 1.0),
    });
    let metal_material = world_builder.push_material(Material::Metal {
        albedo: blue_texture,
        fuzz: 0.01,
    });
    let glass_material = world_builder.push_material(Material::Dielectric { ir: 1.7 });
    let light_texture = world_builder.push_texture(Texture::Solid {
        color: Rgba::new(1.0, 1.0, 1.0, 1.0),
    });
    let light_material = world_builder.push_material(Material::DiffuseLight {
        emit: light_texture,
    });
    world_builder.push_hittable(Primative::sphere(
        Vec3A::new(550.0 / 2.0, 220.0, 550.0 / 2.0),
        15.0,
        light_material,
    ));
    let mesh = Primative::from_obj("./obj/torus_knot.obj", metal_material);
    world_builder.push_hittable(mesh);

    let scene: Scene = Scene::new(world_builder.into(), camera);
    scene
}

fn cornell_box(world_builder: &mut WorldBuilder) -> (Camera, Primative) {
    let camera = Camera::new(
        Vec3A::new(278.0, 278.0, -800.0),
        Vec3A::new(278.0, 278.0, 0.0),
        40.0,
        1.0,
        0.0,
        10.0,
    );

    let red_texture = world_builder.push_texture(Texture::Solid {
        color: Rgba::new(0.65, 0.05, 0.05, 1.0),
    });
    let white_texture = world_builder.push_texture(Texture::Solid {
        color: Rgba::new(0.73, 0.73, 0.73, 1.0),
    });
    let green_texture = world_builder.push_texture(Texture::Solid {
        color: Rgba::new(0.12, 0.45, 0.15, 1.0),
    });
    let light_texture = world_builder.push_texture(Texture::Solid {
        color: Rgba::new(5.0, 5.0, 5.0, 1.0),
    });

    let red_material = world_builder.push_material(Material::Lambertian {
        albedo: red_texture,
    });
    let white_material = world_builder.push_material(Material::Lambertian {
        albedo: white_texture,
    });
    let green_material = world_builder.push_material(Material::Lambertian {
        albedo: green_texture,
    });
    let light_material = world_builder.push_material(Material::DiffuseLight {
        emit: light_texture,
    });

    let mut red_wall = Triangle::vec_from(
        &vec![
            [555.0, 0.0, 0.0],
            [555.0, 555.0, 0.0],
            [555.0, 555.0, 555.0],
            [555.0, 555.0, 555.0],
            [555.0, 0.0, 555.0],
            [555.0001, 0.0, 0.0],
        ],
        red_material,
    );
    let mut green_wall = Triangle::vec_from(
        &vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 555.0],
            [0.0, 555.0, 555.0],
            [0.0, 555.0, 555.0],
            [0.0, 555.0, 0.0],
            [0.0001, 0.0, 0.0],
        ],
        green_material,
    );
    let mut white_wall = Triangle::vec_from(
        &vec![
            [555.0, 0.0, 555.0],
            [0.0, 0.0, 555.0],
            [0.0, 555.0, 555.0],
            [0.0, 555.0, 555.0],
            [555.0, 555.0, 555.0],
            [555.0, 0.0, 555.0001],
        ],
        white_material,
    );
    let mut floor = Triangle::vec_from(
        &vec![
            [555.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 555.0],
            [0.0, 0.0, 555.0],
            [555.0, 0.0, 555.0],
            [555.0, 0.0001, 0.0],
        ],
        white_material,
    );
    let mut ceiling = Triangle::vec_from(
        &vec![
            [555.0, 555.0, 0.0],
            [0.0, 555.0, 0.0],
            [0.0, 555.0, 555.0],
            [0.0, 555.0, 555.0],
            [555.0, 555.0, 555.0],
            [555.0, 555.0001, 0.0],
        ],
        white_material,
    );
    let mut light = Triangle::vec_from(
        &vec![
            [213.0, 554.0, 227.0],
            [343.0, 554.0, 227.0],
            [343.0, 554.0, 332.0],
            [343.0, 554.0, 332.0],
            [213.0, 554.0, 332.0],
            [213.0, 554.0001, 227.0],
        ],
        light_material,
    );

    red_wall.append(&mut green_wall);
    red_wall.append(&mut white_wall);
    red_wall.append(&mut floor);
    red_wall.append(&mut ceiling);
    red_wall.append(&mut light);

    let mesh = Primative::mesh(red_wall);
    dbg!(&mesh);

    (camera, mesh)
}
