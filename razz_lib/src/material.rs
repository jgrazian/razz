use crate::image::Rgba;
use crate::primative::HitRecord;
use crate::traits::{Color, Material, Texture};
use crate::{Float, Point3, Ray, TextureKey, Vec3};

use rand::Rng;
use slotmap::SlotMap;

pub enum ScatterResult<C: Color> {
    Scattered { ray_out: Ray, color: C },
    Absorbed,
}

#[derive(Debug)]
pub enum SimpleMaterial {
    Lambertian { albedo: TextureKey },
    Metal { albedo: TextureKey, fuzz: Float },
    Dielectric { ir: Float },
    DiffuseLight { emit: TextureKey },
}

impl<C, T> Material<C, T> for SimpleMaterial
where
    C: Color,
    T: Texture<C>,
{
    #[inline]
    fn scatter(
        &self,
        ray_in: &Ray,
        rec: &HitRecord,
        texture_map: &SlotMap<TextureKey, T>,
        rng: &mut impl Rng,
    ) -> ScatterResult<C> {
        match self {
            Self::Lambertian { albedo } => lambertian_scatter(albedo, rec, texture_map, rng),
            Self::Metal { albedo, fuzz } => {
                metal_scatter(albedo, *fuzz, ray_in, rec, texture_map, rng)
            }
            Self::Dielectric { ir } => dielectric_scatter(*ir, ray_in, rec, rng),
            Self::DiffuseLight { .. } => ScatterResult::Absorbed,
        }
    }

    #[inline]
    fn emit(&self, u: Float, v: Float, p: Point3, texture_map: &SlotMap<TextureKey, T>) -> C {
        match self {
            Self::Lambertian { .. } => C::from_rgba(Rgba::ZERO),
            Self::Metal { .. } => C::from_rgba(Rgba::ZERO),
            Self::Dielectric { .. } => C::from_rgba(Rgba::ZERO),
            Self::DiffuseLight { emit } => match texture_map.get(*emit) {
                Some(texture) => texture.value(u, v, p, texture_map),
                None => C::from_rgba(Rgba::new(1.0, 0.0, 1.0, 1.0)),
            },
        }
    }
}

impl Default for SimpleMaterial {
    fn default() -> Self {
        Self::Lambertian {
            albedo: TextureKey::default(),
        }
    }
}

#[inline(always)]
fn lambertian_scatter<C, R, T>(
    albedo: &TextureKey,
    rec: &HitRecord,
    texture_map: &SlotMap<TextureKey, T>,
    rng: &mut R,
) -> ScatterResult<C>
where
    C: Color,
    R: Rng,
    T: Texture<C>,
{
    let mut scatter_dir = rec.normal + sample_unit_sphere(rng);

    if near_zero(scatter_dir) {
        scatter_dir = rec.normal;
    }

    ScatterResult::Scattered {
        ray_out: Ray {
            origin: rec.point,
            direction: scatter_dir,
        },
        color: match texture_map.get(*albedo) {
            Some(texture) => texture.value(rec.u, rec.v, rec.point, texture_map),
            None => C::from_rgba(Rgba::new(1.0, 0.0, 1.0, 1.0)),
        },
    }
}

#[inline]
fn metal_scatter<C, R, T>(
    albedo: &TextureKey,
    fuzz: Float,
    ray_in: &Ray,
    rec: &HitRecord,
    texture_map: &SlotMap<TextureKey, T>,
    rng: &mut R,
) -> ScatterResult<C>
where
    C: Color,
    R: Rng,
    T: Texture<C>,
{
    let reflected = reflect(ray_in.direction.normalize(), rec.normal);

    let scattered = Ray {
        origin: rec.point,
        direction: reflected + fuzz * sample_unit_sphere(rng),
    };

    return if Vec3::dot(scattered.direction, rec.normal) > 0.0 {
        ScatterResult::Scattered {
            ray_out: scattered,
            color: match texture_map.get(*albedo) {
                Some(texture) => texture.value(rec.u, rec.v, rec.point, texture_map),
                None => C::from_rgba(Rgba::new(1.0, 0.0, 1.0, 1.0)),
            },
        }
    } else {
        ScatterResult::Absorbed
    };
}

#[inline]
fn dielectric_scatter<R: Rng, C: Color>(
    ir: Float,
    ray_in: &Ray,
    rec: &HitRecord,
    rng: &mut R,
) -> ScatterResult<C> {
    let refraction_ratio = if rec.front_face { 1.0 / ir } else { ir };

    let unit_dir = ray_in.direction.normalize();
    let cos_theta = Vec3::dot(-unit_dir, rec.normal).min(1.0);
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

    let cannot_refract = refraction_ratio * sin_theta > 1.0;
    let angle_criteria = reflectance(cos_theta, refraction_ratio) > rng.gen();
    let dir = if cannot_refract || angle_criteria {
        reflect(unit_dir, rec.normal)
    } else {
        refract(unit_dir, rec.normal, refraction_ratio)
    };

    ScatterResult::Scattered {
        ray_out: Ray {
            origin: rec.point,
            direction: dir,
        },
        color: C::from_rgba(Rgba::ONE),
    }
}

#[inline]
fn sample_unit_sphere<R: Rng>(rng: &mut R) -> Vec3 {
    (rng.gen::<Vec3>() - 0.5 * Vec3::ONE).normalize()
}

#[inline]
fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * Vec3::dot(v, n) * n
}

#[inline]
fn refract(v: Vec3, n: Vec3, eta: Float) -> Vec3 {
    let cos_theta = Vec3::dot(-v, n).min(1.0);
    let perp = eta * (v + cos_theta * n);
    let parallel = -((1.0 - perp.length_squared()).abs().sqrt()) * n;
    perp + parallel
}

#[inline]
fn reflectance(cosine: Float, ref_idx: Float) -> Float {
    let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

#[inline]
pub fn near_zero(v: Vec3) -> bool {
    const ETA: Float = 1e-8;
    (v.x.abs() < ETA) && (v.y.abs() < ETA) && (v.z.abs() < ETA)
}
