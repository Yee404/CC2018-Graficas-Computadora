use minifb::{Key, Window, WindowOptions};
use std::time::Instant;

mod camera;
mod color;
mod config;
mod cube;
mod framebuffer;
mod light;
mod material;
mod ray_intersect;
mod render;
mod scene;
mod skybox;
mod sphere;
mod state;
mod texture;

use camera::Camera;
use config::{
    FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, LOOK_SPEED, MAX_DELTA_TIME, WINDOW_TITLE, ZOOM_SPEED,
};
use framebuffer::Framebuffer;
use nalgebra_glm::Vec3;
use render::{render, RenderScene};
use scene::World;
use state::{GameState, Location};

fn handle_camera_input(window: &Window, camera: &mut Camera, move_speed: f32, dt: f32) {
    let look = LOOK_SPEED * dt;
    if window.is_key_down(Key::Left) {
        camera.look(-look, 0.0);
    }
    if window.is_key_down(Key::Right) {
        camera.look(look, 0.0);
    }
    if window.is_key_down(Key::Up) {
        camera.look(0.0, look);
    }
    if window.is_key_down(Key::Down) {
        camera.look(0.0, -look);
    }

    let mut forward = 0.0;
    let mut right = 0.0;
    let mut up = 0.0;
    if window.is_key_down(Key::W) {
        forward += 1.0;
    }
    if window.is_key_down(Key::S) {
        forward -= 1.0;
    }
    if window.is_key_down(Key::D) {
        right += 1.0;
    }
    if window.is_key_down(Key::A) {
        right -= 1.0;
    }
    if window.is_key_down(Key::Space) {
        up += 1.0;
    }
    if window.is_key_down(Key::LeftShift) {
        up -= 1.0;
    }
    let step = move_speed * dt;
    camera.move_relative(forward * step, right * step, up * step);

    if window.is_key_down(Key::Q) {
        camera.zoom(-ZOOM_SPEED * dt);
    }
    if window.is_key_down(Key::E) {
        camera.zoom(ZOOM_SPEED * dt);
    }
}

fn switch_location(state: &mut GameState, camera: &mut Camera, world: &World, target: Location) {
    if state.location != target {
        state.debug_switch_location(target);
        let spawn = world.scene(target).spawn;
        camera.set_pose(spawn.eye(), spawn.center());
    }
}

fn main() {
    let mut framebuffer = Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
    let mut window = Window::new(
        WINDOW_TITLE,
        FRAMEBUFFER_WIDTH,
        FRAMEBUFFER_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(200, 100);
    window.update();

    let world = World::build();
    let mut state = GameState::new();

    let spawn = world.scene(state.location).spawn;
    let mut camera = Camera::new(spawn.eye(), spawn.center(), Vec3::new(0.0, 1.0, 0.0));

    let mut last_frame = Instant::now();

    while window.is_open() {
        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(MAX_DELTA_TIME);
        last_frame = now;

        state.update(dt);

        if window.is_key_down(Key::Escape) {
            break;
        }

        // Fase 1: cambio temporal de ubicación (lo reemplazarán los portales).
        if window.is_key_down(Key::Key1) {
            switch_location(&mut state, &mut camera, &world, Location::Stronghold);
        }
        if window.is_key_down(Key::Key2) {
            switch_location(&mut state, &mut camera, &world, Location::End);
        }

        let scene = world.scene(state.location);

        if !state.controls_locked {
            handle_camera_input(&window, &mut camera, scene.move_speed, dt);

            if window.is_key_down(Key::R) {
                camera.set_pose(scene.spawn.eye(), scene.spawn.center());
            }
        }
        camera.clamp_to_bounds(&scene.bounds_min, &scene.bounds_max);

        let render_scene = RenderScene::new(scene, &world.materials, &state);
        render(&mut framebuffer, &camera, &render_scene);

        window
            .update_with_buffer(&framebuffer.buffer, FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT)
            .unwrap();
    }
}
