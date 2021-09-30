use crate::Float;

use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba(glam::Vec4);

impl Rgba {
    pub const ZERO: Self = Self(glam::Vec4::ZERO);
    pub const ONE: Self = Self(glam::Vec4::ONE);

    pub fn new(r: Float, g: Float, b: Float, a: Float) -> Self {
        Self(glam::vec4(r, g, b, a))
    }

    pub fn from_rgba(rgba: Rgba) -> Self {
        rgba
    }

    pub fn to_rgba(&self) -> Rgba {
        *self
    }

    pub fn gamma_correct(&self, num_samples: usize, gamma: Float) -> Self {
        Self((self.0 / num_samples as Float).powf(gamma))
    }

    pub fn splat(v: Float) -> Self {
        Self(glam::Vec4::splat(v))
    }

    pub fn to_array(&self) -> [f32; 4] {
        self.0.into()
    }
}

impl Add for Rgba {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Mul for Rgba {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Mul<f32> for Rgba {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Float>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; width * height * 4],
        }
    }

    pub fn from_vec(width: usize, height: usize, data: Vec<f32>) -> Self {
        assert_eq!(data.len(), width * height * 4);

        Self {
            width,
            height,
            data,
        }
    }

    pub fn set_pixel_color(&mut self, x: usize, y: usize, color: Rgba) {
        let index = self.width * y * 4 + x * 4;
        self.data[index + 0] = color.0.x;
        self.data[index + 1] = color.0.y;
        self.data[index + 2] = color.0.z;
        self.data[index + 3] = color.0.w;
    }

    pub fn get_pixel_color(&self, x: usize, y: usize) -> Rgba {
        let index = self.width * y * 4 + x * 4;
        Rgba::new(
            self.data[index + 0],
            self.data[index + 1],
            self.data[index + 2],
            self.data[index + 3],
        )
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.data.len() * 4) }
    }
}
