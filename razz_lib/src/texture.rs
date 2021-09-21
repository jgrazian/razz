use slotmap::SlotMap;

use crate::image::Rgba;
use crate::noise::*;
use crate::{Float, Point3, TextureKey};

#[derive(Debug)]
pub enum Texture {
    Solid {
        color: Rgba,
    },
    Checker {
        odd: TextureKey,
        even: TextureKey,
        scale: Float,
    },
    Noise {
        noise: Box<Noise>,
        scale: Float,
    },
}

impl Default for Texture {
    fn default() -> Self {
        Self::Solid {
            color: Rgba::splat(0.5),
        }
    }
}

impl Texture {
    pub fn value(
        &self,
        u: Float,
        v: Float,
        p: Point3,
        texture_map: &SlotMap<TextureKey, Texture>,
    ) -> Rgba {
        match self {
            Self::Solid { color } => *color,
            Self::Checker { odd, even, scale } => {
                let sines = (scale * p.x).sin() * (scale * p.y).sin() * (scale * p.z).sin();
                if sines < 0.0 {
                    match texture_map.get(*odd) {
                        Some(texture) => texture.value(u, v, p, texture_map),
                        None => Rgba::new(1.0, 0.0, 1.0, 1.0),
                    }
                } else {
                    match texture_map.get(*even) {
                        Some(texture) => texture.value(u, v, p, texture_map),
                        None => Rgba::new(1.0, 0.0, 1.0, 1.0),
                    }
                }
            }
            Self::Noise { noise, scale } => {
                Rgba::ONE * 0.5 * (1.0 + (scale * p.z + 10.0 * noise.sample(p)).sin())
            }
        }
    }
}
