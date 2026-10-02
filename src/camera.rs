use nalgebra_glm::Vec3;
use std::f32::consts::PI;

use crate::config::{DEFAULT_FOV, MAX_FOV, MAX_PITCH, MIN_FOV};

pub struct Camera {
    pub eye: Vec3,    // Camera position in world space
    pub center: Vec3, // Point the camera is looking at
    pub up: Vec3,     // Up vector
    pub fov: f32,     // Campo de visión vertical (radianes); controla el zoom
}

impl Camera {
    pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        Camera {
            eye,
            center,
            up,
            fov: DEFAULT_FOV,
        }
    }

    pub fn basis_change(&self, vector: &Vec3) -> Vec3 {
        let forward = (self.center - self.eye).normalize();
        let right = forward.cross(&self.up).normalize();
        let up = right.cross(&forward).normalize();

        let rotated = vector.x * right + vector.y * up - vector.z * forward;

        rotated.normalize()
    }

    #[allow(dead_code)]
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        // Calculate the vector from the center to the eye (radius vector) and measure the distance
        let radius_vector = self.eye - self.center;
        let radius = radius_vector.magnitude();

        // Calculate current yaw (rotation around Y-axis)
        // atan2(z, x) gives us the angle in the XZ plane
        // Range: [-π, π], where 0 is along positive X-axis, π/2 is along positive Z-axis
        let current_yaw = radius_vector.z.atan2(radius_vector.x);

        // Calculate current pitch (rotation around X-axis)
        // xz here refers to the proyection of the radius over the x axis
        let radius_xz =
            (radius_vector.x * radius_vector.x + radius_vector.z * radius_vector.z).sqrt();
        // We use -y because positive pitch is when we look up (negative y in our coordinate system)
        // Range: [-π/2, π/2], where 0 is horizontal, π/2 is looking straight up
        let current_pitch = (-radius_vector.y).atan2(radius_xz);

        // Apply delta rotations
        // Keep yaw in range [0, 2π] for consistency
        let new_yaw = (current_yaw + delta_yaw) % (2.0 * PI);
        // Clamp pitch to slightly less than [-π/2, π/2] to prevent gimbal lock
        let new_pitch = (current_pitch + delta_pitch).clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);

        // Calculate new eye position
        // We use spherical coordinates to cartesian conversion:
        // x = r * cos(yaw) * cos(pitch)
        // y = -r * sin(pitch)  // Negative because positive y is up
        // z = r * sin(yaw) * cos(pitch)

        let new_eye = self.center
            + Vec3::new(
                radius * new_yaw.cos() * new_pitch.cos(),
                -radius * new_pitch.sin(),
                radius * new_yaw.sin() * new_pitch.cos(),
            );

        self.eye = new_eye;
    }

    /// Coloca la cámara en una pose exacta (usado al cambiar de ubicación,
    /// al restablecer y, en la Fase 2, durante los teletransportes).
    pub fn set_pose(&mut self, eye: Vec3, center: Vec3) {
        self.eye = eye;
        self.center = center;
        self.fov = DEFAULT_FOV;
    }

    /// Traslada `eye` y `center` juntos, conservando la dirección de vista.
    pub fn translate(&mut self, delta: Vec3) {
        self.eye += delta;
        self.center += delta;
    }

    /// Rotación en primera persona: gira el punto observado alrededor del ojo.
    /// `delta_yaw` positivo gira a la derecha; `delta_pitch` positivo mira hacia arriba.
    pub fn look(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let offset = self.center - self.eye;
        let distance = offset.magnitude().max(1e-3);
        let direction = offset / distance;

        let yaw = direction.z.atan2(direction.x) + delta_yaw;
        let pitch =
            (direction.y.clamp(-1.0, 1.0).asin() + delta_pitch).clamp(-MAX_PITCH, MAX_PITCH);

        let new_direction = Vec3::new(
            pitch.cos() * yaw.cos(),
            pitch.sin(),
            pitch.cos() * yaw.sin(),
        );
        self.center = self.eye + new_direction * distance;
    }

    /// Movimiento relativo a la vista: avance sobre el plano horizontal,
    /// desplazamiento lateral y vertical en unidades de mundo.
    pub fn move_relative(&mut self, forward_amount: f32, right_amount: f32, up_amount: f32) {
        let mut forward = self.center - self.eye;
        forward.y = 0.0;
        let forward = if forward.magnitude() < 1e-6 {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            forward.normalize()
        };
        let right = forward.cross(&self.up).normalize();

        let delta = forward * forward_amount + right * right_amount + self.up * up_amount;
        self.translate(delta);
    }

    /// Acercar (delta negativo) o alejar (delta positivo) cambiando el FOV.
    pub fn zoom(&mut self, delta_fov: f32) {
        self.fov = (self.fov + delta_fov).clamp(MIN_FOV, MAX_FOV);
    }

    /// Mantiene el ojo dentro de los límites de la ubicación activa.
    pub fn clamp_to_bounds(&mut self, min: &Vec3, max: &Vec3) {
        let clamped = Vec3::new(
            self.eye.x.clamp(min.x, max.x),
            self.eye.y.clamp(min.y, max.y),
            self.eye.z.clamp(min.z, max.z),
        );
        let delta = clamped - self.eye;
        self.translate(delta);
    }
}
