// config.rs
// Constantes centralizadas del proyecto. Algunas solo se usarán en fases
// posteriores (triggers, fundidos, muerte del dragón, créditos), por eso se
// permite código no usado en este módulo.
#![allow(dead_code)]

use nalgebra_glm::Vec3;
use std::f32::consts::PI;

use crate::color::Color;

pub fn vec3(v: [f32; 3]) -> Vec3 {
    Vec3::new(v[0], v[1], v[2])
}

// ---------------------------------------------------------------------------
// Render
// ---------------------------------------------------------------------------
pub const WINDOW_TITLE: &str = "Stronghold & The End - Raytracer";
pub const FRAMEBUFFER_WIDTH: usize = 400;
pub const FRAMEBUFFER_HEIGHT: usize = 300;
pub const MAX_RECURSION_DEPTH: u32 = 2;
pub const MIN_CONTRIBUTION: f32 = 0.02;
pub const RAY_EPSILON: f32 = 1e-3;
pub const MAX_DELTA_TIME: f32 = 0.1;

// ---------------------------------------------------------------------------
// Cámara
// ---------------------------------------------------------------------------
pub const DEFAULT_FOV: f32 = PI / 3.0; // 60°
pub const MIN_FOV: f32 = PI / 9.0; // 20°
pub const MAX_FOV: f32 = PI / 2.0; // 90°
pub const ZOOM_SPEED: f32 = PI / 4.0; // radianes de FOV por segundo
pub const LOOK_SPEED: f32 = PI / 2.0; // radianes por segundo
pub const MAX_PITCH: f32 = PI / 2.0 - 0.1;
pub const STRONGHOLD_MOVE_SPEED: f32 = 5.0; // unidades por segundo
pub const END_MOVE_SPEED: f32 = 10.0;

#[derive(Debug, Clone, Copy)]
pub struct CameraPose {
    pub eye: [f32; 3],
    pub center: [f32; 3],
}

impl CameraPose {
    pub fn eye(&self) -> Vec3 {
        vec3(self.eye)
    }

    pub fn center(&self) -> Vec3 {
        vec3(self.center)
    }
}

pub const STRONGHOLD_SPAWN: CameraPose = CameraPose {
    eye: [10.0, 8.0, 14.0],
    center: [0.5, 1.5, 0.5],
};
pub const END_ARRIVAL: CameraPose = CameraPose {
    eye: [0.5, 2.5, 15.5],
    center: [0.5, 5.0, 0.5],
};
pub const END_PANORAMA: CameraPose = CameraPose {
    eye: [45.0, 28.0, 45.0],
    center: [0.5, 0.0, 0.5],
};

// ---------------------------------------------------------------------------
// Colores solicitados
// ---------------------------------------------------------------------------
pub const CREDITS_GREEN: Color = Color::new(8, 181, 18); // #08B512
pub const CREDITS_AQUA: Color = Color::new(1, 178, 173); // #01B2AD
pub const DRAGON_BEAM_PURPLE: Color = Color::new(61, 35, 71); // #3D2347
pub const PORTAL_GLOW_COLOR: Color = Color::new(60, 225, 205);

// ---------------------------------------------------------------------------
// Stronghold flotante
// Todo está alineado a la cuadrícula de bloques (1 unidad = 1 bloque), de
// modo que la textura repetida coincide con los bordes de cada bloque.
// El portal queda centrado en (x, z) = (0.5, 0.5).
// ---------------------------------------------------------------------------
pub type BoxBounds = ([f32; 3], [f32; 3]); // (min, max)

// Isla: cuatro capas de 1 bloque; la superior es la más ancha y tiene
// salientes para un contorno irregular.
pub const SH_GRASS_LAYER: [BoxBounds; 5] = [
    ([-6.0, -1.0, -6.0], [7.0, 0.0, 7.0]),
    ([-3.0, -1.0, -7.0], [3.0, 0.0, -6.0]),
    ([-2.0, -1.0, 7.0], [5.0, 0.0, 8.0]),
    ([-7.0, -1.0, -3.0], [-6.0, 0.0, 2.0]),
    ([7.0, -1.0, -1.0], [8.0, 0.0, 4.0]),
];
pub const SH_DIRT_LAYER: [BoxBounds; 3] = [
    ([-5.0, -2.0, -5.0], [6.0, -1.0, 6.0]),
    ([-1.0, -2.0, 6.0], [4.0, -1.0, 7.0]),
    ([-6.0, -2.0, -2.0], [-5.0, -1.0, 1.0]),
];
pub const SH_ROOT_LAYERS: [BoxBounds; 2] = [
    ([-3.0, -3.0, -4.0], [4.0, -2.0, 5.0]),
    ([-2.0, -4.0, -1.0], [3.0, -3.0, 3.0]),
];

// Ruina de la sala del portal
pub const SH_FLOOR_MIN: f32 = -5.0;
pub const SH_FLOOR_MAX: f32 = 6.0;
pub const SH_FLOOR_TOP: f32 = 1.0;
pub const SH_DAIS_MIN: f32 = -3.0;
pub const SH_DAIS_MAX: f32 = 4.0;
pub const SH_DAIS_TOP: f32 = 2.0;
pub const SH_STAIR_MIN_X: f32 = -1.0;
pub const SH_STAIR_MAX_X: f32 = 2.0;
pub const SH_WALL_HEIGHT: f32 = 2.0;
pub const SH_COLUMN_HEIGHT: f32 = 3.0;
pub const SH_BAR_GAPS: [(f32, f32); 2] = [(-3.0, -1.0), (2.0, 4.0)]; // tramos x de la pared norte
pub const SH_BARS_PER_GAP: usize = 3;
pub const SH_BAR_THICKNESS: f32 = 0.12;

