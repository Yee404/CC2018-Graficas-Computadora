// Proyecto 1 - Raycaster estilo retro (motor base)
mod app;
mod assets;
mod enemy;
mod entities;
mod game;
mod hunter;
mod levels;
mod maze;
mod menu;
mod minimap;
mod pathfind;
mod player;
mod raycaster;
mod renderer;

use app::App;
use assets::Assets;

// --- Configuracion global del motor -----------------------------------------
// Resolucion moderada 16:9: barata de renderizar y facil de portar.
pub const WINDOW_WIDTH: i32 = 960;
pub const WINDOW_HEIGHT: i32 = 540;

// Columnas logicas de raycasting. Puede ser menor que WINDOW_WIDTH:
// cada columna se escala horizontalmente. Bajar este valor abarata el render
// sin tocar el tamano de la ventana (util para hardware modesto).
pub const RAY_COUNT: usize = 480;

pub const FOV: f32 = std::f32::consts::PI / 3.0; // 60 grados
pub const TARGET_FPS: u32 = 60;
pub const MOUSE_SENSITIVITY: f32 = 0.0022;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Proyecto 1 - Raycaster")
        .build();

    rl.set_target_fps(TARGET_FPS);
    // ESC ya no cierra la ventana: cada estado decide que hace con ESC.
    rl.set_exit_key(None);

    // Los recursos se cargan UNA sola vez, antes del bucle. Lo que no
    // exista queda como fallback geometrico.
    let assets = Assets::load(&mut rl, &thread);

    let mut app = App::new();

    while !rl.window_should_close() && !app.quit {
        let dt = rl.get_frame_time();

        // INPUT -> UPDATE -> RENDER quedan separados a proposito, para poder
        // cambiar despues el dispositivo de entrada o la plataforma.
        app.update(&mut rl, dt);

        let fps = rl.get_fps();
        let mut d = rl.begin_drawing(&thread);
        app.render(&mut d, fps, &assets);
    }
}
