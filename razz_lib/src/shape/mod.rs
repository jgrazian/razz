mod mesh;
mod sphere;

use std::{fmt::Debug, path::Path};

use crate::{Float, MaterialKey, Point3, Ray3A, Vec3A};
pub use mesh::{Mesh, Triangle};
pub use sphere::Sphere;

use boxtree::{Bounded, Bounds3A, Bvh3A, RayHittable};
use tobj;

const PI: Float = std::f64::consts::PI as Float;

#[inline(always)]
fn get_face(ray: &Ray3A, normal: Vec3A) -> (Face, Vec3A) {
    if Vec3A::dot(ray.direction, normal) < 0.0 {
        (Face::Front, normal)
    } else {
        (Face::Back, -normal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Front,
    Back,
}

#[derive(Debug, Clone, Copy)]
pub struct HitRecord {
    pub point: Point3,
    pub normal: Vec3A,
    pub u: Float,
    pub v: Float,
    pub face: Face,
    pub material_key: MaterialKey,
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3A,
    pub rotation: glam::Quat,
    pub scale: Float,
}

#[derive(Debug, Clone)]
pub enum Primative {
    Sphere(Sphere),
    Mesh(Mesh),
}

impl Primative {
    pub fn sphere(center: Point3, radius: Float, material_key: MaterialKey) -> Self {
        Self::Sphere(Sphere::new(center, radius, material_key))
    }

    pub fn mesh(triangles: Vec<Triangle>) -> Self {
        Self::Mesh(Mesh::new(triangles))
    }

    pub fn from_obj(path: impl AsRef<Path> + Debug, material_key: MaterialKey) -> Self {
        Self::Mesh(Mesh::from_obj(path, material_key))
    }
}

impl Default for Primative {
    fn default() -> Self {
        Self::Sphere(Sphere::new(
            Vec3A::new(0.0, 0.0, 0.0),
            1.0,
            MaterialKey::default(),
        ))
    }
}

impl Bounded<Bounds3A> for Primative {
    fn bounds(&self) -> Bounds3A {
        match self {
            Self::Sphere(s) => s.bounds(),
            Self::Mesh(m) => m.bounds(),
        }
    }
}

impl RayHittable<Bounds3A> for Primative {
    type Item = HitRecord;

    fn ray_hit(&self, ray: &Ray3A, t_min: f32, t_max: f32) -> Option<(f32, Self::Item)> {
        match self {
            Self::Sphere(s) => s.ray_hit(ray, t_min, t_max).map(|t| t),
            Self::Mesh(m) => m.ray_hit(ray, t_min, t_max).map(|t| t),
        }
    }
}