// End Portal: interior 3 × 3 en x, z ∈ [-1, 2]
pub const PORTAL_INNER_MIN: f32 = -1.0;
pub const PORTAL_INNER_MAX: f32 = 2.0;
pub const FRAME_HEIGHT: f32 = 0.8125; // 13/16 de bloque
pub const PORTAL_SURFACE_OFFSET: f32 = 0.75; // altura sobre la base del marco
pub const PORTAL_SURFACE_THICKNESS: f32 = 0.05;
pub const EYE_SIZE: f32 = 0.5;
pub const EYE_HEIGHT: f32 = 0.1875;

pub const SH_SUN_POSITION: [f32; 3] = [14.0, 24.0, 18.0];
pub const SH_SUN_COLOR: Color = Color::new(255, 244, 220);
pub const SH_SUN_INTENSITY: f32 = 1.0;
pub const SH_PORTAL_LIGHT_POSITION: [f32; 3] = [0.5, 3.6, 0.5];
pub const SH_PORTAL_LIGHT_INTENSITY: f32 = 0.9;
pub const SH_PORTAL_LIGHT_RANGE: f32 = 6.0;
pub const SH_AMBIENT: f32 = 0.28;

pub const SH_BOUNDS_MIN: [f32; 3] = [-30.0, -15.0, -30.0];
pub const SH_BOUNDS_MAX: [f32; 3] = [30.0, 25.0, 30.0];

// Volumen de trigger (Fase 2).
pub const SH_PORTAL_TRIGGER_MIN: [f32; 3] = [PORTAL_INNER_MIN, SH_DAIS_TOP, PORTAL_INNER_MIN];
pub const SH_PORTAL_TRIGGER_MAX: [f32; 3] = [PORTAL_INNER_MAX, SH_DAIS_TOP + 1.2, PORTAL_INNER_MAX];

// ---------------------------------------------------------------------------
// The End
// ---------------------------------------------------------------------------
// Isla de cuatro bloques de altura total: capa superior amplia (con
// salientes irregulares) y tres capas inferiores cada vez más pequeñas.
pub const END_LAYERS: [BoxBounds; 8] = [
    ([-17.0, -1.0, -17.0], [18.0, 0.0, 18.0]),
    ([-10.0, -1.0, -19.0], [8.0, 0.0, -17.0]),
    ([-6.0, -1.0, 18.0], [12.0, 0.0, 20.0]),
    ([18.0, -1.0, -9.0], [20.0, 0.0, 7.0]),
    ([-19.0, -1.0, -5.0], [-17.0, 0.0, 10.0]),
    ([-13.0, -2.0, -13.0], [14.0, -1.0, 14.0]),
    ([-9.0, -3.0, -9.0], [10.0, -2.0, 10.0]),
    ([-5.0, -4.0, -5.0], [6.0, -3.0, 6.0]),
];
pub const END_STONE_TILE_SIZE: f32 = 1.0;

// (x, z, semiancho, altura); centros en .5 para alinear con los bloques
pub const END_PILLARS: [(f32, f32, f32, f32); 5] = [
    (13.5, -2.5, 1.5, 14.0),
    (9.5, 10.5, 1.5, 19.0),
    (-11.5, 6.5, 1.5, 16.0),
    (-8.5, -10.5, 1.5, 12.0),
    (4.5, -12.5, 1.5, 18.0),
];

// Fuente de bedrock: interior 3 × 3 en x, z ∈ [-1, 2]
pub const EXIT_INNER_MIN: f32 = -1.0;
pub const EXIT_INNER_MAX: f32 = 2.0;
pub const EXIT_RIM_HEIGHT: f32 = 1.0;
pub const EXIT_BASIN_FLOOR: f32 = 0.25;
pub const EXIT_PILLAR_HEIGHT: f32 = 4.0;
pub const EXIT_PORTAL_SURFACE_Y: f32 = 0.6;

pub const DRAGON_ORIGIN: [f32; 3] = [-2.0, 15.0, -9.0];

pub const END_LIGHT_POSITION: [f32; 3] = [25.0, 45.0, 20.0];
pub const END_LIGHT_COLOR: Color = Color::new(225, 215, 240);
pub const END_LIGHT_INTENSITY: f32 = 1.05;
pub const END_AMBIENT: f32 = 0.22;

pub const END_BOUNDS_MIN: [f32; 3] = [-70.0, -25.0, -70.0];
pub const END_BOUNDS_MAX: [f32; 3] = [70.0, 60.0, 70.0];

// Volumen de trigger del portal de salida (Fase 2).
pub const EXIT_PORTAL_TRIGGER_MIN: [f32; 3] = [EXIT_INNER_MIN, 0.0, EXIT_INNER_MIN];
pub const EXIT_PORTAL_TRIGGER_MAX: [f32; 3] = [EXIT_INNER_MAX, 1.5, EXIT_INNER_MAX];

// ---------------------------------------------------------------------------
// Secuencia (fases futuras)
// ---------------------------------------------------------------------------
pub const FADE_DURATION: f32 = 0.8;
pub const TELEPORT_COOLDOWN: f32 = 1.5;
pub const DRAGON_DEATH_DURATION: f32 = 10.0;
pub const CREDITS_SCROLL_SPEED: f32 = 20.0; // píxeles por segundo
