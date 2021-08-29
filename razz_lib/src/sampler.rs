use rand::Rng;

use crate::{Float, Ray, Vec3};

use crate::traits::Sampler;

#[derive(Default, Debug)]
pub struct BasicCamera {
    origin: Vec3,
    top_right: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    lens_radius: Float,
    ar: Float,

    u: Vec3,
    v: Vec3,
    w: Vec3,
}

impl Sampler for BasicCamera {
    fn get_ray(
        &self,
        pixel_x: usize,
        pixel_y: usize,
        width: usize,
        height: usize,
        rng: &mut impl Rng,
    ) -> Ray {
        let u: Float = (pixel_x as Float + rng.gen::<Float>()) / ((width - 1) as Float);
        let v: Float = (pixel_y as Float + rng.gen::<Float>()) / ((height - 1) as Float);

        Ray {
            origin: self.origin,
            direction: self.top_right + (u * self.horizontal) - (v * self.vertical) - self.origin,
        }
    }
}

impl BasicCamera {
    pub fn new(
        from: Vec3,
        at: Vec3,
        vfov: Float,
        ar: Float,
        aperture: Float,
        focus_dist: Float,
    ) -> Self {
        let theta = vfov.to_radians();
        let h = (theta * 0.5).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = ar * viewport_height;

        let w = (from - at).normalize();
        let u = Vec3::cross(Vec3::new(0.0, 1.0, 0.0), w).normalize();
        let v = Vec3::cross(w, u);

        let origin = from;
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
