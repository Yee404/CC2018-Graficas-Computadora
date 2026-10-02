// skybox.rs
// Skybox procedural configurable por ubicación: gradiente vertical, patrón
// moteado proyectado sobre las seis caras de un cubo y estrellas opcionales.
// No usa texturas ni dependencias externas y es determinista.

use nalgebra_glm::Vec3;

use crate::color::Color;

pub struct Skybox {
    pub zenith: Color,
    pub horizon: Color,
    pub nadir: Color,
    pub pattern_cells: f32,    // celdas por cara del cubo
    pub pattern_strength: f32, // 0 = sin moteado
    pub star_threshold: f32,   // >= 1.0 desactiva las estrellas
    pub star_color: Color,
}

impl Skybox {
    /// Cielo del End: oscuro, morado grisáceo y moteado.
    pub fn end() -> Self {
        Skybox {
            zenith: Color::new(36, 24, 46),
            horizon: Color::new(24, 17, 32),
            nadir: Color::new(8, 6, 12),
            pattern_cells: 24.0,
            pattern_strength: 0.45,
            star_threshold: 0.996,
            star_color: Color::new(200, 180, 230),
        }
    }

    /// Fondo de la Stronghold (solo visible si un rayo escapa de la sala).
    pub fn stronghold() -> Self {
        Skybox {
            zenith: Color::new(20, 20, 24),
            horizon: Color::new(12, 12, 14),
            nadir: Color::new(5, 5, 6),
            pattern_cells: 8.0,
            pattern_strength: 0.2,
            star_threshold: 2.0,
            star_color: Color::black(),
        }
    }

    /// Color del cielo para un rayo que no golpeó geometría.
    pub fn sample(&self, direction: &Vec3) -> Color {
        let d = direction.normalize();

        let mut color = if d.y >= 0.0 {
            self.horizon.lerp(self.zenith, d.y)
        } else {
            self.horizon.lerp(self.nadir, -d.y)
        };

        if self.pattern_strength > 0.0 {
            let n = Self::pattern_value(&d, self.pattern_cells);
            color = color * (1.0 + self.pattern_strength * (n * 2.0 - 1.0));
        }

        if self.star_threshold < 1.0 {
            let star = Self::star_intensity(&d, self.star_threshold);
            if star > 0.0 {
                color = color + self.star_color * star;
            }
        }

        color
    }

    /// Proyecta la dirección sobre la cara dominante de un cubo y devuelve
    /// un valor de ruido por celda en [0, 1].
    fn pattern_value(d: &Vec3, cells: f32) -> f32 {
        let (ax, ay, az) = (d.x.abs(), d.y.abs(), d.z.abs());
        let (face, u, v) = if ax >= ay && ax >= az {
            (if d.x > 0.0 { 0 } else { 1 }, d.z / ax, d.y / ax)
        } else if ay >= az {
            (if d.y > 0.0 { 2 } else { 3 }, d.x / ay, d.z / ay)
        } else {
            (if d.z > 0.0 { 4 } else { 5 }, d.x / az, d.y / az)
        };

        let cu = ((u * 0.5 + 0.5) * cells).floor() as i32;
        let cv = ((v * 0.5 + 0.5) * cells).floor() as i32;
        Self::hash3(cu, cv, face)
    }

    fn star_intensity(d: &Vec3, threshold: f32) -> f32 {
        let scale = 180.0;
        let hx = (d.x * scale).floor() as i32;
        let hy = (d.y * scale).floor() as i32;
        let hz = (d.z * scale).floor() as i32;

        let h = Self::hash3(hx, hy, hz.wrapping_add(97));
        if h > threshold {
            ((h - threshold) / (1.0 - threshold)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Hash entero determinista usado como pseudo-ruido.
    fn hash3(x: i32, y: i32, z: i32) -> f32 {
        let mut n = x
            .wrapping_mul(374_761_393)
            .wrapping_add(y.wrapping_mul(668_265_263))
            .wrapping_add(z.wrapping_mul(2_147_483_647_i32.wrapping_div(3)));
        n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
        n ^= n >> 16;
        (n as u32) as f32 / u32::MAX as f32
    }
}
