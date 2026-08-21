// Minimapa estilo HUD, superpuesto en la esquina superior derecha.
use raylib::prelude::*;

use crate::enemy::Enemy;
use crate::entities::{Entity, EntityKind};
use crate::hunter::Hunter;
use crate::maze::Maze;
use crate::player::Player;
use crate::renderer::{
    COIN_COLOR, DEPOSIT_COLOR, ENEMY_COLOR, HIDEOUT_COLOR, HUNTER_COLOR, ITEM_COLOR, PLAYER_COLOR,
};
use crate::WINDOW_WIDTH;

const MARGIN: i32 = 10;
const MAX_SIZE: i32 = 180; // lado maximo en pixeles, sin importar el mapa

pub fn draw(
    d: &mut RaylibDrawHandle,
    maze: &Maze,
    player: &Player,
    entities: &[Entity],
    enemies: &[Enemy],
    hunter: Option<&Hunter>,
    show_goal: bool,
) {
    // Escala calculada para que un mapa grande siga cabiendo en el HUD.
    let cell = (MAX_SIZE / maze.width.max(maze.height) as i32).max(2);
    let w = cell * maze.width as i32;
    let h = cell * maze.height as i32;
    let ox = WINDOW_WIDTH - w - MARGIN;
    let oy = MARGIN;

    d.draw_rectangle(
        ox - 4,
        oy - 4,
        w + 8,
        h + 8,
        Color::new(0, 0, 0, 140), // fondo semitransparente
    );

    for y in 0..maze.height as i32 {
        for x in 0..maze.width as i32 {
            let color = if maze.is_solid(x, y) {
                match maze.tile(x, y) {
                    b'B' => Color::new(170, 95, 60, 200),
                    b'C' => Color::new(80, 150, 110, 200),
                    _ => Color::new(170, 170, 180, 200),
                }
            } else {
                Color::new(30, 30, 35, 160)
            };
            d.draw_rectangle(ox + x * cell, oy + y * cell, cell, cell, color);
        }
    }

    // La meta solo se dibuja si el objetivo del nivel la usa.
    if show_goal {
        d.draw_rectangle(
            ox + maze.goal_x as i32 * cell,
            oy + maze.goal_y as i32 * cell,
            cell,
            cell,
            Color::GOLD,
        );
    }

    // Entidades activas (lo recolectado desaparece)
    for e in entities {
        if !e.active {
            continue;
        }
        let ex = (ox as f32 + e.x * cell as f32) as i32;
        let ey = (oy as f32 + e.y * cell as f32) as i32;
        match e.kind {
            EntityKind::Coin => {
                d.draw_circle(ex, ey, (cell as f32 * 0.25).max(1.5), COIN_COLOR);
            }
            EntityKind::Item => {
                d.draw_circle(ex, ey, (cell as f32 * 0.3).max(2.0), ITEM_COLOR);
            }
            EntityKind::Deposit => {
                let s = (cell as f32 * 0.9).max(3.0) as i32;
                d.draw_rectangle(ex - s / 2, ey - s / 2, s, s, DEPOSIT_COLOR);
            }
            EntityKind::Hideout => {
                let s = (cell as f32 * 0.8).max(3.0) as i32;
                d.draw_rectangle(ex - s / 2, ey - s / 2, s, s, HIDEOUT_COLOR);
            }
        }
    }

    // Enemigos (visibles por ahora, util para depurar)
    for e in enemies {
        d.draw_circle(
            (ox as f32 + e.x * cell as f32) as i32,
            (oy as f32 + e.y * cell as f32) as i32,
            (cell as f32 * 0.3).max(2.0),
            ENEMY_COLOR,
        );
    }

    // Monstruo de los niveles 4-5
    if let Some(h) = hunter {
        d.draw_circle(
            (ox as f32 + h.x * cell as f32) as i32,
            (oy as f32 + h.y * cell as f32) as i32,
            (cell as f32 * 0.35).max(2.5),
            HUNTER_COLOR,
        );
    }

    // Jugador y su direccion
    let px = ox as f32 + player.x * cell as f32;
    let py = oy as f32 + player.y * cell as f32;
    let len = cell as f32 * 2.0;
    d.draw_line(
        px as i32,
        py as i32,
        (px + player.angle.cos() * len) as i32,
        (py + player.angle.sin() * len) as i32,
        PLAYER_COLOR,
    );
    d.draw_circle(
        px as i32,
        py as i32,
        (cell as f32 * 0.35).max(2.0),
        PLAYER_COLOR,
    );
}
