use nalgebra_glm::{Vec3, dot};
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        // Vector from the ray origin to the center of the sphere
        let oc = ray_origin - self.center;

        // Coefficients for the quadratic equation
        let a = dot(ray_direction, ray_direction);
        let b = 2.0 * dot(&oc, ray_direction);
        let c = dot(&oc, &oc) - self.radius * self.radius;

        // Discriminant of the quadratic equation
        let discriminant = b * b - 4.0 * a * c;

        if discriminant > 0.0 {
            // Calculate the nearest point of intersection
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t > 0.0 {
                // Compute intersection point, normal at the intersection, and distance from the ray origin
                let point = ray_origin + ray_direction * t;
                let normal = (point - self.center).normalize();
                let distance = t;

                return Intersect::new(point, normal, distance, self.material);
            }
        }

        // If no intersection, return an empty intersect
        Intersect::empty()
    }
}



