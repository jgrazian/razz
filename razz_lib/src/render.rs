use crate::image::Image;
use crate::{Float, Scene};

use rand::Rng;
use rayon::prelude::*;

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

#[derive(Debug)]
pub struct ParallelRenderer {
    width: usize,
    height: usize,
    max_ray_depth: usize,
    image: Image,
    num_samples: usize,
}

impl ParallelRenderer {
    pub fn new(width: usize, height: usize, max_ray_depth: usize) -> Self {
        Self {
            width,
            height,
            max_ray_depth,
            image: Image::new(width, height),
            num_samples: 0,
        }
    }

    pub fn render(&mut self, scene: &Scene) -> &Image {
        // Render 1 passes over the image
        let img_data: Vec<f32> = (0..self.height)
            .into_par_iter()
            .flat_map(|j| {
                let mut rng = rand::thread_rng();

                (0..self.width)
                    .into_iter()
                    .flat_map(|i| {
                        let sample_ray =
                            scene
                                .sampler
                                .get_ray(i, j, self.width, self.height, &mut rng);
                        let sample_color =
                            scene
                                .world
                                .ray_color(&sample_ray, &mut rng, self.max_ray_depth);

                        let pixel_rgb = sample_color.gamma_correct(1, 2.0).to_rgba();
                        pixel_rgb.to_array()
                    })
                    .collect::<Vec<f32>>()
            })
            .collect();

        if self.num_samples == 0 {
            self.image.data = img_data;
        } else {
            let num_samples_float = self.num_samples as Float;

            self.image
                .data
                .iter_mut()
                .zip(img_data)
                .for_each(|(old, new)| {
                    *old = (*old * num_samples_float + new) * (1.0 / (num_samples_float + 1.0))
                });
        }

        self.num_samples += 1;
        &self.image
    }
}
