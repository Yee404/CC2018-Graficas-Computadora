use std::rc::Rc;

use crate::color::Color;
use crate::texture::Texture;

/// Identificador de material. Los cubos y las intersecciones guardan este id
/// (Copy) en lugar de una copia del material, así nunca se clonan materiales
/// ni texturas por intersección.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialId {
    StrongholdBricks,
    MossyBricks,
    EndPortalFrame,
    IronBars,
    EndStone,
    Obsidian,
    EyeOfEnder,
    EndPortal,
    Bedrock,
    Dragon,
    DragonEye,
    ExitPortal,
    Grass,
    Dirt,
}

impl MaterialId {
    // Debe seguir exactamente el orden de declaración del enum.
    pub const ALL: [MaterialId; 14] = [
        MaterialId::StrongholdBricks,
        MaterialId::MossyBricks,
        MaterialId::EndPortalFrame,
        MaterialId::IronBars,
        MaterialId::EndStone,
        MaterialId::Obsidian,
        MaterialId::EyeOfEnder,
        MaterialId::EndPortal,
        MaterialId::Bedrock,
        MaterialId::Dragon,
        MaterialId::DragonEye,
        MaterialId::ExitPortal,
        MaterialId::Grass,
        MaterialId::Dirt,
    ];
}

#[derive(Debug)]
pub struct Material {
    pub texture: Option<Rc<Texture>>,
    pub albedo: Color, // tinte que multiplica la textura
    pub diffuse_weight: f32,
    pub specular_weight: f32,
    pub specular_exponent: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub index_of_refraction: f32,
    pub emission_color: Color,
    pub emission_strength: f32,
}

impl Material {
    /// Valores por defecto (opaco, mate, sin emisión) con una textura.
    fn base(texture_path: &str) -> Self {
        Material {
            texture: Some(Texture::from_ppm(texture_path)),
            ..Material::untextured(Color::new(255, 255, 255))
        }
    }

    fn untextured(albedo: Color) -> Self {
        Material {
            texture: None,
            albedo,
            diffuse_weight: 0.9,
            specular_weight: 0.1,
            specular_exponent: 10.0,
            transparency: 0.0,
            reflectivity: 0.0,
            index_of_refraction: 1.0,
            emission_color: Color::black(),
            emission_strength: 0.0,
        }
    }

    /// Color base en el punto golpeado: textura (por UV) multiplicada por el albedo.
    pub fn base_color(&self, u: f32, v: f32) -> Color {
        match &self.texture {
            Some(texture) => texture.sample(u, v).modulate(self.albedo),
            None => self.albedo,
        }
    }
}

pub struct MaterialLibrary {
    materials: Vec<Material>,
}

impl MaterialLibrary {
    pub fn load() -> Self {
        for (index, id) in MaterialId::ALL.iter().enumerate() {
            debug_assert_eq!(*id as usize, index, "MaterialId::ALL fuera de orden");
        }

        let materials = MaterialId::ALL
            .iter()
            .map(|&id| build_material(id))
            .collect();
        MaterialLibrary { materials }
    }

    pub fn get(&self, id: MaterialId) -> &Material {
        &self.materials[id as usize]
    }
}

fn build_material(id: MaterialId) -> Material {
    match id {
        // ---- Cinco materiales puntuables ----
        MaterialId::StrongholdBricks => Material {
            albedo: Color::new(205, 205, 210),
            diffuse_weight: 0.90,
            specular_weight: 0.12,
            specular_exponent: 12.0,
            transparency: 0.0,
            reflectivity: 0.03,
            index_of_refraction: 1.0,
            ..Material::base("assets/stronghold_bricks.ppm")
        },
        MaterialId::EndPortalFrame => Material {
            albedo: Color::new(220, 235, 215),
            diffuse_weight: 0.85,
            specular_weight: 0.35,
            specular_exponent: 40.0,
            transparency: 0.0,
            reflectivity: 0.08,
            index_of_refraction: 1.0,
            ..Material::base("assets/end_portal_frame.ppm")
        },
        MaterialId::IronBars => Material {
            albedo: Color::new(210, 215, 225),
            diffuse_weight: 0.45,
            specular_weight: 0.80,
            specular_exponent: 150.0,
            transparency: 0.0,
            reflectivity: 0.35,
            index_of_refraction: 1.0,
            ..Material::base("assets/iron_bars.ppm")
        },
        MaterialId::EndStone => Material {
            albedo: Color::new(240, 240, 215),
            diffuse_weight: 0.95,
            specular_weight: 0.08,
            specular_exponent: 6.0,
            transparency: 0.0,
            reflectivity: 0.02,
            index_of_refraction: 1.0,
            ..Material::base("assets/end_stone.ppm")
        },
        MaterialId::Obsidian => Material {
            albedo: Color::new(210, 190, 240),
            diffuse_weight: 0.55,
            specular_weight: 0.90,
            specular_exponent: 180.0,
            transparency: 0.0,
            reflectivity: 0.30,
            index_of_refraction: 1.0,
            ..Material::base("assets/obsidian.ppm")
        },

        // ---- Materiales adicionales ----
        MaterialId::MossyBricks => Material {
            diffuse_weight: 0.92,
            specular_weight: 0.06,
            specular_exponent: 8.0,
            reflectivity: 0.01,
            ..Material::base("assets/mossy_bricks.ppm")
        },
        MaterialId::EyeOfEnder => Material {
            diffuse_weight: 0.6,
            specular_weight: 0.7,
            specular_exponent: 90.0,
            reflectivity: 0.1,
            emission_color: Color::new(40, 200, 150),
            emission_strength: 0.35,
            ..Material::base("assets/eye_of_ender.ppm")
        },
        // Fase 1: refracción y emisión básicas; el efecto dimensional es Fase 2.
        MaterialId::EndPortal => Material {
            diffuse_weight: 0.3,
            specular_weight: 0.9,
            specular_exponent: 200.0,
            transparency: 0.35,
            reflectivity: 0.10,
            index_of_refraction: 1.33,
            emission_color: Color::new(20, 90, 95),
            emission_strength: 0.6,
            ..Material::base("assets/end_portal.ppm")
        },
        MaterialId::Bedrock => Material {
            diffuse_weight: 0.9,
            specular_weight: 0.05,
            specular_exponent: 6.0,
            ..Material::base("assets/bedrock.ppm")
        },
        MaterialId::Dragon => Material {
            diffuse_weight: 0.8,
            specular_weight: 0.4,
            specular_exponent: 40.0,
            ..Material::base("assets/dragon.ppm")
        },
        MaterialId::DragonEye => Material {
            emission_color: Color::new(230, 60, 255),
            emission_strength: 1.0,
            ..Material::untextured(Color::new(200, 80, 230))
        },
        MaterialId::ExitPortal => Material {
            diffuse_weight: 0.3,
            specular_weight: 0.9,
            specular_exponent: 200.0,
            transparency: 0.35,
            reflectivity: 0.10,
            index_of_refraction: 1.33,
            emission_color: Color::new(20, 90, 95),
            emission_strength: 0.6,
            ..Material::base("assets/end_portal.ppm")
        },
        // Atlas vertical tapa/lado/fondo (usar con `Cube::top_side_bottom`).
        MaterialId::Grass => Material {
            diffuse_weight: 0.92,
            specular_weight: 0.05,
            specular_exponent: 6.0,
            ..Material::base("assets/grass.ppm")
        },
        MaterialId::Dirt => Material {
            diffuse_weight: 0.92,
            specular_weight: 0.04,
            specular_exponent: 5.0,
            ..Material::base("assets/dirt.ppm")
        },
    }
}
