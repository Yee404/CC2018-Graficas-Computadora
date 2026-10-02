use crate::color::Color;
use nalgebra_glm::Vec3;

pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: Option<f32>, // None = sin atenuación (luz lejana)
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
            range: None,
        }
    }

    /// Luz local cuya intensidad cae suavemente hasta 0 en `range`.
    pub fn point(position: Vec3, color: Color, intensity: f32, range: f32) -> Self {
        Light {
            position,
            color,
            intensity,
            range: Some(range),
        }
    }

    pub fn attenuation(&self, distance: f32) -> f32 {
        match self.range {
            None => 1.0,
            Some(range) => {
                let x = (distance / range).min(1.0);
                let falloff = 1.0 - x * x;
                falloff * falloff
            }
        }
    }
}
