use super::*;
use std::sync::Arc;

use glam::Affine3A;

#[derive(Debug, Clone)]
pub struct Triangle {
    mesh: Arc<Mesh>,
    index: usize,
}

impl Triangle {
    fn vertices(&self) -> (Point3, Point3, Point3) {
        let (i0, i1, i2) = self.mesh.indices[self.index];
        let v0 = self.mesh.vertices[i0];
        let v1 = self.mesh.vertices[i1];
        let v2 = self.mesh.vertices[i2];

        (v0, v1, v2)
    }
}

impl Bounded<Bounds3A> for Triangle {
    fn bounds(&self) -> Bounds3A {
        let (v0, v1, v2) = self.vertices();

        Bounds3A {
            min: v0.min(v1).min(v2),
            max: v0.max(v1).max(v2),
        }
    }
}

impl RayHittable<Bounds3A> for Triangle {
    type Item = HitRecord;

    fn ray_hit(&self, ray: &Ray3A, t_min: f32, t_max: f32) -> Option<(f32, Self::Item)> {
        let (v0, v1, v2) = self.vertices();

        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        let pvec = ray.direction.cross(v0v2);
        let det = v0v1.dot(pvec);

        if det.abs() < 0.0001 {
            return None;
        };

        let inv_det = 1.0 / det;

        let tvec = ray.origin - v0;
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
                material_key: self.mesh.material_key,
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    bvh: Bvh3A<Triangle>,

    vertices: Vec<Point3>,
    indices: Vec<(usize, usize, usize)>,

    material_key: MaterialKey,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Point3>,
        indices: Vec<(usize, usize, usize)>,
        material_key: MaterialKey,
    ) -> Arc<Self> {
        let mesh = Self {
            bvh: Bvh3A::build(vec![]),
            vertices,
            indices,
            material_key,
        };

        let mesh = Arc::new(mesh);
        let triangles = (0..mesh.indices.len())
            .map(|i| Triangle {
                mesh: Arc::clone(&mesh),
                index: i,
            })
            .collect();
        let bvh = Bvh3A::build(triangles);

        // SAFTEY: This is safe. Only mutate once during construction to set the bvh.
        unsafe {
            let ptr = Arc::as_ptr(&mesh) as *mut mesh::Mesh;
            let mesh = &mut *ptr;
            mesh.bvh = bvh;
        }

        mesh
    }

    pub fn from_obj(path: impl AsRef<Path> + Debug, material_key: MaterialKey) -> Arc<Self> {
        let affine = Affine3A::from_scale_rotation_translation(
            glam::Vec3::splat(10.0),
            glam::Quat::from_rotation_x(3.14159 / 2.0),
            glam::Vec3::new(550.0 / 2.0, 220.0, 550.0 / 2.0),
        );
        let obj = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        );

        let (models, _) = obj.expect("Failed to load OBJ file");

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for model in models {
            let mesh = &model.mesh;

            let mesh_indices: Vec<_> = mesh
                .indices
                .chunks(3)
                .map(|c| (c[0] as usize, c[1] as usize, c[2] as usize))
                .collect();
            let mesh_vertices: Vec<_> = mesh
                .positions
                .chunks(3)
                .map(|c| affine.transform_point3a(Point3::new(c[0], c[1], c[2])))
                .collect();

            indices.extend(mesh_indices);
            vertices.extend(mesh_vertices);
        }

        Self::new(vertices, indices, material_key)
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
