// Maquina de estados de la aplicacion: menu, selector, gameplay y fin de nivel.
use raylib::prelude::*;

use crate::assets::Assets;
use crate::game::{self, Game};
use crate::levels::{LEVELS, LEVEL_COUNT};
use crate::menu;
use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub enum State {
    MainMenu,
    LevelSelect,
    Playing,
    LevelComplete,
    GameOver,
}

pub struct App {
    state: State,
    main_index: usize,
    select_index: usize,
    /// Cantidad de niveles desbloqueados (siempre >= 1). Sin guardado en disco.
    unlocked: usize,
    game: Option<Game>,
    current_level: usize,
    /// Frames de mouse ignorados tras capturar el cursor: evita el salto brusco.
    skip_mouse: u8,
    pub quit: bool,
}

impl App {
    pub fn new() -> App {
        App {
            state: State::MainMenu,
            main_index: 0,
            select_index: 0,
            unlocked: 1,
            game: None,
            current_level: 0,
            skip_mouse: 0,
            quit: false,
        }
    }

    fn start_level(&mut self, rl: &mut RaylibHandle, index: usize) {
        match Game::new(&LEVELS[index]) {
            Ok(g) => {
                self.game = Some(g); // el nivel anterior se descarga aqui
                self.current_level = index;
                self.state = State::Playing;
                rl.disable_cursor();
                self.skip_mouse = 2;
            }
            Err(e) => {
                eprintln!("Error cargando '{}': {}", LEVELS[index].map_path, e);
                self.game = None;
                self.state = State::LevelSelect;
                release_cursor(rl);
            }
        }
    }

    fn go_to_level_select(&mut self, rl: &mut RaylibHandle) {
        self.game = None;
        self.state = State::LevelSelect;
        release_cursor(rl);
    }

    pub fn update(&mut self, rl: &mut RaylibHandle, dt: f32) {
        match self.state {
            State::MainMenu => {
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    self.quit = true;
                }
                if let Some(choice) = menu::interact(rl, 2, &mut self.main_index) {
                    match choice {
                        0 => self.state = State::LevelSelect,
                        _ => self.quit = true,
                    }
                }
            }

            State::LevelSelect => {
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    self.state = State::MainMenu;
                }
                if let Some(choice) = menu::interact(rl, LEVEL_COUNT, &mut self.select_index) {
                    // Los niveles bloqueados no se pueden iniciar.
                    if choice < self.unlocked {
                        self.start_level(rl, choice);
                    }
                }
            }

            State::Playing => {
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    self.go_to_level_select(rl);
                    return;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_R) {
                    self.start_level(rl, self.current_level);
                    return;
                }

                let mut input = game::read_input(rl);
                if self.skip_mouse > 0 {
                    input.mouse_dx = 0.0;
                    self.skip_mouse -= 1;
                }

                if let Some(g) = self.game.as_mut() {
                    g.update(&input, dt);
                    if g.is_dead() {
                        // Morir no quita los desbloqueos ya obtenidos.
                        self.state = State::GameOver;
                        release_cursor(rl);
                    } else if g.is_complete() {
                        // Completar un nivel desbloquea el siguiente.
                        if self.current_level + 1 < LEVEL_COUNT {
                            self.unlocked = self.unlocked.max(self.current_level + 2);
                        }
                        self.select_index = (self.current_level + 1).min(LEVEL_COUNT - 1);
                        self.state = State::LevelComplete;
                        release_cursor(rl);
                    }
                }
            }

            State::LevelComplete | State::GameOver => {
                // El gameplay deja de actualizarse en ambos casos.
                if rl.is_key_pressed(KeyboardKey::KEY_R) {
                    self.start_level(rl, self.current_level);
                } else if rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                    || rl.is_key_pressed(KeyboardKey::KEY_KP_ENTER)
                    || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
                {
                    self.go_to_level_select(rl);
                }
            }
        }
    }

    pub fn render(&self, d: &mut RaylibDrawHandle, fps: u32, assets: &Assets) {
        match self.state {
            State::MainMenu => menu::draw_main_menu(d, self.main_index, assets),
            State::LevelSelect => {
                menu::draw_level_select(d, self.select_index, self.unlocked, assets)
            }
            State::Playing => {
                if let Some(g) = self.game.as_ref() {
                    g.render(d, fps, assets);
                }
            }
            State::LevelComplete => {
                if let Some(g) = self.game.as_ref() {
                    g.render(d, fps, assets); // nivel congelado de fondo
                    menu::draw_level_complete(d, LEVELS[self.current_level].id, &g.summary());
                }
            }
            State::GameOver => {
                if let Some(g) = self.game.as_ref() {
                    g.render(d, fps, assets); // nivel congelado de fondo
                    menu::draw_game_over(d);
                }
            }
        }
    }
}

/// Al salir del gameplay hay que devolver el cursor Y asegurar que la ventana
/// siga recibiendo teclado. En Windows, tras `disable_cursor()` el puntero
/// reaparece donde estaba antes de la captura (posiblemente fuera de la
/// ventana), y entonces el menu parece "muerto" hasta hacer clic. Por eso,
/// ademas de habilitar el cursor, lo recolocamos en el centro de la ventana y
/// pedimos el foco explicitamente. `set_window_focused()` es lo maximo que
/// ofrece raylib 5.5: si el gestor de ventanas lo ignora (algunos lo hacen por
/// politica anti-robo-de-foco) no existe forma portable de forzarlo mas.
fn release_cursor(rl: &mut RaylibHandle) {
    rl.enable_cursor();
    rl.set_mouse_position(Vector2::new(
        WINDOW_WIDTH as f32 / 2.0,
        WINDOW_HEIGHT as f32 / 2.0,
    ));
    rl.set_window_focused();
}
