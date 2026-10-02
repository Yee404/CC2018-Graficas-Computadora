use crate::material::MaterialId;
use nalgebra_glm::Vec3;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub is_intersecting: bool,
    pub material: MaterialId,
    pub u: f32,
    pub v: f32,
}

impl Intersect {
    pub fn new(
        point: Vec3,
        normal: Vec3,
        distance: f32,
        material: MaterialId,
        u: f32,
        v: f32,
    ) -> Self {
        Intersect {
            point,
            normal,
            distance,
            is_intersecting: true,
            material,
            u,
            v,
        }
    }

    pub fn empty() -> Self {
        Intersect {
            point: Vec3::zeros(),
            normal: Vec3::zeros(),
            distance: 0.0,
            is_intersecting: false,
            // Sin significado cuando `is_intersecting` es false.
            material: MaterialId::StrongholdBricks,
            u: 0.0,
            v: 0.0,
        }
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect;
}
