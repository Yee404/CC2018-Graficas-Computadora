// Render pseudo-3D: cielo, suelo, columnas de pared y billboards de entidades.
// Todo con colores solidos: la representacion visual es PROVISIONAL, pero la
// proyeccion y la oclusion no cambiaran al sustituirla por sprites.
use raylib::prelude::*;

use crate::enemy::Enemy;
use crate::entities::{Entity, EntityKind};
use crate::hunter::Hunter;
use crate::levels::{LevelDefinition, LevelObjective};
use crate::player::Player;
use crate::raycaster::Hit;
use crate::{FOV, WINDOW_HEIGHT, WINDOW_WIDTH};

/// Color base segun el caracter de la celda (no segun su posicion).
/// Mas adelante esta funcion es el unico punto a cambiar para usar texturas.
fn wall_color(tile: u8) -> Color {
    match tile {
        b'#' => Color::new(180, 180, 190, 255), // tipo 1: gris piedra
        b'B' => Color::new(170, 95, 60, 255),   // tipo 2: ladrillo
        b'C' => Color::new(80, 150, 110, 255),  // tipo 3: verde
        _ => Color::new(140, 140, 140, 255),
    }
}

/// Sombreado barato: caras horizontales mas oscuras, atenuacion por distancia
/// y brillo global del nivel (atmosfera provisional, no es iluminacion real).
fn shade(color: Color, dist: f32, side: u8, light: f32) -> Color {
    let mut f = light / (1.0 + dist * 0.12);
    if side == 1 {
        f *= 0.72;
    }
    let f = f.clamp(0.10, 1.0);
    Color::new(
        (color.r as f32 * f) as u8,
        (color.g as f32 * f) as u8,
        (color.b as f32 * f) as u8,
        255,
    )
}

pub fn draw_world(d: &mut RaylibDrawHandle, hits: &[Hit], level: &LevelDefinition) {
    let half_h = WINDOW_HEIGHT / 2;
    d.draw_rectangle(0, 0, WINDOW_WIDTH, half_h, level.ceiling_color);
    d.draw_rectangle(
        0,
        half_h,
        WINDOW_WIDTH,
        WINDOW_HEIGHT - half_h,
        level.floor_color,
    );

    let n = hits.len() as i32;
    for (i, hit) in hits.iter().enumerate() {
        // Cada columna logica se escala a uno o mas pixeles de pantalla.
        let x0 = (i as i32) * WINDOW_WIDTH / n;
        let x1 = (i as i32 + 1) * WINDOW_WIDTH / n;
        let w = (x1 - x0).max(1);

        let line_h = (WINDOW_HEIGHT as f32 / hit.dist) as i32;
        let top = (half_h - line_h / 2).max(0);
        let bottom = (half_h + line_h / 2).min(WINDOW_HEIGHT);
        if bottom <= top {
            continue;
        }

        d.draw_rectangle(
            x0,
            top,
            w,
            bottom - top,
            shade(wall_color(hit.tile), hit.dist, hit.side, level.light),
        );
    }
}

// --- Billboards de entidades -------------------------------------------------
// Representacion visual PROVISIONAL: rectangulos de color solido. Al pasar a
// sprites PNG solo cambia el relleno de `draw_billboard`.
pub const COIN_COLOR: Color = Color::new(240, 200, 60, 255);
pub const ITEM_COLOR: Color = Color::new(80, 200, 255, 255);
pub const DEPOSIT_COLOR: Color = Color::new(60, 230, 110, 255);
pub const ENEMY_COLOR: Color = Color::new(220, 60, 60, 255);
pub const HUNTER_COLOR: Color = Color::new(210, 60, 220, 255);
pub const HIDEOUT_COLOR: Color = Color::new(70, 110, 240, 255);
pub const PLAYER_COLOR: Color = Color::new(255, 255, 255, 255);

/// (color, lado en celdas, cuanto baja respecto al horizonte)
fn entity_visual(kind: EntityKind) -> (Color, f32, f32) {
    match kind {
        EntityKind::Coin => (COIN_COLOR, 0.42, 0.22),
        EntityKind::Item => (ITEM_COLOR, 0.50, 0.18),
        EntityKind::Deposit => (DEPOSIT_COLOR, 1.00, 0.00),
        EntityKind::Hideout => (HIDEOUT_COLOR, 0.90, 0.05),
    }
}

