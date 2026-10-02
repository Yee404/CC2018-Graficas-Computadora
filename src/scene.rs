// scene.rs
// Construcción de las dos ubicaciones. Cada una tiene sus propios objetos,
// luces, skybox, pose inicial, límites de cámara y velocidad de movimiento.
// Superficies grandes son cuboides con textura repetida por bloque, no
// bloques sueltos.

use nalgebra_glm::Vec3;

use crate::color::Color;
use crate::config::*;
use crate::cube::Cube;
use crate::light::Light;
use crate::material::{MaterialId, MaterialLibrary};
use crate::skybox::Skybox;
use crate::state::Location;

pub struct Scene {
    pub objects: Vec<Cube>,     // geometría estática
    pub dragon: Vec<Cube>,      // partes del dragón (ocultables en Fase 4)
    pub exit_portal: Vec<Cube>, // superficie del portal de salida (inactiva al inicio)
    pub lights: Vec<Light>,
    pub skybox: Skybox,
    pub ambient: f32,
    pub spawn: CameraPose,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub move_speed: f32,
}

pub struct World {
    pub materials: MaterialLibrary,
    pub stronghold: Scene,
    pub end: Scene,
}

impl World {
    pub fn build() -> Self {
        World {
            materials: MaterialLibrary::load(),
            stronghold: build_stronghold(),
            end: build_end(),
        }
    }

    pub fn scene(&self, location: Location) -> &Scene {
        match location {
            Location::Stronghold => &self.stronghold,
            Location::End => &self.end,
        }
    }
}

fn bx(min: [f32; 3], max: [f32; 3], material: MaterialId) -> Cube {
    Cube::from_bounds(vec3(min), vec3(max), material)
}

// ---------------------------------------------------------------------------
// Stronghold flotante: ruina abierta de la sala del End Portal sobre una isla
// ---------------------------------------------------------------------------

