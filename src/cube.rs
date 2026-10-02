use nalgebra_glm::Vec3;

use crate::material::MaterialId;
use crate::ray_intersect::{Intersect, RayIntersect};

/// Cómo se calculan las UV de cada cara.
#[derive(Debug, Clone, Copy)]
pub enum UvMapping {
    /// La textura completa se estira sobre la cara (objetos pequeños).
    Stretch,
    /// La textura se repite cada `n` unidades de mundo (pisos, paredes, isla).
    Tile(f32),
    /// Como `Tile`, pero la textura es un atlas vertical de tres regiones
    /// (de arriba hacia abajo en la imagen: tapa, lado, fondo). Permite que
    /// el césped tenga tapa verde y lados con tierra.
    TopSideBottom(f32),
}

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: MaterialId,
    pub uv: UvMapping,
}

impl Cube {
    #[allow(dead_code)]
    pub fn new(center: Vec3, size: f32, material: MaterialId) -> Self {
        let half_size = size / 2.0;
        let offset = Vec3::new(half_size, half_size, half_size);

        Self {
            min: center - offset,
            max: center + offset,
            material,
            uv: UvMapping::Stretch,
        }
    }

    /// Cuboide alineado a los ejes definido por dos esquinas opuestas.
    pub fn from_bounds(a: Vec3, b: Vec3, material: MaterialId) -> Self {
        Self {
            min: Vec3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            max: Vec3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
            material,
            uv: UvMapping::Tile(1.0),
        }
    }

    /// Cuboide definido por su centro y su tamaño en cada eje.
    pub fn cuboid(center: Vec3, size: Vec3, material: MaterialId) -> Self {
        let half = size * 0.5;
        Self::from_bounds(center - half, center + half, material)
    }

    pub fn stretched(mut self) -> Self {
        self.uv = UvMapping::Stretch;
        self
    }

    pub fn with_tile_size(mut self, tile_size: f32) -> Self {
        self.uv = UvMapping::Tile(tile_size.max(1e-3));
        self
    }

    pub fn top_side_bottom(mut self) -> Self {
        self.uv = UvMapping::TopSideBottom(1.0);
        self
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let mut t_near = f32::NEG_INFINITY;
        let mut t_far = f32::INFINITY;

        let mut near_normal = Vec3::new(0.0, 0.0, 0.0);
        let mut far_normal = Vec3::new(0.0, 0.0, 0.0);

        for axis in 0..3 {
            if ray_direction[axis].abs() < 1e-6 {
                if ray_origin[axis] < self.min[axis] || ray_origin[axis] > self.max[axis] {
                    return Intersect::empty();
                }

                continue;
            }

            let mut t1 = (self.min[axis] - ray_origin[axis]) / ray_direction[axis];

            let mut t2 = (self.max[axis] - ray_origin[axis]) / ray_direction[axis];

            let mut normal1 = Vec3::new(0.0, 0.0, 0.0);
            let mut normal2 = Vec3::new(0.0, 0.0, 0.0);

            normal1[axis] = -1.0;
            normal2[axis] = 1.0;

            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
                std::mem::swap(&mut normal1, &mut normal2);
            }

            if t1 > t_near {
                t_near = t1;
                near_normal = normal1;
            }

            if t2 < t_far {
                t_far = t2;
                far_normal = normal2;
            }

            if t_near > t_far {
                return Intersect::empty();
            }
        }

        let (distance, normal) = if t_near > 1e-4 {
            (t_near, near_normal)
        } else if t_far > 1e-4 {
            (t_far, far_normal)
        } else {
            return Intersect::empty();
        };

        let point = ray_origin + ray_direction * distance;
        let (u, v) = self.face_uv(&point, &normal);

        Intersect::new(point, normal, distance, self.material, u, v)
    }
}

impl Cube {
    /// UV de la cara golpeada: ±X usa Z/Y, ±Y usa X/Z, ±Z usa X/Y, con
    /// inversiones para que las caras opuestas no queden espejadas.
    /// En modo `Tile` se usan coordenadas de mundo, por lo que cuboides
    /// vecinos continúan la textura sin costuras.
    fn face_uv(&self, point: &Vec3, normal: &Vec3) -> (f32, f32) {
        let (fx, fy, fz) = match self.uv {
            UvMapping::Stretch => {
                let size = self.max - self.min;
                let local = point - self.min;
                (
                    if size.x > 1e-6 { local.x / size.x } else { 0.0 },
                    if size.y > 1e-6 { local.y / size.y } else { 0.0 },
                    if size.z > 1e-6 { local.z / size.z } else { 0.0 },
                )
            }
            UvMapping::Tile(tile_size) | UvMapping::TopSideBottom(tile_size) => (
                point.x / tile_size,
                point.y / tile_size,
                point.z / tile_size,
            ),
        };

        let (u, v) = if normal.x.abs() > 0.5 {
            if normal.x > 0.0 {
                (1.0 - fz, fy)
            } else {
                (fz, fy)
            }
        } else if normal.y.abs() > 0.5 {
            if normal.y > 0.0 {
                (fx, 1.0 - fz)
            } else {
                (fx, fz)
            }
        } else if normal.z > 0.0 {
            (fx, fy)
        } else {
            (1.0 - fx, fy)
        };

        match self.uv {
            UvMapping::TopSideBottom(_) => {
                // v en [0, 1/3) = fondo, [1/3, 2/3) = lado, [2/3, 1) = tapa.
                let region = if normal.y > 0.5 {
                    2.0
                } else if normal.y < -0.5 {
                    0.0
                } else {
                    1.0
                };
                let local_v = v.rem_euclid(1.0).min(0.999);
                (u, (local_v + region) / 3.0)
            }
            _ => (u, v),
        }
    }
}
