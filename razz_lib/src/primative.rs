use crate::{Float, MaterialKey, Point3, Ray, Vec3A};

use rust_bvh::{Bounded, Bounds3A, Bvh3A, Ray3A, RayHittable};

const PI: Float = std::f64::consts::PI as Float;

#[derive(Debug, Default, Clone, Copy)]
pub struct HitRecord {
    pub point: Point3,
    pub normal: Vec3A,
    pub u: Float,
    pub v: Float,
    pub front_face: bool,
    pub material_key: MaterialKey,
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    v0: Point3,
    v1: Point3,
    v2: Point3,
}

impl From<[Point3; 3]> for Triangle {
    fn from(v: [Point3; 3]) -> Self {
        Self {
            v0: v[0].into(),
            v1: v[1].into(),
            v2: v[2].into(),
        }
    }
}

impl From<&[[f32; 3]]> for Triangle {
    fn from(v: &[[f32; 3]]) -> Self {
        Self {
            v0: v[0].into(),
            v1: v[1].into(),
            v2: v[2].into(),
        }
    }
}

impl Triangle {
    pub fn new(v0: Point3, v1: Point3, v2: Point3) -> Self {
        Self { v0, v1, v2 }
    }

    pub fn vec_from(v: &[[f32; 3]]) -> Vec<Self> {
        v.chunks_exact(3).map(|v| v.into()).collect()
    }
}

impl Bounded<Bounds3A> for Triangle {
    fn bounds(&self) -> Bounds3A {
        Bounds3A {
            min: self.v0.min(self.v1).min(self.v2),
            max: self.v0.max(self.v1).max(self.v2),
        }
    }
}

impl RayHittable<Bounds3A> for Triangle {
    type Item = HitRecord;

