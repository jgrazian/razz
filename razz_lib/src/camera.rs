use crate::{Float, Ray3A, Vec3A};

use rand::Rng;

#[derive(Default, Debug)]
pub struct Camera {
    origin: Vec3A,
    top_right: Vec3A,
    horizontal: Vec3A,
    vertical: Vec3A,
    lens_radius: Float,
    ar: Float,

    u: Vec3A,
    v: Vec3A,
    w: Vec3A,
}

impl Camera {
    pub fn get_ray(
        &self,
        pixel_x: usize,
        pixel_y: usize,
        width: usize,
        height: usize,
        rng: &mut impl Rng,
    ) -> Ray3A {
        let u: Float = (pixel_x as Float + rng.gen::<Float>()) / ((width - 1) as Float);
        let v: Float = (pixel_y as Float + rng.gen::<Float>()) / ((height - 1) as Float);

        Ray3A {
            origin: self.origin,
            direction: self.top_right + (u * self.horizontal) - (v * self.vertical) - self.origin,
        }
    }
}

impl Camera {
    pub fn new(
        look_from: Vec3A,
        look_at: Vec3A,
        vfov: Float,
        ar: Float,
        aperture: Float,
        focus_dist: Float,
    ) -> Self {
        let theta = vfov.to_radians();
        let h = (theta * 0.5).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = ar * viewport_height;

        let w = (look_from - look_at).normalize();
        let u = Vec3A::cross(Vec3A::new(0.0, 1.0, 0.0), w).normalize();
        let v = Vec3A::cross(w, u);

        let origin = look_from;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let top_right = origin - (0.5 * horizontal) + (0.5 * vertical) - focus_dist * w;

        Self {
            origin,
            horizontal,
            vertical,
            top_right,
            lens_radius: 0.5 * aperture,
            ar,
            u,
            v,
            w,
        }
    }
}
