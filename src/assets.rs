// Gestion minima de recursos graficos.
//
// Todavia NO existe arte: todas las texturas se cargan como `Option` y, cuando
// faltan (que es el caso actual), el juego usa el fallback geometrico de
// siempre. Nada aqui puede provocar panic ni bloquear el arranque.
//
// Reglas:
//   - se carga UNA sola vez, antes del bucle principal (`Assets::load`);
//   - dentro del bucle solo se consultan referencias ya cargadas;
//   - los `Texture2D` se liberan solos al destruirse `Assets`, que vive menos
//     que la ventana.
//
// Rutas esperadas cuando exista el arte (no se crean aqui):
//   assets/textures/wall_1.png | wall_2.png | wall_3.png
//   assets/sprites/coins/coin.png
//   assets/sprites/items/item.png
//   assets/sprites/enemies/enemy.png
//   assets/sprites/hunter/hunter.png
//   assets/sprites/hunter/hunter_walk.png   (sprite sheet horizontal)
//   assets/ui/deposit.png | hideout.png
//   assets/ui/menu_background.png | menu_character.png | locker_overlay.png
use std::path::Path;

use raylib::prelude::*;

use crate::entities::EntityKind;

/// Frames y duracion de la animacion del monstruo cuando exista su hoja.
const HUNTER_WALK_FRAMES: u32 = 4;
const HUNTER_WALK_FRAME_DURATION: f32 = 0.15;

/// Sprite animado por hoja horizontal de frames del mismo tamano.
/// El frame se elige por TIEMPO ACUMULADO, nunca por numero de iteraciones,
/// asi que la animacion es independiente de los FPS.
pub struct AnimatedSprite {
    pub sheet: Texture2D,
    pub frame_count: u32,
    pub frame_duration: f32,
}

impl AnimatedSprite {
    pub fn frame_index(&self, time: f32) -> u32 {
        if self.frame_count == 0 || self.frame_duration <= 0.0 {
            return 0;
        }
        let total = self.frame_duration * self.frame_count as f32;
        let t = time.rem_euclid(total);
        ((t / self.frame_duration) as u32).min(self.frame_count - 1)
    }

    /// Rectangulo del frame actual dentro de la hoja.
    pub fn frame_rect(&self, time: f32) -> Rectangle {
        let frames = self.frame_count.max(1);
        let w = self.sheet.width as f32 / frames as f32;
        Rectangle::new(
            self.frame_index(time) as f32 * w,
            0.0,
            w,
            self.sheet.height as f32,
        )
    }
}

const SPRITE_SLOTS: usize = 6;

fn sprite_slot(kind: SpriteKind) -> usize {
    match kind {
        SpriteKind::Coin => 0,
        SpriteKind::Item => 1,
        SpriteKind::Deposit => 2,
        SpriteKind::Hideout => 3,
        SpriteKind::Enemy => 4,
        SpriteKind::Hunter => 5,
    }
}

#[derive(Clone, Copy)]
pub enum SpriteKind {
    Coin,
    Item,
    Deposit,
    Hideout,
    Enemy,
    Hunter,
}

impl From<EntityKind> for SpriteKind {
    fn from(kind: EntityKind) -> SpriteKind {
        match kind {
            EntityKind::Coin => SpriteKind::Coin,
            EntityKind::Item => SpriteKind::Item,
            EntityKind::Deposit => SpriteKind::Deposit,
            EntityKind::Hideout => SpriteKind::Hideout,
        }
    }
}

pub struct Assets {
    /// Texturas de pared, en el mismo orden que los tipos '#', 'B' y 'C'.
    walls: [Option<Texture2D>; 3],
    sprites: [Option<Texture2D>; SPRITE_SLOTS],
    hunter_walk: Option<AnimatedSprite>,
    pub menu_background: Option<Texture2D>,
    pub menu_character: Option<Texture2D>,
    pub locker_overlay: Option<Texture2D>,
}

impl Assets {
    /// Se llama UNA vez, despues de crear la ventana y antes del bucle.
    /// Lo que falte queda en `None` y el juego sigue con las formas actuales.
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Assets {
        Assets {
            walls: [
                try_load(rl, thread, "assets/textures/wall_1.png"),
                try_load(rl, thread, "assets/textures/wall_2.png"),
                try_load(rl, thread, "assets/textures/wall_3.png"),
            ],
            sprites: [
                try_load(rl, thread, "assets/sprites/coins/coin.png"),
                try_load(rl, thread, "assets/sprites/items/item.png"),
                try_load(rl, thread, "assets/ui/deposit.png"),
                try_load(rl, thread, "assets/ui/hideout.png"),
                try_load(rl, thread, "assets/sprites/enemies/enemy.png"),
                try_load(rl, thread, "assets/sprites/hunter/hunter.png"),
            ],
            hunter_walk: try_load(rl, thread, "assets/sprites/hunter/hunter_walk.png").map(
                |sheet| AnimatedSprite {
                    sheet,
                    frame_count: HUNTER_WALK_FRAMES,
                    frame_duration: HUNTER_WALK_FRAME_DURATION,
                },
            ),
            menu_background: try_load(rl, thread, "assets/ui/menu_background.png"),
            menu_character: try_load(rl, thread, "assets/ui/menu_character.png"),
            locker_overlay: try_load(rl, thread, "assets/ui/locker_overlay.png"),
        }
    }

    /// Textura del tipo de pared, si existe. El caracter viene de `Hit::tile`.
    pub fn wall(&self, tile: u8) -> Option<&Texture2D> {
        let slot = match tile {
            b'#' => 0,
            b'B' => 1,
            b'C' => 2,
            _ => return None,
        };
        self.walls[slot].as_ref()
    }

    pub fn sprite(&self, kind: SpriteKind) -> Option<&Texture2D> {
        self.sprites[sprite_slot(kind)].as_ref()
    }

    /// Animacion del monstruo (billboard frontal). Todavia no existe la hoja.
    pub fn hunter_walk(&self) -> Option<&AnimatedSprite> {
        self.hunter_walk.as_ref()
    }
}

/// Carga silenciosa: si el archivo no esta (situacion normal por ahora)
/// devuelve `None`; si esta pero falla, avisa por stderr y sigue sin el.
fn try_load(rl: &mut RaylibHandle, thread: &RaylibThread, path: &str) -> Option<Texture2D> {
    if !Path::new(path).exists() {
        return None;
    }
    match rl.load_texture(thread, path) {
        Ok(tex) => Some(tex),
        Err(e) => {
            eprintln!("No se pudo cargar '{}': {}", path, e);
            None
        }
    }
}
