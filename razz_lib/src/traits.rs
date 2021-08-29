use std::fmt::Debug;
use std::ops::{Add, Mul};

use rand::Rng;
use slotmap::SlotMap;

use crate::image::{Image, Rgba};
use crate::material::ScatterResult;
use crate::primative::{BoundingBox, HitRecord, RaycastResult};
use crate::{Float, Point3, Ray, TextureKey};

use crate::Scene;

pub trait Color: Debug + Default + Add<Self, Output = Self> + Mul<Self, Output = Self> {
    const ZERO: Self;
    fn to_rgba(&self) -> Rgba;
    fn from_rgba(rgba: Rgba) -> Self;
    fn gamma_correct(&self, num_samples: usize, gamma: Float) -> Self;
}

pub trait Texture<C>: Debug + Default
where
    C: Color,
{
    fn value<T: Texture<C>>(
        &self,
        u: Float,
        v: Float,
        p: Point3,
        texture_map: &SlotMap<TextureKey, T>,
    ) -> C;
}

pub trait Material<C, T>: Debug + Default
where
    C: Color,
    T: Texture<C>,
{
    fn scatter(
        &self,
        ray_in: &Ray,
        rec: &HitRecord,
        texture_map: &SlotMap<TextureKey, T>,
        rng: &mut impl Rng,
    ) -> ScatterResult<C>;

    fn emit(&self, u: Float, v: Float, p: Point3, texture_map: &SlotMap<TextureKey, T>) -> C;
}

pub trait Hittable: Debug + Default {
    fn hit(&self, ray_in: &Ray, t_min: Float, t_max: Float) -> RaycastResult;
    fn bounds(&self) -> BoundingBox;
}

pub trait Sampler: Debug + Default {
    fn get_ray(
        &self,
        pixel_x: usize,
        pixel_y: usize,
        width: usize,
        height: usize,
        rng: &mut impl Rng,
    ) -> Ray;
}

pub trait Renderer<C, T, M, H, S>: Debug
where
    C: Color,
    T: Texture<C>,
    M: Material<C, T>,
    H: Hittable,
    S: Sampler,
{
    fn render(&mut self, scene: &Scene<C, T, M, H, S>, rng: &mut impl Rng) -> &Image;
}
