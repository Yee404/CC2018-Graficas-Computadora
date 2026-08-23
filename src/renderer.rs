// Render pseudo-3D: cielo, suelo, columnas de pared y billboards de entidades.
//
// Cada elemento tiene DOS caminos: si su textura existe en `Assets` se dibuja
// con ella; si no (situacion actual, todavia sin arte), se usa exactamente el
// mismo relleno geometrico de color de siempre. La proyeccion, la oclusion y
// el raycasting son identicos en ambos casos.
use raylib::prelude::*;

use crate::assets::{Assets, SpriteKind};
use crate::enemy::Enemy;
use crate::entities::{Entity, EntityKind};
use crate::hunter::Hunter;
use crate::levels::{LevelDefinition, LevelObjective};
use crate::player::Player;
use crate::raycaster::Hit;
use crate::{FOV, WINDOW_HEIGHT, WINDOW_WIDTH};

/// Color base segun el caracter de la celda (no segun su posicion).
/// Es el fallback cuando el tipo de pared no tiene textura cargada.
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

pub fn draw_world(
    d: &mut RaylibDrawHandle,
    hits: &[Hit],
    level: &LevelDefinition,
    assets: &Assets,
) {
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

        match assets.wall(hit.tile) {
            // Con textura: una tira vertical de 1 px de ancho segun wall_x.
            Some(tex) => {
                let src =
                    Rectangle::new(hit.wall_x * tex.width as f32, 0.0, 1.0, tex.height as f32);
                let dest = Rectangle::new(x0 as f32, top as f32, w as f32, (bottom - top) as f32);
                d.draw_texture_pro(
                    tex,
                    src,
                    dest,
                    Vector2::zero(),
                    0.0,
                    shade(Color::WHITE, hit.dist, hit.side, level.light),
                );
            }
            // Sin textura: color solido de siempre.
            None => d.draw_rectangle(
                x0,
                top,
                w,
                bottom - top,
                shade(wall_color(hit.tile), hit.dist, hit.side, level.light),
            ),
        }
    }
}

// --- Billboards de entidades -------------------------------------------------
// Colores de fallback mientras no existan sprites.
pub const COIN_COLOR: Color = Color::new(240, 200, 60, 255);
pub const ITEM_COLOR: Color = Color::new(80, 200, 255, 255);
pub const DEPOSIT_COLOR: Color = Color::new(60, 230, 110, 255);
pub const ENEMY_COLOR: Color = Color::new(220, 60, 60, 255);
pub const HUNTER_COLOR: Color = Color::new(210, 60, 220, 255);
pub const HIDEOUT_COLOR: Color = Color::new(70, 110, 240, 255);
pub const PLAYER_COLOR: Color = Color::new(255, 255, 255, 255);

/// (color de fallback, lado en celdas, cuanto baja respecto al horizonte)
fn entity_visual(kind: EntityKind) -> (Color, f32, f32) {
    match kind {
        EntityKind::Coin => (COIN_COLOR, 0.42, 0.22),
        EntityKind::Item => (ITEM_COLOR, 0.50, 0.18),
        EntityKind::Deposit => (DEPOSIT_COLOR, 1.00, 0.00),
        EntityKind::Hideout => (HIDEOUT_COLOR, 0.90, 0.05),
    }
}

/// Textura del billboard y, si es un sprite animado, el frame que toca.
/// `None` significa "todavia no hay arte": se dibuja el rectangulo de color.
struct Billboard<'a> {
    texture: Option<&'a Texture2D>,
    frame: Option<Rectangle>,
}

