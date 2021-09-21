use std::fmt::Debug;

use crate::primative::{BoundingBox, RaycastResult};
use crate::{Float, Ray};

pub trait Hittable: Debug + Default {
    fn hit(&self, ray_in: &Ray, t_min: Float, t_max: Float) -> RaycastResult;
    fn bounds(&self) -> BoundingBox;
}
