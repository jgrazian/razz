use super::*;

use glam::Affine3A;

#[derive(Debug, Clone)]
pub struct Triangle {
    v0: Vec3A,
    v1: Vec3A,
    v2: Vec3A,
    material_key: MaterialKey,
}

impl Triangle {
    pub fn new(v0: Point3, v1: Point3, v2: Point3, material_key: MaterialKey) -> Self {
        Self {
            v0,
            v1,
            v2,
            material_key,
        }
    }

    pub fn vec_from(v: &[[f32; 3]], material_key: MaterialKey) -> Vec<Self> {
        v.chunks_exact(3)
            .map(|v| (v, material_key).into())
            .collect()
    }
}

impl Bounded<Bounds3A> for Triangle {
    fn bounds(&self) -> Bounds3A {
        Bounds3A {
            min: self.v0.min(self.v1).min(self.v2),
            max: self.v0.max(self.v1).max(self.v2),
        }
    }
}

impl RayHittable<Bounds3A> for Triangle {
    type Item = HitRecord;

    fn ray_hit(&self, ray: &Ray3A, t_min: f32, t_max: f32) -> Option<(f32, Self::Item)> {
        let v0v1 = self.v1 - self.v0;
        let v0v2 = self.v2 - self.v0;
        let pvec = ray.direction.cross(v0v2);
        let det = v0v1.dot(pvec);

        if det.abs() < 0.0001 {
            return None;
        };

        let inv_det = 1.0 / det;

        let tvec = ray.origin - self.v0;
        let u = tvec.dot(pvec) * inv_det;
        if u < 0.0 || u > 1.0 {
            return None;
        };

        let qvec = tvec.cross(v0v1);
        let v = ray.direction.dot(qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        };

        let time = v0v2.dot(qvec) * inv_det;
        if time < t_min || t_max < time {
            return None;
        };

        let point = ray.at(time);
        let normal = v0v1.cross(v0v2).normalize();
        let (face, normal) = get_face(ray, normal);

        Some((
            time,
            HitRecord {
                point,
                normal,
                u,
                v,
                face,
                material_key: self.material_key,
            },
        ))
    }
}

impl From<([Point3; 3], MaterialKey)> for Triangle {
    fn from(v: ([Point3; 3], MaterialKey)) -> Self {
        Self {
            v0: v.0[0].into(),
            v1: v.0[1].into(),
            v2: v.0[2].into(),
            material_key: v.1,
        }
    }
}

impl From<(&[[f32; 3]], MaterialKey)> for Triangle {
    fn from(v: (&[[f32; 3]], MaterialKey)) -> Self {
        Self {
            v0: v.0[0].into(),
            v1: v.0[1].into(),
            v2: v.0[2].into(),
            material_key: v.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    bvh: Bvh3A<Triangle>,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        Self {
            bvh: Bvh3A::build(triangles),
        }
    }

    pub fn from_obj(path: impl AsRef<Path> + Debug, material_key: MaterialKey) -> Self {
        let affine = Affine3A::from_scale_rotation_translation(
            glam::Vec3::splat(10.0),
            glam::Quat::from_rotation_x(3.14159 / 2.0),
            glam::Vec3::new(550.0 / 2.0, 220.0, 550.0 / 2.0),
        );
        let obj = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                single_index: false,
                triangulate: false,
                ..Default::default()
            },
        );

        let (models, _) = obj.expect("Failed to load OBJ file");

        let mut triangles = Vec::new();
        for model in models {
            let mesh = &model.mesh;

            let verts: Vec<_> = mesh
                .indices
                .iter()
                .map(|c| {
                    [
                        mesh.positions[*c as usize * 3 + 0],
                        mesh.positions[*c as usize * 3 + 1],
                        mesh.positions[*c as usize * 3 + 2],
                    ]
                })
                .collect();

            triangles.extend(Triangle::vec_from(&verts, material_key));
        }

        triangles.iter_mut().for_each(|t| {
            t.v0 = affine.transform_point3a(t.v0);
            t.v1 = affine.transform_point3a(t.v1);
            t.v2 = affine.transform_point3a(t.v2)
        });

        Self {
            bvh: Bvh3A::build(triangles),
        }
    }
}

impl Bounded<Bounds3A> for Mesh {
    fn bounds(&self) -> Bounds3A {
        self.bvh.bounds()
    }
}

impl RayHittable<Bounds3A> for Mesh {
    type Item = HitRecord;

    fn ray_hit(&self, ray: &Ray3A, t_min: f32, t_max: f32) -> Option<(f32, Self::Item)> {
        self.bvh.ray_hit(ray, t_min, t_max)
    }
}
// #[derive(Debug, Clone)]
// pub struct Mesh {
//     pub indices: Vec<(usize, usize, usize)>,
//     pub vertices: Vec<Point3>,
//     pub normals: Option<Vec<Vec3A>>,
//     pub tangents: Option<Vec<Vec3A>>,
//     pub uvs: Option<Vec<Vec2>>,
// }

// impl Mesh {
//     pub fn new(
//         indices: Vec<(usize, usize, usize)>,
//         vertices: Vec<Point3>,
//         normals: Option<Vec<Vec3A>>,
//         tangents: Option<Vec<Vec3A>>,
//         uvs: Option<Vec<Vec2>>,
//     ) -> Self {
//         Self {
//             indices,
//             vertices,
//             normals,
//             tangents,
//             uvs,
//         }
//     }
// }