/// Dibuja un billboard en coordenadas del mundo.
/// `hits` hace de buffer de profundidad: una columna solo se pinta si la pared
/// de esa columna esta mas lejos que la entidad.
#[allow(clippy::too_many_arguments)]
fn draw_billboard(
    d: &mut RaylibDrawHandle,
    hits: &[Hit],
    player: &Player,
    x: f32,
    y: f32,
    size: f32,
    y_offset: f32,
    color: Color,
) {
    let n = hits.len();
    if n == 0 {
        return;
    }
    let half_h = WINDOW_HEIGHT / 2;
    let plane_len = (FOV * 0.5).tan();
    let (sin, cos) = player.angle.sin_cos();

    // Coordenadas de camara: depth = distancia perpendicular,
    // lateral = desplazamiento horizontal en unidades del plano.
    let rel_x = x - player.x;
    let rel_y = y - player.y;
    let depth = rel_x * cos + rel_y * sin;
    if depth < 0.15 {
        return; // detras del jugador o demasiado cerca
    }
    let lateral = (-rel_x * sin + rel_y * cos) / plane_len;

    let center_col = n as f32 * 0.5 * (1.0 + lateral / depth);
    let half_w_cols = n as f32 * 0.5 * (size * 0.5) / (plane_len * depth);
    let col_start = (center_col - half_w_cols).floor() as i32;
    let col_end = (center_col + half_w_cols).ceil() as i32;
    if col_end < 0 || col_start >= n as i32 {
        return; // fuera del campo de vision
    }

    let height_px = (size * WINDOW_HEIGHT as f32 / depth) as i32;
    let center_y = half_h + (y_offset * WINDOW_HEIGHT as f32 / depth) as i32;
    let top = (center_y - height_px / 2).max(0);
    let bottom = (center_y + height_px / 2).min(WINDOW_HEIGHT);
    if bottom <= top {
        return;
    }

    // Las entidades se mantienen legibles aunque el nivel sea oscuro.
    let color = shade(color, depth, 0, 1.0);
    for col in col_start.max(0)..col_end.min(n as i32) {
        if hits[col as usize].dist <= depth {
            continue; // tapado por una pared
        }
        let x0 = col * WINDOW_WIDTH / n as i32;
        let x1 = (col + 1) * WINDOW_WIDTH / n as i32;
        d.draw_rectangle(x0, top, (x1 - x0).max(1), bottom - top, color);
    }
}

/// Dibuja entidades y enemigos de lejos a cerca.
/// `order` viene ordenado desde `Game` (buffer reutilizado): los indices
/// menores que `entities.len()` son entidades; el resto, enemigos.
pub fn draw_billboards(
    d: &mut RaylibDrawHandle,
    hits: &[Hit],
    player: &Player,
    entities: &[Entity],
    enemies: &[Enemy],
    hunter: Option<&Hunter>,
    order: &[(f32, usize)],
) {
    for &(_, idx) in order {
        if idx < entities.len() {
            let e = &entities[idx];
            if !e.active {
                continue;
            }
            let (color, size, offset) = entity_visual(e.kind);
            draw_billboard(d, hits, player, e.x, e.y, size, offset, color);
        } else if idx - entities.len() < enemies.len() {
            let e = &enemies[idx - entities.len()];
            draw_billboard(d, hits, player, e.x, e.y, 0.8, 0.05, ENEMY_COLOR);
        } else if let Some(h) = hunter {
            draw_billboard(d, hits, player, h.x, h.y, 1.0, 0.0, HUNTER_COLOR);
        }
    }
}

// --- HUD ---------------------------------------------------------------------
pub struct Hud<'a> {
    pub fps: u32,
    pub objective: LevelObjective,
    pub coins: (u32, u32),
    pub items: (u32, u32),
    /// Solo se muestra en niveles con enemigos.
    pub health: Option<f32>,
    pub message: Option<&'a str>,
    /// Aviso de interaccion (por ejemplo "E - HIDE"), abajo al centro.
    pub hint: Option<&'a str>,
    /// 0..1: intensidad del flash rojo al recibir dano.
    pub damage_flash: f32,
}

pub fn draw_hud(d: &mut RaylibDrawHandle, hud: &Hud) {
    if hud.damage_flash > 0.0 {
        let alpha = (hud.damage_flash.clamp(0.0, 1.0) * 120.0) as u8;
        d.draw_rectangle(
            0,
            0,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            Color::new(200, 30, 30, alpha),
        );
    }

    d.draw_text(&format!("FPS: {}", hud.fps), 10, 10, 20, Color::LIME);

    match hud.objective {
        LevelObjective::CollectCoinsAndExit => {
            d.draw_text(
                &format!("COINS: {} / {}", hud.coins.0, hud.coins.1),
                10,
                34,
                20,
                COIN_COLOR,
            );
        }
        LevelObjective::CollectItemsAndDeposit
        | LevelObjective::CollectItemsAndDepositWithHunter => {
            d.draw_text(
                &format!("ITEMS: {} / {}", hud.items.0, hud.items.1),
                10,
                34,
                20,
                ITEM_COLOR,
            );
        }
    }

    if let Some(hp) = hud.health {
        d.draw_text(
            &format!("HEALTH: {}", hp.max(0.0).round() as i32),
            10,
            58,
            20,
            Color::new(240, 90, 90, 255),
        );
    }

    if let Some(text) = hud.hint {
        let size = 24;
        let w = d.measure_text(text, size);
        d.draw_text(
            text,
            WINDOW_WIDTH / 2 - w / 2,
            WINDOW_HEIGHT - 70,
            size,
            HIDEOUT_COLOR,
        );
    }

    if let Some(text) = hud.message {
        let size = 34;
        let w = d.measure_text(text, size);
        d.draw_text(
            text,
            WINDOW_WIDTH / 2 - w / 2,
            WINDOW_HEIGHT / 2 - size / 2,
            size,
            Color::GOLD,
        );
    }
}