impl<'a> Billboard<'a> {
    fn none() -> Billboard<'a> {
        Billboard {
            texture: None,
            frame: None,
        }
    }

    fn still(texture: Option<&'a Texture2D>) -> Billboard<'a> {
        Billboard {
            texture,
            frame: None,
        }
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
    art: Billboard,
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
    let tint = shade(color, depth, 0, 1.0);
    let span = (col_end - col_start).max(1) as f32;

    for col in col_start.max(0)..col_end.min(n as i32) {
        if hits[col as usize].dist <= depth {
            continue; // tapado por una pared
        }
        let x0 = col * WINDOW_WIDTH / n as i32;
        let x1 = (col + 1) * WINDOW_WIDTH / n as i32;
        let w = (x1 - x0).max(1);

        match art.texture {
            // Con sprite: tira vertical del PNG (respeta su transparencia).
            Some(tex) => {
                let full = Rectangle::new(0.0, 0.0, tex.width as f32, tex.height as f32);
                let frame = art.frame.unwrap_or(full);
                let u = (col - col_start) as f32 / span;
                let src = Rectangle::new(
                    frame.x + u * frame.width,
                    frame.y,
                    frame.width / span,
                    frame.height,
                );
                let dest = Rectangle::new(x0 as f32, top as f32, w as f32, (bottom - top) as f32);
                d.draw_texture_pro(
                    tex,
                    src,
                    dest,
                    Vector2::zero(),
                    0.0,
                    shade(Color::WHITE, depth, 0, 1.0),
                );
            }
            // Sin sprite: rectangulo de color, como hasta ahora.
            None => d.draw_rectangle(x0, top, w, bottom - top, tint),
        }
    }
}

/// Dibuja entidades, enemigos y monstruo de lejos a cerca.
/// `order` viene ordenado desde `Game` (buffer reutilizado): los indices
/// menores que `entities.len()` son entidades; el resto, enemigos y monstruo.
/// `anim_time` es tiempo acumulado con delta time, no un contador de frames.
#[allow(clippy::too_many_arguments)]
pub fn draw_billboards(
    d: &mut RaylibDrawHandle,
    hits: &[Hit],
    player: &Player,
    entities: &[Entity],
    enemies: &[Enemy],
    hunter: Option<&Hunter>,
    order: &[(f32, usize)],
    assets: &Assets,
    anim_time: f32,
) {
    for &(_, idx) in order {
        if idx < entities.len() {
            let e = &entities[idx];
            if !e.active {
                continue;
            }
            let (color, size, offset) = entity_visual(e.kind);
            let art = Billboard::still(assets.sprite(SpriteKind::from(e.kind)));
            draw_billboard(d, hits, player, e.x, e.y, size, offset, color, art);
        } else if idx - entities.len() < enemies.len() {
            let e = &enemies[idx - entities.len()];
            let art = Billboard::still(assets.sprite(SpriteKind::Enemy));
            draw_billboard(d, hits, player, e.x, e.y, 0.8, 0.05, ENEMY_COLOR, art);
        } else if let Some(h) = hunter {
            // El monstruo usa su hoja animada si existe; si no, el sprite fijo;
            // y si tampoco, el rectangulo magenta actual.
            let art = match assets.hunter_walk() {
                Some(anim) => Billboard {
                    texture: Some(&anim.sheet),
                    frame: Some(anim.frame_rect(anim_time)),
                },
                None => match assets.sprite(SpriteKind::Hunter) {
                    Some(tex) => Billboard::still(Some(tex)),
                    None => Billboard::none(),
                },
            };
            draw_billboard(d, hits, player, h.x, h.y, 1.0, 0.0, HUNTER_COLOR, art);
        }
    }
}

// --- Overlay de escondite ----------------------------------------------------
/// Vista desde dentro del casillero: casi todo oscuro con dos rendijas.
/// Se dibuja despues del mundo y antes del HUD, para que `HIDDEN` y
/// `E - LEAVE` sigan leyendose por encima.
pub fn draw_hideout_overlay(d: &mut RaylibDrawHandle, assets: &Assets) {
    // Con arte: un PNG con transparencia en las rendijas.
    if let Some(tex) = assets.locker_overlay.as_ref() {
        d.draw_texture_pro(
            tex,
            Rectangle::new(0.0, 0.0, tex.width as f32, tex.height as f32),
            Rectangle::new(0.0, 0.0, WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        return;
    }

    // Fallback geometrico: bandas opacas dejando dos rendijas horizontales.
    let dark = Color::new(8, 8, 10, 245);
    let h = WINDOW_HEIGHT as f32;
    let slit_h = (h * 0.09) as i32;
    let slit_1 = (h * 0.40) as i32;
    let slit_2 = (h * 0.60) as i32;

    d.draw_rectangle(0, 0, WINDOW_WIDTH, slit_1, dark);
    d.draw_rectangle(
        0,
        slit_1 + slit_h,
        WINDOW_WIDTH,
        slit_2 - (slit_1 + slit_h),
        dark,
    );
    d.draw_rectangle(
        0,
        slit_2 + slit_h,
        WINDOW_WIDTH,
        WINDOW_HEIGHT - (slit_2 + slit_h),
        dark,
    );

    // Marcos laterales: refuerzan la sensacion de estar dentro de un mueble.
    let side = (WINDOW_WIDTH as f32 * 0.06) as i32;
    d.draw_rectangle(0, 0, side, WINDOW_HEIGHT, dark);
    d.draw_rectangle(WINDOW_WIDTH - side, 0, side, WINDOW_HEIGHT, dark);

    // Penumbra suave sobre las propias rendijas.
    let haze = Color::new(0, 0, 0, 70);
    d.draw_rectangle(side, slit_1, WINDOW_WIDTH - side * 2, slit_h, haze);
    d.draw_rectangle(side, slit_2, WINDOW_WIDTH - side * 2, slit_h, haze);
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
