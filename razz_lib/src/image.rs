use crate::traits::Color;
use crate::Float;

pub type Rgba = glam::Vec4;

impl Color for Rgba {
    const ZERO: Self = Self::ZERO;

    fn from_rgba(rgba: Rgba) -> Self {
        rgba
    }

    fn to_rgba(&self) -> Rgba {
        *self
    }

    fn gamma_correct(&self, num_samples: usize, gamma: Float) -> Self {
        (*self / num_samples as Float).powf(gamma)
    }
}

#[derive(Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    data: Vec<Float>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; width * height * 4],
        }
    }

    pub fn set_pixel_color(&mut self, x: usize, y: usize, color: Rgba) {
        let index = self.width * y * 4 + x * 4;
        self.data[index + 0] = color.x;
        self.data[index + 1] = color.y;
        self.data[index + 2] = color.z;
        self.data[index + 3] = color.w;
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
