use crate::{Float, Point3, Vec3A};

use rand::{distributions::Uniform, Rng};

#[derive(Debug)]
pub enum Noise {
    Perlin(PerlinData),
    Turbulent(PerlinData, usize),
}

impl Noise {
    pub fn perlin<T: Rng>(rng: &mut T) -> Self {
        Self::Perlin(PerlinData::new(rng))
    }

    pub fn turbulent<T: Rng>(rng: &mut T, depth: usize) -> Self {
        Self::Turbulent(PerlinData::new(rng), depth)
    }

    pub fn sample(&self, p: Point3) -> Float {
        match self {
            Self::Perlin(data) => data.noise(p),
            Self::Turbulent(data, depth) => data.turb(p, *depth),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerlinData {
    ranvec: [Vec3A; Self::POINT_COUNT],
    perm_x: [usize; Self::POINT_COUNT],
    perm_y: [usize; Self::POINT_COUNT],
    perm_z: [usize; Self::POINT_COUNT],
}

impl PerlinData {
    const POINT_COUNT: usize = 256;

    #[inline]
    pub fn new<T: Rng>(rng: &mut T) -> Self {
        let mut ranvec = [Vec3A::ZERO; Self::POINT_COUNT];
        ranvec
            .iter_mut()
            .for_each(|v| *v = (rng.gen::<Vec3A>() - 0.5 * Vec3A::ONE).normalize());

        let perm_x = Self::generate_perm(rng);
        let perm_y = Self::generate_perm(rng);
        let perm_z = Self::generate_perm(rng);

        Self {
            ranvec,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    #[inline]
    pub fn noise(&self, p: Point3) -> Float {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = p.x.floor() as isize;
        let j = p.y.floor() as isize;
        let k = p.z.floor() as isize;

        let mut c = [[[Vec3A::new(0.0, 0.0, 0.0); 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.ranvec[(self.perm_x[((i + di as isize) & 255) as usize]
                        ^ self.perm_y[((j + dj as isize) & 255) as usize]
                        ^ self.perm_z[((k + dk as isize) & 255) as usize])
                        as usize];
                }
            }
        }

        Self::perlin_interp(&c, u, v, w)
    }

    #[inline]
    fn generate_perm<T: Rng>(rng: &mut T) -> [usize; Self::POINT_COUNT] {
        let mut p = [0; Self::POINT_COUNT];
        p.iter_mut().enumerate().for_each(|(i, v)| *v = i);

        Self::permute(&mut p, Self::POINT_COUNT, rng);
        p
    }

    #[inline]
    fn permute<T: Rng>(p: &mut [usize], n: usize, rng: &mut T) {
        for i in (0..n).rev() {
            let target = rng.sample(Uniform::new_inclusive(0, i));
            let tmp = p[i];
            p[i] = p[target];
            p[target] = tmp;
        }
    }

    #[inline]
    fn perlin_interp(c: &[[[Vec3A; 2]; 2]; 2], u: Float, v: Float, w: Float) -> Float {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);

        let mut accum = 0.0;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight_v = Vec3A::new(u - i as Float, v - j as Float, w - k as Float);
                    accum += (i as Float * uu + (1 - i) as Float * (1.0 - uu))
                        * (j as Float * vv + (1 - j) as Float * (1.0 - vv))
                        * (k as Float * ww + (1 - k) as Float * (1.0 - ww))
                        * Vec3A::dot(c[i][j][k], weight_v);
                }
            }
        }

        accum
    }

    #[inline]
    pub fn turb(&self, p: Point3, depth: usize) -> Float {
        let mut accum = 0.0;
        let mut temp_p = p;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }

        accum.abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn perlin_new() {
        let mut rng = thread_rng();

        let _perlin = Noise::perlin(&mut rng);
    }
}
