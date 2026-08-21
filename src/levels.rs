// Definicion de los 5 niveles. Datos, no logica: agregar un nivel es agregar
// una entrada a LEVELS. La atmosfera es PROVISIONAL (solo colores baratos).
use raylib::prelude::Color;

use crate::hunter::HunterConfig;

/// Objetivo del nivel. La maquina de estados no lo consulta: solo pregunta
/// `Game::is_complete()`. Agregar un objetivo nuevo es agregar una variante y
/// su rama en `Game::check_objective()`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LevelObjective {
    /// Recolectar todas las monedas y llegar a la meta `G`.
    CollectCoinsAndExit,
    /// Recolectar los objetos `I` y entregarlos en la zona `D`.
    CollectItemsAndDeposit,
    /// Igual, pero con un monstruo acechando y escondites disponibles.
    CollectItemsAndDepositWithHunter,
}

pub struct LevelDefinition {
    pub id: usize,
    pub name: &'static str,
    pub map_path: &'static str,
    pub ceiling_color: Color,
    pub floor_color: Color,
    /// Brillo general de las paredes (1.0 = normal, mas bajo = mas oscuro).
    pub light: f32,
    pub objective: LevelObjective,
    /// Dificultad del monstruo. `None` = el nivel no tiene monstruo.
    pub hunter: Option<HunterConfig>,
}

pub const LEVEL_COUNT: usize = 5;

pub static LEVELS: [LevelDefinition; LEVEL_COUNT] = [
    LevelDefinition {
        id: 1,
        name: "GREEN GARDEN",
        map_path: "levels/level1.txt",
        ceiling_color: Color::new(120, 180, 220, 255),
        floor_color: Color::new(70, 120, 70, 255),
        light: 1.0,
        objective: LevelObjective::CollectCoinsAndExit,
        hunter: None,
    },
    LevelDefinition {
        id: 2,
        name: "FACTORY",
        map_path: "levels/level2.txt",
        ceiling_color: Color::new(70, 85, 80, 255),
        floor_color: Color::new(60, 65, 60, 255),
        light: 0.8,
        objective: LevelObjective::CollectCoinsAndExit,
        hunter: None,
    },
    LevelDefinition {
        id: 3,
        name: "RUST HALLS",
        map_path: "levels/level3.txt",
        ceiling_color: Color::new(55, 35, 30, 255),
        floor_color: Color::new(45, 30, 25, 255),
        light: 0.62,
        objective: LevelObjective::CollectItemsAndDeposit,
        hunter: None,
    },
    LevelDefinition {
        id: 4,
        name: "COLD DEPTHS",
        map_path: "levels/level4.txt",
        ceiling_color: Color::new(25, 32, 48, 255),
        floor_color: Color::new(22, 26, 34, 255),
        light: 0.48,
        objective: LevelObjective::CollectItemsAndDepositWithHunter,
        hunter: Some(HunterConfig {
            speed: 2.4,
            detection_range: 9.0,
            fov: 1.4, // ~80 grados
            damage_per_second: 35.0,
            search_time: 6.0,
            guard_change_interval: 9.0,
            can_check_seen_hideouts: false,
        }),
    },
    LevelDefinition {
        id: 5,
        name: "DESCENT",
        map_path: "levels/level5.txt",
        ceiling_color: Color::new(30, 10, 12, 255),
        floor_color: Color::new(20, 8, 8, 255),
        light: 0.36,
        objective: LevelObjective::CollectItemsAndDepositWithHunter,
        hunter: Some(HunterConfig {
            speed: 2.9,
            detection_range: 12.0,
            fov: 1.8, // ~103 grados
            damage_per_second: 50.0,
            search_time: 10.0,
            guard_change_interval: 7.0,
            can_check_seen_hideouts: true,
        }),
    },
];
