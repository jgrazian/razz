mod camera;
mod image;
mod material;
mod noise;
mod primative;
mod render;
mod texture;
mod traits;

pub use glam::Vec3A as Vec3;
use rand::Rng;
use slotmap::{new_key_type, SlotMap};

pub use camera::*;
pub use image::*;
pub use material::*;
pub use primative::*;
pub use render::*;
pub use texture::*;
pub use traits::*;

pub type Point3 = Vec3;
pub type Float = f32;

new_key_type! { pub struct PrimativeKey; }
new_key_type! { pub struct MaterialKey; }
new_key_type! { pub struct TextureKey; }

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    #[inline]
    fn at(&self, t: Float) -> Point3 {
        self.origin + t * self.direction
    }
}

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
pub struct World {
    textures: SlotMap<TextureKey, Texture>,
    materials: SlotMap<MaterialKey, Material>,
    hittables: HittableCollection,
}

impl World {
    fn ray_color(&self, ray_in: &Ray, rng: &mut impl Rng, depth: usize) -> Rgba {
        if depth <= 0 {
            return Rgba::ZERO;
        }

        match self.hittables.hit(ray_in, 0.001, Float::INFINITY) {
            RaycastResult::Hit(rec) => {
                let material = self
                    .materials
                    .get(rec.material)
                    .expect("No material found!");
                let emitted = material.emit(rec.u, rec.v, rec.point, &self.textures);

                match material.scatter(ray_in, &rec, &self.textures, rng) {
                    ScatterResult::Scattered { ray_out, color } => {
                        emitted + color * self.ray_color(&ray_out, rng, depth - 1)
                    }
                    ScatterResult::Absorbed => emitted,
                }
            }
            RaycastResult::Miss => Rgba::ONE,
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
pub enum HittableCollection {
    List { primatives: Vec<Primative> },
}

impl HittableCollection {
    fn push(&mut self, obj: Primative) {
        match self {
            HittableCollection::List { primatives } => primatives.push(obj),
        }
    }
}

impl Default for HittableCollection {
    fn default() -> Self {
        Self::List {
            primatives: Vec::new(),
        }
    }
}

impl Hittable for HittableCollection {
    fn hit(&self, ray_in: &Ray, t_min: f32, t_max: f32) -> RaycastResult {
        let mut result = RaycastResult::Miss;
        let mut closest_time = t_max;

        match self {
            HittableCollection::List { primatives } => {
                for primative in primatives {
                    match primative.hit(ray_in, t_min, closest_time) {
                        RaycastResult::Hit(rec) => {
                            closest_time = rec.time;
                            result = RaycastResult::Hit(rec);
                        }
                        RaycastResult::Miss => (),
                    }
                }
            }
        }

        result
    }

    fn bounds(&self) -> BoundingBox {
        let mut bounds = BoundingBox {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        };

        match self {
            HittableCollection::List { primatives } => {
                for primative in primatives {
                    bounds = BoundingBox::union(bounds, primative.bounds());
                }
            }
        }
        bounds
    }
}
