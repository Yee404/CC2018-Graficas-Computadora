use nalgebra_glm::Vec3;

use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
}

impl Cube {
    pub fn new(center: Vec3, size: f32, material: Material) -> Self{
        let half_size = size / 2.0;
        let offset = Vec3::new(half_size, half_size, half_size);

        Self {
            min: center - offset,
            max: center + offset,
            material,
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(
        &self,
        ray_origin: &Vec3,
        ray_direction: &Vec3
    ) -> Intersect {
        let mut t_near = f32::NEG_INFINITY;
        let mut t_far = f32::INFINITY;

        let mut near_normal = Vec3::new(0.0, 0.0, 0.0);
        let mut far_normal = Vec3::new(0.0, 0.0, 0.0);

        for axis in 0..3 {
            if ray_direction[axis].abs() < 1e-6 {
                if ray_origin[axis] < self.min[axis] || ray_origin[axis] > self.max[axis]
                {
                    return Intersect::empty();
                }

                continue;
            }

            let mut t1 =
            (self.min[axis] - ray_origin[axis]) / ray_direction[axis];

            let mut t2 =
            (self.max[axis] - ray_origin[axis]) / ray_direction[axis];

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

        Intersect::new(
            point,
            normal,
            distance,
            self.material,
        )
        
    }
}



