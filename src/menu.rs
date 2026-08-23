// Pantallas de menu provisionales: solo texto, rectangulos y colores.
// La navegacion por teclado y por mouse comparten el mismo indice, para poder
// mapear despues las mismas acciones a un gamepad.
use raylib::prelude::*;

use crate::assets::Assets;
use crate::levels::{LEVELS, LEVEL_COUNT};
use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

const BUTTON_W: i32 = 300;
const BUTTON_H: i32 = 46;
const BUTTON_GAP: i32 = 12;
const FIRST_BUTTON_Y: i32 = 200;

pub fn button_rect(i: usize) -> Rectangle {
    Rectangle::new(
        (WINDOW_WIDTH / 2 - BUTTON_W / 2) as f32,
        (FIRST_BUTTON_Y + i as i32 * (BUTTON_H + BUTTON_GAP)) as f32,
        BUTTON_W as f32,
        BUTTON_H as f32,
    )
}

/// Actualiza el indice seleccionado (flechas o mouse) y devuelve el indice
/// activado con Enter o con clic izquierdo.
pub fn interact(rl: &RaylibHandle, count: usize, index: &mut usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_DOWN) || rl.is_key_pressed(KeyboardKey::KEY_S) {
        *index = (*index + 1) % count;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_UP) || rl.is_key_pressed(KeyboardKey::KEY_W) {
        *index = (*index + count - 1) % count;
    }

    let mouse = rl.get_mouse_position();
    let mut hovered = None;
    for i in 0..count {
        if button_rect(i).check_collision_point_rec(mouse) {
            hovered = Some(i);
            *index = i;
        }
    }

    if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_KP_ENTER) {
        return Some(*index);
    }
    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        return hovered;
    }
    None
}

fn draw_title(d: &mut RaylibDrawHandle, title: &str, subtitle: &str, assets: &Assets) {
    d.clear_background(Color::new(16, 16, 22, 255));

    // Con arte: fondo y personaje del menu. Sin arte: fondo liso actual.
    if let Some(tex) = assets.menu_background.as_ref() {
        d.draw_texture_pro(
            tex,
            Rectangle::new(0.0, 0.0, tex.width as f32, tex.height as f32),
            Rectangle::new(0.0, 0.0, WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
    }
    if let Some(tex) = assets.menu_character.as_ref() {
        d.draw_texture(
            tex,
            WINDOW_WIDTH - tex.width - 20,
            WINDOW_HEIGHT - tex.height - 20,
            Color::WHITE,
        );
    }
    let size = 60;
    let w = d.measure_text(title, size);
    d.draw_text(title, WINDOW_WIDTH / 2 - w / 2, 90, size, Color::GOLD);
    let w = d.measure_text(subtitle, 20);
    d.draw_text(
        subtitle,
        WINDOW_WIDTH / 2 - w / 2,
        160,
        20,
        Color::new(160, 160, 170, 255),
    );
}

fn draw_button(d: &mut RaylibDrawHandle, i: usize, label: &str, selected: bool, locked: bool) {
    let r = button_rect(i);
    let (bg, fg) = if locked {
        (Color::new(35, 35, 40, 255), Color::new(90, 90, 95, 255))
    } else if selected {
        (Color::new(90, 80, 40, 255), Color::GOLD)
    } else {
        (Color::new(40, 42, 52, 255), Color::new(210, 210, 215, 255))
    };
    d.draw_rectangle_rec(r, bg);
    d.draw_rectangle_lines_ex(r, 2.0, fg);

    let size = 22;
    let w = d.measure_text(label, size);
    d.draw_text(
        label,
        r.x as i32 + r.width as i32 / 2 - w / 2,
        r.y as i32 + r.height as i32 / 2 - size / 2,
        size,
        fg,
    );
}

pub fn draw_main_menu(d: &mut RaylibDrawHandle, index: usize, assets: &Assets) {
    draw_title(d, "DESCENT", "ENTER / CLICK PARA SELECCIONAR", assets);
    draw_button(d, 0, "PLAY", index == 0, false);
    draw_button(d, 1, "QUIT", index == 1, false);
}

pub fn draw_level_select(d: &mut RaylibDrawHandle, index: usize, unlocked: usize, assets: &Assets) {
    draw_title(d, "LEVEL SELECT", "ESC PARA VOLVER AL MENU", assets);
    for i in 0..LEVEL_COUNT {
        let locked = i >= unlocked;
        let label = if locked {
            format!("LEVEL {} - LOCKED", i + 1)
        } else {
            format!("LEVEL {} - {}", i + 1, LEVELS[i].name)
        };
        draw_button(d, i, &label, index == i, locked);
    }
}

fn draw_overlay(d: &mut RaylibDrawHandle) {
    d.draw_rectangle(
        0,
        0,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        Color::new(0, 0, 0, 200), // se dibuja encima del nivel congelado
    );
}

pub fn draw_level_complete(d: &mut RaylibDrawHandle, level_id: usize, summary: &str) {
    draw_overlay(d);

    let text = "LEVEL COMPLETE";
    let size = 50;
    let w = d.measure_text(text, size);
    d.draw_text(text, WINDOW_WIDTH / 2 - w / 2, 140, size, Color::GOLD);

    let sub = format!("LEVEL {} - {}", level_id, summary);
    let w = d.measure_text(&sub, 22);
    d.draw_text(&sub, WINDOW_WIDTH / 2 - w / 2, 210, 22, Color::RAYWHITE);

    let lines = ["ENTER - CONTINUE", "R - REPLAY", "ESC - LEVEL SELECT"];
    for (i, line) in lines.iter().enumerate() {
        let w = d.measure_text(line, 24);
        d.draw_text(
            line,
            WINDOW_WIDTH / 2 - w / 2,
            290 + i as i32 * 36,
            24,
            Color::new(210, 210, 215, 255),
        );
    }
}

pub fn draw_game_over(d: &mut RaylibDrawHandle) {
    draw_overlay(d);

    let text = "YOU DIED";
    let size = 56;
    let w = d.measure_text(text, size);
    d.draw_text(
        text,
        WINDOW_WIDTH / 2 - w / 2,
        160,
        size,
        Color::new(220, 60, 60, 255),
    );

    for (i, line) in ["R - RETRY", "ESC - LEVEL SELECT"].iter().enumerate() {
        let w = d.measure_text(line, 24);
        d.draw_text(
            line,
            WINDOW_WIDTH / 2 - w / 2,
            280 + i as i32 * 36,
            24,
            Color::new(210, 210, 215, 255),
        );
    }
}
