use nalgebra_glm::{Vec3, normalize};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;

mod framebuffer;
mod ray_intersect;
mod sphere; 
mod cube;
mod color;
mod camera;
mod light;
mod material;

use framebuffer::Framebuffer;
//use sphere::Sphere;       esta línea es para utilizar la esfera
use cube::Cube;
use color::Color;
use ray_intersect::{Intersect, RayIntersect};
use camera::Camera;
use light::Light;
use material::Material;

const SHADOW_BIAS: f32 = 1e-4;

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    //objects: &[Sphere],       esto es para la esfera
    objects: &[Cube],
) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let offset_normal = intersect.normal * SHADOW_BIAS;
    let shadow_ray_origin = if light_dir.dot(&intersect.normal) < 0.0 {
        intersect.point - offset_normal
    } else {
        intersect.point + offset_normal
    };

    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            let distance_ratio = shadow_intersect.distance / light_distance;
            shadow_intensity = 1.0 - distance_ratio.powf(2.0).min(1.0);
            break;
        }
    }

    shadow_intensity
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    //objects: &[Sphere],       esto es para el cubo
    objects: &[Cube],
    light: &Light,
) -> Color {
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        // return default sky box color
        return Color::new(4, 12, 36);
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal);

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;

    diffuse + specular
}

//pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere], camera: &Camera, light: &Light) {        para la esfera
pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI/3.0;
    let perspective_scale = (fov * 0.5).tan();

    // random number generator
    // let mut rng = rand::thread_rng();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            // if rng.gen_range(0.0..1.0) < 0.3 {
            //     // we skip 30% of the points
            //     continue;
            // }

            // Map the pixel coordinate to screen space [-1, 1]
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            // Adjust for aspect ratio and perspective 
            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            // Calculate the direction of the ray for this pixel
            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));

            // Apply camera rotation to the ray direction
            let rotated_direction = camera.basis_change(&ray_direction);

            // Cast the ray and get the pixel color
            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light);

            // Draw the pixel on screen with the returned color
            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Rust Graphics - Raytracer Example",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // move the window around
    window.set_position(500, 500);
    window.update();

    let rubber = Material::new(
        Color::new(80, 0, 0),   // Albedo: rojo oscuro
        1.0,                    // n: brillo amplio y débil
        [0.9, 0.1],             // [kd, ks]
    );

    let ivory = Material::new(
        Color::new(100, 100, 80),   // Albedo: marfil
        50.0,                       // n: brillo más concentrado
        [0.6, 0.3],                 // [kd, ks]
    );

    let matte_blue = Material::new(
        Color::new(20, 60, 180),
        5.0,
        [0.9, 0.05],
    );

    let shiny_green = Material::new(
        Color::new(20, 160, 70),
        100.0,
        [0.5, 0.8],
    );

    let plastic_yellow = Material::new(
        Color::new(220, 180, 20),
        32.0,
        [0.7, 0.4],
    );

    let min = Material::new(
        Color::new(80, 0, 0),
        1.0,
        [0.9, 0.1],
    );

    let max = Material::new(
        Color::new(80, 0, 0),
        100.0,
        [0.9, 0.1],
    );

    let red_material = Material::new(
        Color::new(180,20,20),
        10.0,
        [0.8, 0.3],
    );

    let ivory_material = Material::new(
        Color::new(180,180,140),
        10.0,
        [0.6, 0.6],
    );

    let objects = [
        //Sphere { center: Vec3::new(0.0, 0.0, 0.0), radius: 1.0, material: min },
        //Sphere { center: Vec3::new(-2.0, 2.0, -1.0), radius: 0.5, material: max },
        //Cube { center: Vec3::new(-1.2, 0.0, 0.0), radius: 1.0, material: red_material }, <- dió error, idk pq

        Cube::new(
            Vec3::new(-1.2, 0.0, 0.0),
            1.5,
            red_material,
        ),
        Cube::new(
            Vec3::new(1.2, 0.0, -1.0),
            1.5,
            ivory_material,
        ),

    ];

    // Initialize camera
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),  // eye: Initial camera position
        Vec3::new(0.0, 0.0, 0.0),  // center: Point the camera is looking at (origin)
        Vec3::new(0.0, 1.0, 0.0)   // up: World up vector
    );
    let rotation_speed = PI/50.0;

    let light = Light::new(
        Vec3::new(0.0, 0.0, 5.0),
        Color::new(255, 255, 255),
        1.0
    );

    while window.is_open() {
        // listen to inputs
        if window.is_key_down(Key::Escape) {
            break;
        }

        //  camera orbit controls
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, rotation_speed);
        }

        // draw some points
        render(&mut framebuffer, &objects, &camera, &light);


        // update the window with the framebuffer contents
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
