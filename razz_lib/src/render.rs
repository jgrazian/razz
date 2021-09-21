use rand::Rng;

use crate::image::Image;
use crate::{Float, Scene};

#[derive(Debug)]
pub struct ProgressiveRenderer {
    width: usize,
    height: usize,
    max_ray_depth: usize,
    image: Image,
    num_samples: usize,
}

impl ProgressiveRenderer {
    pub fn new(width: usize, height: usize, max_ray_depth: usize) -> Self {
        Self {
            width,
            height,
            max_ray_depth,
            image: Image::new(width, height),
            num_samples: 0,
        }
    }
}

impl ProgressiveRenderer {
    pub fn render(&mut self, scene: &Scene, rng: &mut impl Rng) -> &Image {
        // Render 1 passes over the image
        for j in 0..self.height {
            for i in 0..self.width {
                let sample_ray = scene.sampler.get_ray(i, j, self.width, self.height, rng);
                let sample_color = scene.world.ray_color(&sample_ray, rng, self.max_ray_depth);

                let pixel_rgb = sample_color.gamma_correct(1, 2.0).to_rgba();

                if self.num_samples == 0 {
                    self.image.set_pixel_color(i, j, pixel_rgb);
                } else {
                    let old_rgb = self.image.get_pixel_color(i, j);
                    let new_rgb = (old_rgb * self.num_samples as Float + pixel_rgb)
                        * (1.0 / (self.num_samples as Float + 1.0));
                    self.image.set_pixel_color(i, j, new_rgb);
                }
            }
        }
        self.num_samples += 1;
        &self.image
    }
}
