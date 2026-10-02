use crate::material::MaterialId;
use crate::ray_intersect::{Intersect, RayIntersect};
use nalgebra_glm::{dot, Vec3};

#[allow(dead_code)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: MaterialId,
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

                // UV esférico simple (no usado por la escena actual, que solo usa Cube).
                let u = 0.5 + normal.z.atan2(normal.x) / (2.0 * std::f32::consts::PI);
                let v = 0.5 - normal.y.asin() / std::f32::consts::PI;

                return Intersect::new(point, normal, distance, self.material, u, v);
            }
        }

        // If no intersection, return an empty intersect
        Intersect::empty()
    }
}