fn build_stronghold() -> Scene {
    let bricks = MaterialId::StrongholdBricks;
    let mossy = MaterialId::MossyBricks;
    let grass = MaterialId::Grass;

    let mut objects = Vec::new();

    // --- Isla flotante: 4 capas de 1 bloque que se reducen hacia abajo ---
    for &(min, max) in SH_GRASS_LAYER.iter() {
        objects.push(bx(min, max, grass).top_side_bottom());
    }
    for &(min, max) in SH_DIRT_LAYER.iter() {
        objects.push(bx(min, max, MaterialId::Dirt));
    }
    for &(min, max) in SH_ROOT_LAYERS.iter() {
        objects.push(bx(min, max, mossy));
    }

    // --- Piso de la ruina: ladrillo normal con un sector cubierto de musgo ---
    let f0 = SH_FLOOR_MIN;
    let f1 = SH_FLOOR_MAX;
    let ft = SH_FLOOR_TOP;
    let split_x = -1.0;
    let split_z = 1.0;
    objects.push(bx([f0, 0.0, f0], [f1, ft, split_z], bricks));
    objects.push(bx([f0, 0.0, split_z], [split_x, ft, f1], mossy));
    objects.push(bx([split_x, 0.0, split_z], [f1, ft, f1], bricks));

    // --- Tarima del portal y escalera sur (media altura por escalón) ---
    let d0 = SH_DAIS_MIN;
    let d1 = SH_DAIS_MAX;
    let dt = SH_DAIS_TOP;
    let s0 = SH_STAIR_MIN_X;
    let s1 = SH_STAIR_MAX_X;
    objects.push(bx([d0, ft, d0], [d1, dt, d1], bricks));
    objects.push(bx([s0, ft, d1], [s1, ft + 0.5, d1 + 1.0], bricks));
    objects.push(bx([s0, 0.0, f1], [s1, 0.5, f1 + 1.0], bricks));

    // --- Columnas: las del norte completas, las del sur derruidas ---
    let wall_top = ft + SH_WALL_HEIGHT;
    let column_top = ft + SH_COLUMN_HEIGHT;
    objects.push(bx([f0, ft, f0], [f0 + 1.0, column_top, f0 + 1.0], bricks));
    objects.push(bx([f1 - 1.0, ft, f0], [f1, column_top, f0 + 1.0], bricks));
    objects.push(bx([f0, ft, f1 - 1.0], [f0 + 1.0, ft + 1.0, f1], mossy));
    objects.push(bx([f1 - 1.0, ft, f1 - 1.0], [f1, ft + 2.0, f1], bricks));

    // --- Pared norte baja con dos secciones enrejadas ---
    let nz0 = f0;
    let nz1 = f0 + 1.0;
    let (gap_a, gap_b) = (SH_BAR_GAPS[0], SH_BAR_GAPS[1]);
    objects.push(bx([f0 + 1.0, ft, nz0], [gap_a.0, wall_top, nz1], bricks));
    objects.push(bx([gap_a.1, ft, nz0], [gap_b.0, wall_top, nz1], mossy));
    objects.push(bx([gap_b.1, ft, nz0], [f1 - 1.0, wall_top, nz1], bricks));
    for &(x0, x1) in SH_BAR_GAPS.iter() {
        push_iron_bars(&mut objects, x0, x1, ft, wall_top, (nz0 + nz1) * 0.5);
    }

    // --- Paredes oeste y este derruidas (el frente sur queda abierto) ---
    objects.push(bx([f0, ft, f0 + 1.0], [f0 + 1.0, ft + 1.0, 2.0], mossy));
    objects.push(bx(
        [f0, ft + 1.0, f0 + 1.0],
        [f0 + 1.0, wall_top, -1.0],
        bricks,
    ));
    objects.push(bx([f1 - 1.0, ft, f0 + 1.0], [f1, wall_top, 1.0], bricks));
    objects.push(bx([f1 - 1.0, ft, 1.0], [f1, ft + 1.0, 3.0], mossy));

    // --- Césped creciendo sobre la ruina ---
    objects.push(bx([f0 + 1.0, ft, f1 - 2.0], [f0 + 3.0, ft + 1.0, f1], grass).top_side_bottom());
    objects.push(bx([f0, ft + 1.0, -1.0], [f0 + 1.0, ft + 2.0, 2.0], grass).top_side_bottom());

    // --- End Portal: 12 marcos (3 por lado, esquinas vacías) y 12 ojos ---
    let p0 = PORTAL_INNER_MIN;
    let p1 = PORTAL_INNER_MAX;
    let y0 = dt;
    let y1 = y0 + FRAME_HEIGHT;
    for i in 0..3 {
        let a = p0 + i as f32;
        let b = a + 1.0;
        push_frame_with_eye(&mut objects, [a, y0, p0 - 1.0], [b, y1, p0]);
        push_frame_with_eye(&mut objects, [a, y0, p1], [b, y1, p1 + 1.0]);
        push_frame_with_eye(&mut objects, [p0 - 1.0, y0, a], [p0, y1, b]);
        push_frame_with_eye(&mut objects, [p1, y0, a], [p1 + 1.0, y1, b]);
    }

    // Superficie activa: llena exactamente el interior 3 × 3, muy por encima
    // del fondo (la tarima) para evitar z-fighting.
    let surface_top = y0 + PORTAL_SURFACE_OFFSET;
    objects.push(bx(
        [p0, surface_top - PORTAL_SURFACE_THICKNESS, p0],
        [p1, surface_top, p1],
        MaterialId::EndPortal,
    ));

    let lights = vec![
        Light::new(vec3(SH_SUN_POSITION), SH_SUN_COLOR, SH_SUN_INTENSITY),
        Light::point(
            vec3(SH_PORTAL_LIGHT_POSITION),
            PORTAL_GLOW_COLOR,
            SH_PORTAL_LIGHT_INTENSITY,
            SH_PORTAL_LIGHT_RANGE,
        ),
    ];

    // Cielo diurno con nubes pixeladas sutiles, visible alrededor de la isla.
    let skybox = Skybox {
        zenith: Color::new(96, 146, 214),
        horizon: Color::new(178, 206, 232),
        nadir: Color::new(140, 170, 205),
        pattern_cells: 10.0,
        pattern_strength: 0.06,
        star_threshold: 2.0,
        star_color: Color::black(),
    };

    Scene {
        objects,
        dragon: Vec::new(),
        exit_portal: Vec::new(),
        lights,
        skybox,
        ambient: SH_AMBIENT,
        spawn: STRONGHOLD_SPAWN,
        bounds_min: vec3(SH_BOUNDS_MIN),
        bounds_max: vec3(SH_BOUNDS_MAX),
        move_speed: STRONGHOLD_MOVE_SPEED,
    }
}

/// Marco del portal (tapa con hueco del ojo, lados crema con franja verde)
/// y su ojo centrado, apoyado exactamente sobre la tapa.
fn push_frame_with_eye(objects: &mut Vec<Cube>, min: [f32; 3], max: [f32; 3]) {
    objects.push(bx(min, max, MaterialId::EndPortalFrame).top_side_bottom());

    let cx = (min[0] + max[0]) * 0.5;
    let cz = (min[2] + max[2]) * 0.5;
    let half = EYE_SIZE * 0.5;
    objects.push(
        bx(
            [cx - half, max[1], cz - half],
            [cx + half, max[1] + EYE_HEIGHT, cz + half],
            MaterialId::EyeOfEnder,
        )
        .stretched(),
    );
}

/// Barrotes verticales y un travesaño que llenan un tramo de pared.
fn push_iron_bars(objects: &mut Vec<Cube>, x0: f32, x1: f32, y0: f32, y1: f32, z: f32) {
    let t = SH_BAR_THICKNESS * 0.5;
    for i in 1..=SH_BARS_PER_GAP {
        let x = x0 + (x1 - x0) * i as f32 / (SH_BARS_PER_GAP as f32 + 1.0);
        objects.push(bx(
            [x - t, y0, z - t],
            [x + t, y1, z + t],
            MaterialId::IronBars,
        ));
    }
    let y = (y0 + y1) * 0.5;
    objects.push(bx(
        [x0, y - t, z - t],
        [x1, y + t, z + t],
        MaterialId::IronBars,
    ));
}

