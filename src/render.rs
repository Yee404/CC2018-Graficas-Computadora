// render.rs
// Trazado de rayos: intersección más cercana, iluminación Phong con varias
// luces y sombras, emisión, reflexión y refracción recursivas, y skybox
// cuando un rayo (primario o secundario) no golpea geometría.

use nalgebra_glm::{normalize, Vec3};

use crate::camera::Camera;
use crate::color::Color;
use crate::config::{MAX_RECURSION_DEPTH, MIN_CONTRIBUTION, RAY_EPSILON};
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::material::MaterialLibrary;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::scene::Scene;
use crate::skybox::Skybox;
use crate::state::GameState;

/// Vista de solo lectura de la ubicación activa para el cuadro actual.
/// Solo los grupos de esta vista se intersectan.
pub struct RenderScene<'a> {
    pub groups: [&'a [Cube]; 3],
    pub lights: &'a [Light],
    pub skybox: &'a Skybox,
    pub ambient: f32,
    pub materials: &'a MaterialLibrary,
}

impl<'a> RenderScene<'a> {
    pub fn new(scene: &'a Scene, materials: &'a MaterialLibrary, state: &GameState) -> Self {
        let dragon: &'a [Cube] = if state.dragon_visible() {
            &scene.dragon
        } else {
            &[]
        };
        let exit_portal: &'a [Cube] = if state.exit_portal_active() {
            &scene.exit_portal
        } else {
            &[]
        };

        RenderScene {
            groups: [scene.objects.as_slice(), dragon, exit_portal],
            lights: &scene.lights,
            skybox: &scene.skybox,
            ambient: scene.ambient,
            materials,
        }
    }
}

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

/// Ley de Snell. Detecta entrada/salida por el signo de I·N, intercambia los
/// índices al salir y devuelve None si hay reflexión interna total.
fn refract(incident: &Vec3, normal: &Vec3, index_of_refraction: f32) -> Option<Vec3> {
    let mut cos_i = incident.dot(normal).clamp(-1.0, 1.0);
    let (eta, n) = if cos_i < 0.0 {
        cos_i = -cos_i;
        (1.0 / index_of_refraction, *normal)
    } else {
        (index_of_refraction, -*normal)
    };

    let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
    if k < 0.0 {
        None
    } else {
        Some((incident * eta + n * (eta * cos_i - k.sqrt())).normalize())
    }
}

/// Desplaza el origen de un rayo secundario hacia el lado correcto de la
/// superficie para evitar auto-intersecciones (acné).
fn offset_origin(point: &Vec3, normal: &Vec3, direction: &Vec3) -> Vec3 {
    let offset = normal * RAY_EPSILON;
    if direction.dot(normal) < 0.0 {
        point - offset
    } else {
        point + offset
    }
}

fn closest_hit(origin: &Vec3, direction: &Vec3, rs: &RenderScene) -> Intersect {
    let mut closest = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for group in rs.groups.iter() {
        for object in group.iter() {
            let hit = object.ray_intersect(origin, direction);
            if hit.is_intersecting && hit.distance < zbuffer {
                zbuffer = hit.distance;
                closest = hit;
            }
        }
    }

    closest
}

/// Intensidad de sombra en [0, 1]. Los bloqueadores transparentes dejan
/// pasar parte de la luz según su `transparency`.
fn cast_shadow(hit: &Intersect, light_dir: &Vec3, light_distance: f32, rs: &RenderScene) -> f32 {
    let origin = offset_origin(&hit.point, &hit.normal, light_dir);

    for group in rs.groups.iter() {
        for object in group.iter() {
            let blocker = object.ray_intersect(&origin, light_dir);
            if blocker.is_intersecting && blocker.distance < light_distance {
                let distance_ratio = blocker.distance / light_distance;
                let shadow = 1.0 - distance_ratio.powf(2.0).min(1.0);
                let transparency = rs.materials.get(blocker.material).transparency;
                return shadow * (1.0 - transparency);
            }
        }
    }

    0.0
}

pub fn cast_ray(origin: &Vec3, direction: &Vec3, rs: &RenderScene, depth: u32) -> Color {
    let hit = closest_hit(origin, direction, rs);
    if !hit.is_intersecting {
        return rs.skybox.sample(direction);
    }

    let material = rs.materials.get(hit.material);
    let base_color = material.base_color(hit.u, hit.v);
    let view_dir = -*direction;

    // Iluminación local: ambiente + (difuso + especular) por cada luz
    let mut surface = base_color * rs.ambient;
    for light in rs.lights.iter() {
        let to_light = light.position - hit.point;
        let light_distance = to_light.magnitude();
        let light_dir = to_light / light_distance;

        let attenuation = light.attenuation(light_distance);
        let n_dot_l = hit.normal.dot(&light_dir);
        if attenuation <= 0.0 || n_dot_l <= 0.0 {
            continue;
        }

        let shadow = cast_shadow(&hit, &light_dir, light_distance, rs);
        let intensity = light.intensity * attenuation * (1.0 - shadow);
        if intensity <= 0.0 {
            continue;
        }

        let diffuse =
            base_color.modulate(light.color) * (material.diffuse_weight * n_dot_l * intensity);

        let reflect_dir = reflect(&-light_dir, &hit.normal);
        let specular_intensity = view_dir
            .dot(&reflect_dir)
            .max(0.0)
            .powf(material.specular_exponent);
        let specular = light.color * (material.specular_weight * specular_intensity * intensity);

        surface = surface + diffuse + specular;
    }

    let emission = material.emission_color * material.emission_strength;

    let reflectivity = material.reflectivity;
    let transparency = material.transparency;
    let needs_secondary = depth < MAX_RECURSION_DEPTH
        && (reflectivity >= MIN_CONTRIBUTION || transparency >= MIN_CONTRIBUTION);
    if !needs_secondary {
        return surface + emission;
    }

    // Reflexión: R = I - 2(I·N)N
    let reflected = reflect(direction, &hit.normal).normalize();
    let reflect_color = if reflectivity >= MIN_CONTRIBUTION {
        let reflect_origin = offset_origin(&hit.point, &hit.normal, &reflected);
        cast_ray(&reflect_origin, &reflected, rs, depth + 1)
    } else {
        Color::black()
    };

    // Refracción (con reflexión interna total como respaldo)
    let refract_color = if transparency >= MIN_CONTRIBUTION {
        let refracted =
            refract(direction, &hit.normal, material.index_of_refraction).unwrap_or(reflected);
        let refract_origin = offset_origin(&hit.point, &hit.normal, &refracted);
        cast_ray(&refract_origin, &refracted, rs, depth + 1)
    } else {
        Color::black()
    };

    let surface_weight = (1.0 - reflectivity - transparency).max(0.0);
    surface * surface_weight
        + reflect_color * reflectivity
        + refract_color * transparency
        + emission
}

pub fn render(framebuffer: &mut Framebuffer, camera: &Camera, rs: &RenderScene) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let perspective_scale = (camera.fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let rotated_direction = camera.basis_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, rs, 0);

            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}
