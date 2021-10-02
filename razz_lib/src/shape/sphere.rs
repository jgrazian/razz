use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vec3A,
    pub radius: f32,
    material_key: MaterialKey,
}

impl Sphere {
    pub fn new(center: Vec3A, radius: f32, material_key: MaterialKey) -> Sphere {
        Sphere {
            center,
            radius,
            material_key,
        }
    }
}

impl Bounded<Bounds3A> for Sphere {
    fn bounds(&self) -> Bounds3A {
        Bounds3A::new(
            self.center - Vec3A::splat(self.radius),
            self.center + Vec3A::splat(self.radius),
        )
    }
}

impl RayHittable<Bounds3A> for Sphere {
    type Item = HitRecord;

    fn ray_hit(&self, ray: &Ray3A, t_min: f32, t_max: f32) -> Option<(f32, HitRecord)> {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = Vec3A::dot(oc, ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;

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

        let point = ray.at(root);
        let normal = (point - self.center) / self.radius;
        let (face, normal) = get_face(&ray, normal);

        let theta = -normal.y.acos();
        let phi = -normal.z.atan2(normal.x) + PI;
        let u = phi / (2.0 * PI as Float);
        let v = theta / PI;

        Some((
            root,
            HitRecord {
                point,
                normal,
                u,
                v,
                face,
                material_key: self.material_key,
            },
        ))
    }
}
