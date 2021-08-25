use std::cmp::Ordering;

use crate::traits::Hittable;
use crate::{Float, MaterialKey, Point3, Ray, Vec3};

const PI: Float = std::f64::consts::PI as Float;

#[derive(Debug, Clone, Copy)]
pub enum RaycastResult {
    Hit(HitRecord),
    Miss,
}

impl RaycastResult {
    fn with_material(mut self, mat: MaterialKey) -> Self {
        match self {
            Self::Hit(ref mut rec) => {
                rec.material = mat;
            }
            Self::Miss => (),
        }
        self
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct HitRecord {
    pub point: Point3,
    pub normal: Vec3,
    pub time: Float,
    pub u: Float,
    pub v: Float,
    pub front_face: bool,
    pub material: MaterialKey,
}

#[derive(Debug, Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn compare(&self, box1: &BoundingBox, box2: &BoundingBox) -> Ordering {
        match self {
            Self::X => {
                if box1.min.x < box2.min.x {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            Self::Y => {
                if box1.min.y < box2.min.y {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            Self::Z => {
                if box1.min.z < box2.min.z {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct BoundingBox {
    pub min: Point3,
    pub max: Point3,
}

impl Hittable for BoundingBox {
    #[inline(always)]
    fn hit(&self, ray_in: &Ray, t_min: Float, t_max: Float) -> RaycastResult {
        let mut loc_t_min = t_min;
        let mut loc_t_max = t_max;

        for a in 0..3 {
            let inv_d = 1.0 / ray_in.direction[a];
            let mut t0 = (self.min[a] - ray_in.origin[a]) * inv_d;
            let mut t1 = (self.max[a] - ray_in.origin[a]) * inv_d;

            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            loc_t_min = if t0 > t_min { t0 } else { loc_t_min };
            loc_t_max = if t1 < t_max { t1 } else { loc_t_max };

            if loc_t_max <= loc_t_min {
                return RaycastResult::Miss;
            }
        }
        RaycastResult::Hit(HitRecord::default())
    }

    fn bounds(&self) -> BoundingBox {
        *self
    }
}

impl BoundingBox {
    pub fn union(box1: Self, box2: Self) -> Self {
        let min = Vec3::min(box1.min, box2.min);
        let max = Vec3::max(box1.max, box2.max);

        Self { min, max }
    }

    pub fn centroid(&self) -> Point3 {
        0.5 * self.min + self.max
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Primative {
    Sphere {
        center: Point3,
        radius: Float,
        material: MaterialKey,
    },
}

impl Primative {
    pub fn sphere(center: Point3, radius: Float, material: MaterialKey) -> Self {
        Self::Sphere {
            center,
            radius,
            material,
        }
    }
}

impl Default for Primative {
    fn default() -> Self {
        Self::Sphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 1.0,
            material: MaterialKey::default(),
        }
    }
}

impl Hittable for Primative {
    #[inline(always)]
    fn hit(&self, ray_in: &Ray, t_min: Float, t_max: Float) -> RaycastResult {
        match self {
            Self::Sphere {
                center,
                radius,
                material,
            } => sphere_hit(*center, *radius, ray_in, t_min, t_max).with_material(*material),
        }
    }

    fn bounds(&self) -> BoundingBox {
        match self {
            Self::Sphere { center, radius, .. } => BoundingBox {
                min: *center - Vec3::splat(*radius),
                max: *center + Vec3::splat(*radius),
            },
        }
    }
}

#[inline(always)]
fn set_front_face(r: &Ray, outward_normal: Vec3) -> (Vec3, bool) {
    let front_face = Vec3::dot(r.direction, outward_normal) < 0.0;
    if front_face {
        (outward_normal, front_face)
    } else {
        (-outward_normal, front_face)
    }
}

#[inline(always)]
fn sphere_uv(normal: &Vec3) -> (Float, Float) {
    let theta = -normal.y.acos();
    let phi = -normal.z.atan2(normal.x) + PI;

    (phi / (2.0 * PI as Float), theta / PI)
}

#[inline(always)]
fn sphere_hit(center: Vec3, radius: Float, r: &Ray, t_min: Float, t_max: Float) -> RaycastResult {
    let oc = r.origin - center;
    let a = r.direction.length_squared();
    let half_b = Vec3::dot(oc, r.direction);
    let c = oc.length_squared() - radius * radius;

    let disc = half_b * half_b - a * c;
    if disc < 0.0 {
        return RaycastResult::Miss;
    }
    let sqrtd = disc.sqrt();

    let mut root = (-half_b - sqrtd) / a;
    if root < t_min || t_max < root {
        root = (-half_b + sqrtd) / a;
        if root < t_min || t_max < root {
            return RaycastResult::Miss;
        }
    }

    let time = root;
    let point = r.at(root);
    let outward_normal = (point - center) / radius;
    let (normal, front_face) = set_front_face(r, outward_normal);
    let (u, v) = sphere_uv(&normal);

    RaycastResult::Hit(HitRecord {
        point,
        normal,
        time,
        u,
        v,
        front_face,
        material: MaterialKey::default(),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hittable_bounds() {
        let box1 = BoundingBox {
            min: -1.0 * Vec3::ONE,
            max: Vec3::ONE,
        };
        assert_eq!(box1, box1.bounds());

        let sphere = Primative::sphere(Vec3::ZERO, 1.0, MaterialKey::default());
        assert_eq!(sphere.bounds(), box1);
    }

    #[test]
    fn hittable_hit() {
        let box1 = BoundingBox {
            min: -1.0 * Vec3::ONE,
            max: Vec3::ONE,
        };
        let sphere = Primative::sphere(Vec3::ZERO, 1.0, MaterialKey::default());
        let ray = Ray {
            origin: Vec3::new(-3.0, 0.0, 0.0),
            direction: Vec3::new(1.0, 0.0, 0.0),
        };
        let ray2 = Ray {
            origin: Vec3::new(-3.0, 0.0, 0.0),
            direction: Vec3::new(-1.0, 0.0, 0.0),
        };

        match box1.hit(&ray, 0.0, Float::INFINITY) {
            RaycastResult::Hit(_) => assert!(true),
            _ => assert!(false),
        }
        match sphere.hit(&ray, 0.0, Float::INFINITY) {
            RaycastResult::Hit(rec) => assert_eq!(rec.time, 2.0),
            _ => assert!(false),
        }
        match sphere.hit(&ray2, 0.0, Float::INFINITY) {
            RaycastResult::Hit(_) => assert!(false),
            RaycastResult::Miss => assert!(true),
        }
    }

    #[test]
    fn axis_compare() {
        let box1 = BoundingBox {
            min: -1.0 * Vec3::ONE,
            max: Vec3::ONE,
        };
        let box2 = BoundingBox {
            min: 2.0 * Vec3::ONE,
            max: 3.0 * Vec3::ONE,
        };

        assert_eq!(Axis::X.compare(&box1, &box2), Ordering::Less);
        assert_eq!(Axis::Y.compare(&box1, &box2), Ordering::Less);
        assert_eq!(Axis::Z.compare(&box1, &box2), Ordering::Less);
    }
}