// ---------------------------------------------------------------------------
// The End
// ---------------------------------------------------------------------------

fn build_end() -> Scene {
    let mut objects = Vec::new();

    // Isla de End Stone: 4 bloques de altura total, textura por bloque
    for &(min, max) in END_LAYERS.iter() {
        objects.push(bx(min, max, MaterialId::EndStone).with_tile_size(END_STONE_TILE_SIZE));
    }

    // Pilares de obsidiana con remate de bedrock
    for &(x, z, hw, height) in END_PILLARS.iter() {
        objects.push(bx(
            [x - hw, 0.0, z - hw],
            [x + hw, height, z + hw],
            MaterialId::Obsidian,
        ));
        objects.push(bx(
            [x - 0.5, height, z - 0.5],
            [x + 0.5, height + 1.0, z + 0.5],
            MaterialId::Bedrock,
        ));
    }

    // Fuente de bedrock (portal de salida) centrada en la isla
    let e0 = EXIT_INNER_MIN;
    let e1 = EXIT_INNER_MAX;
    let c = (e0 + e1) * 0.5;
    let rim = EXIT_RIM_HEIGHT;
    let bedrock = MaterialId::Bedrock;
    objects.push(bx([e0 - 1.0, 0.0, e0 - 1.0], [e1 + 1.0, rim, e0], bedrock));
    objects.push(bx([e0 - 1.0, 0.0, e1], [e1 + 1.0, rim, e1 + 1.0], bedrock));
    objects.push(bx([e0 - 1.0, 0.0, e0], [e0, rim, e1], bedrock));
    objects.push(bx([e1, 0.0, e0], [e1 + 1.0, rim, e1], bedrock));
    objects.push(bx([e0, 0.0, e0], [e1, EXIT_BASIN_FLOOR, e1], bedrock));
    objects.push(bx(
        [c - 0.5, 0.0, c - 0.5],
        [c + 0.5, EXIT_PILLAR_HEIGHT, c + 0.5],
        bedrock,
    ));

    let exit_portal = vec![bx(
        [e0, EXIT_PORTAL_SURFACE_Y - 0.05, e0],
        [e1, EXIT_PORTAL_SURFACE_Y, e1],
        MaterialId::ExitPortal,
    )];

    let lights = vec![Light::new(
        vec3(END_LIGHT_POSITION),
        END_LIGHT_COLOR,
        END_LIGHT_INTENSITY,
    )];

    Scene {
        objects,
        dragon: build_dragon(vec3(DRAGON_ORIGIN)),
        exit_portal,
        lights,
        skybox: Skybox::end(),
        ambient: END_AMBIENT,
        spawn: END_ARRIVAL,
        bounds_min: vec3(END_BOUNDS_MIN),
        bounds_max: vec3(END_BOUNDS_MAX),
        move_speed: END_MOVE_SPEED,
    }
}

/// Dragón estático orientado hacia +X, armado con cuboides por partes.
/// Cada parte es un elemento separado para poder animarla u ocultarla
/// individualmente en la Fase 4.
fn build_dragon(origin: Vec3) -> Vec<Cube> {
    let part = |center: [f32; 3], size: [f32; 3], material: MaterialId| {
        Cube::cuboid(origin + vec3(center), vec3(size), material)
    };
    let body = MaterialId::Dragon;
    let eye = MaterialId::DragonEye;

    vec![
        // Cuerpo, cuello y cabeza
        part([0.0, 0.0, 0.0], [8.0, 3.0, 4.0], body),
        part([5.5, 0.8, 0.0], [3.0, 1.6, 1.6], body),
        part([8.3, 1.2, 0.0], [2.6, 2.0, 2.2], body),
        part([10.3, 0.9, 0.0], [1.6, 1.0, 1.6], body),
        // Ojos emisivos
        part([9.0, 1.8, 1.0], [0.5, 0.3, 0.3], eye).stretched(),
        part([9.0, 1.8, -1.0], [0.5, 0.3, 0.3], eye).stretched(),
        // Cola en tres segmentos
        part([-6.0, 0.3, 0.0], [4.0, 1.6, 1.6], body),
        part([-9.5, 0.6, 0.0], [3.0, 1.1, 1.1], body),
        part([-12.0, 0.9, 0.0], [2.0, 0.7, 0.7], body),
        // Alas
        part([0.5, 1.3, 6.5], [5.0, 0.3, 9.0], body),
        part([0.5, 1.3, -6.5], [5.0, 0.3, 9.0], body),
        // Patas
        part([2.5, -2.3, 1.3], [1.0, 2.0, 1.0], body),
        part([2.5, -2.3, -1.3], [1.0, 2.0, 1.0], body),
        part([-2.5, -2.3, 1.3], [1.0, 2.0, 1.0], body),
        part([-2.5, -2.3, -1.3], [1.0, 2.0, 1.0], body),
    ]
}
