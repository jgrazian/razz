mod image;
mod material;
mod noise;
mod primative;
mod render;
mod sampler;
mod texture;
mod traits;

use std::marker::PhantomData;

pub use glam::Vec3A as Vec3;
use rand::Rng;
use slotmap::{new_key_type, SlotMap};

pub use image::*;
pub use material::*;
pub use primative::*;
pub use render::*;
pub use sampler::*;
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

pub struct Scene<C, T, M, H, S>
where
    C: Color,
    T: Texture<C>,
    M: Material<C, T>,
    H: HittableCollection,
    S: Sampler,
{
    pub world: World<C, T, M, H>,
    pub sampler: S,
    _color: PhantomData<C>,
}

impl<C, T, M, H, S> Scene<C, T, M, H, S>
where
    C: Color,
    T: Texture<C>,
    M: Material<C, T>,
    H: HittableCollection,
    S: Sampler,
{
    pub fn new(world: World<C, T, M, H>, sampler: S) -> Self {
        Self {
            world,
            sampler,
            _color: PhantomData,
        }
    }
}

#[derive(Default, Debug)]
pub struct World<C, T, M, H>
where
    C: Color,
    T: Texture<C>,
    M: Material<C, T>,
    H: HittableCollection,
{
    textures: SlotMap<TextureKey, T>,
    materials: SlotMap<MaterialKey, M>,
    hittables: H,
    _color: PhantomData<C>,
}

impl<C, H, M, T> World<C, T, M, H>
where
    C: Color,
    T: Texture<C>,
    M: Material<C, T>,
    H: HittableCollection,
{
    fn ray_color(&self, ray_in: &Ray, rng: &mut impl Rng, depth: usize) -> C {
        if depth <= 0 {
            return C::from_rgba(Rgba::ZERO);
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
            RaycastResult::Miss => C::from_rgba(Rgba::ONE),
        }
    }

    pub fn push_texture(&mut self, texture: T) -> TextureKey {
        self.textures.insert(texture)
    }

    pub fn push_material(&mut self, material: M) -> MaterialKey {
        self.materials.insert(material)
    }

    pub fn push_hittable(&mut self, primative: H::Object) {
        self.hittables.push(primative)
    }
}

#[derive(Default, Debug)]
pub struct HittableList<H: Hittable> {
    objects: Vec<H>,
}

impl<H: Hittable> HittableCollection for HittableList<H> {
    type Object = H;
    fn push(&mut self, obj: Self::Object) {
        self.objects.push(obj)
    }
}

impl<H: Hittable> Hittable for HittableList<H> {
    fn hit(&self, ray_in: &Ray, t_min: f32, t_max: f32) -> RaycastResult {
        let mut result = RaycastResult::Miss;
        let mut closest_time = t_max;

        for obj in &self.objects {
            match obj.hit(ray_in, t_min, closest_time) {
                RaycastResult::Hit(rec) => {
                    closest_time = rec.time;
                    result = RaycastResult::Hit(rec);
                }
                RaycastResult::Miss => {}
            }
        }

        result
    }

    fn bounds(&self) -> BoundingBox {
        let mut bounds = BoundingBox {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        };
        for obj in &self.objects {
            bounds = BoundingBox::union(bounds, obj.bounds());
        }
        bounds
    }
}