    fn ray_hit(&self, ray: &Ray3A, t_min: f32, t_max: f32) -> Option<(f32, Self::Item)> {
        let v0v1 = self.v1 - self.v0;
        let v0v2 = self.v2 - self.v0;
        let pvec = ray.direction.cross(v0v2);
        let det = v0v1.dot(pvec);

        if det.abs() < 0.0001 {
            return None;
        };

        let inv_det = 1.0 / det;

        let tvec = ray.origin - self.v0;
        let u = tvec.dot(pvec) * inv_det;
        if u < 0.0 || u > 1.0 {
            return None;
        };

        let qvec = tvec.cross(v0v1);
        let v = ray.direction.dot(qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        };

        let time = v0v2.dot(qvec) * inv_det;
        if time < t_min || t_max < time {
            return None;
        };

        let point = ray.at(time);
        let n = v0v1.cross(v0v2).normalize();
        let (normal, front_face) = set_front_face(ray, n);

        Some((
            time,
            HitRecord {
                point,
                normal,
                u,
                v,
                front_face,
                material_key: MaterialKey::default(),
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Primative {
    Sphere {
        center: Point3,
        radius: Float,
        material_key: MaterialKey,
    },
    Mesh {
        bvh: Bvh3A<Triangle>,
        material_key: MaterialKey,
    },
}

impl Primative {
    pub fn sphere(center: Point3, radius: Float, material_key: MaterialKey) -> Self {
        Self::Sphere {
            center,
            radius,
            material_key,
        }
    }

    pub fn mesh(triangles: Vec<Triangle>, material_key: MaterialKey) -> Self {
        Self::Mesh {
            bvh: Bvh3A::build(triangles),
            material_key,
        }
    }
}

impl Default for Primative {
    fn default() -> Self {
        Self::Sphere {
            center: Vec3A::new(0.0, 0.0, 0.0),
            radius: 1.0,
            material_key: MaterialKey::default(),
        }
    }
}

impl Bounded<Bounds3A> for Primative {
    fn bounds(&self) -> Bounds3A {
        match self {
            Self::Sphere { center, radius, .. } => Bounds3A {
                min: *center - Vec3A::splat(*radius),
                max: *center + Vec3A::splat(*radius),
            },
            Self::Mesh { bvh, .. } => bvh.bounds(),
        }
    }
}

impl RayHittable<Bounds3A> for Primative {
    type Item = HitRecord;

    fn ray_hit(&self, ray: &Ray3A, t_min: f32, t_max: f32) -> Option<(f32, Self::Item)> {
        match self {
            Self::Sphere {
                center,
                radius,
                material_key,
            } => sphere_hit(*center, *radius, ray, t_min, t_max).map(|mut t| {
                t.1.material_key = *material_key;
                t
            }),
            Self::Mesh { bvh, material_key } => bvh.ray_hit(ray, t_min, t_max).map(|mut t| {
                t.1.material_key = *material_key;
                t
            }),
        }
    }
}

#[inline(always)]
fn set_front_face(r: &Ray, outward_normal: Vec3A) -> (Vec3A, bool) {
    let front_face = Vec3A::dot(r.direction, outward_normal) < 0.0;
    if front_face {
        (outward_normal, front_face)
    } else {
        (-outward_normal, front_face)
    }
}

#[inline(always)]
fn sphere_uv(normal: &Vec3A) -> (Float, Float) {
    let theta = -normal.y.acos();
    let phi = -normal.z.atan2(normal.x) + PI;

    (phi / (2.0 * PI as Float), theta / PI)
}

#[inline(always)]
fn sphere_hit(
    center: Vec3A,
    radius: Float,
    r: &Ray,
    t_min: Float,
    t_max: Float,
) -> Option<(f32, HitRecord)> {
    let oc = r.origin - center;
    let a = r.direction.length_squared();
    let half_b = Vec3A::dot(oc, r.direction);
    let c = oc.length_squared() - radius * radius;

    let disc = half_b * half_b - a * c;
    if disc < 0.0 {
        return None;
    }
    let sqrtd = disc.sqrt();

    let mut root = (-half_b - sqrtd) / a;
    if root < t_min || t_max < root {
        root = (-half_b + sqrtd) / a;
        if root < t_min || t_max < root {
            return None;
        }
    }

    let point = r.at(root);
    let outward_normal = (point - center) / radius;
    let (normal, front_face) = set_front_face(&r, outward_normal);
    let (u, v) = sphere_uv(&normal);

    Some((
        root,
        HitRecord {
            point,
            normal,
            u,
            v,
            front_face,
            material_key: MaterialKey::default(),
        },
    ))
}

// #[inline(always)]
// fn triangle_hit(
//     v0: Point3,
//     v1: Point3,
//     v2: Point3,
//     r: &Ray,
//     t_min: Float,
//     t_max: Float,
// ) -> RaycastResult {
//     let v0v1 = v1 - v0;
//     let v0v2 = v2 - v0;
//     let pvec = r.direction.cross(v0v2);
//     let det = v0v1.dot(pvec);

//     if det.abs() < 0.0001 {
//         return RaycastResult::Miss;
//     };

//     let inv_det = 1.0 / det;

//     let tvec = r.origin - v0;
//     let u = tvec.dot(pvec) * inv_det;
//     if u < 0.0 || u > 1.0 {
//         return RaycastResult::Miss;
//     };

//     let qvec = tvec.cross(v0v1);
//     let v = r.direction.dot(qvec) * inv_det;
//     if v < 0.0 || u + v > 1.0 {
//         return RaycastResult::Miss;
//     };

//     let time = v0v2.dot(qvec) * inv_det;
//     if time < t_min || t_max < time {
//         return RaycastResult::Miss;
//     };

//     let point = r.at(time);
//     let n = v0v1.cross(v0v2).normalize();
//     let (normal, front_face) = set_front_face(r, n);

//     RaycastResult::Hit(HitRecord {
//         point,
//         normal,
//         time,
//         u,
//         v,
//         front_face,
//         material: MaterialKey::default(),
//     })
// }

#[cfg(test)]
mod test {
    // use super::*;
}
