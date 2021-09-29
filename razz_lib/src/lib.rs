mod camera;
mod image;
mod material;
mod noise;
mod primative;
mod render;
mod texture;
mod traits;

use rand::Rng;
use rust_bvh::{Bvh3A, RayHittable};
use slotmap::{new_key_type, SlotMap};

pub use camera::*;
pub use image::*;
pub use material::*;
pub use primative::*;
pub use render::*;
pub use texture::*;
pub use traits::*;

pub use glam::Vec3A;
pub type Point3 = Vec3A;
pub type Ray = rust_bvh::Ray3A;
pub type Float = f32;

new_key_type! { pub struct PrimativeKey; }
new_key_type! { pub struct MaterialKey; }
new_key_type! { pub struct TextureKey; }

pub struct Scene {
    pub world: World,
    pub sampler: Camera,
}

impl Scene {
    pub fn new(world: World, sampler: Camera) -> Self {
        Self { world, sampler }
    }
}

#[derive(Default, Debug)]
pub struct WorldBuilder {
    textures: SlotMap<TextureKey, Texture>,
    materials: SlotMap<MaterialKey, Material>,
    hittables: Vec<Primative>,
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self {
            textures: SlotMap::default(),
            materials: SlotMap::default(),
            hittables: Vec::new(),
        }
    }

    pub fn push_texture(&mut self, texture: Texture) -> TextureKey {
        self.textures.insert(texture)
    }

    pub fn push_material(&mut self, material: Material) -> MaterialKey {
        self.materials.insert(material)
    }

    pub fn push_hittable(&mut self, primative: Primative) {
        self.hittables.push(primative)
    }
}

#[derive(Debug)]
pub struct World {
    textures: SlotMap<TextureKey, Texture>,
    materials: SlotMap<MaterialKey, Material>,
    bvh: Bvh3A<Primative>,
}

impl World {
    fn ray_color(&self, ray_in: &Ray, rng: &mut impl Rng, depth: usize) -> Rgba {
        if depth <= 0 {
            return Rgba::ZERO;
        }

        match self.bvh.ray_hit(ray_in, 0.001, Float::INFINITY) {
            Some((_, hit_rec)) => {
                let material = self
                    .materials
                    .get(hit_rec.material_key)
                    .expect("No material found!");
                let emitted = material.emit(hit_rec.u, hit_rec.v, hit_rec.point, &self.textures);

                match material.scatter(ray_in, &hit_rec, &self.textures, rng) {
                    ScatterResult::Scattered { ray_out, color } => {
                        emitted + color * self.ray_color(&ray_out, rng, depth - 1)
                    }
                    ScatterResult::Absorbed => emitted,
                }
            }
            None => Rgba::ONE,
        }
    }
}

impl From<WorldBuilder> for World {
    fn from(builder: WorldBuilder) -> Self {
        Self {
            textures: builder.textures,
            materials: builder.materials,
            bvh: Bvh3A::build(builder.hittables),
        }
    }
}
