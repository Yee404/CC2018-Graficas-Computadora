// Monstruo de los niveles 4 y 5.
// A diferencia del enemigo simple del nivel 3 no es omnisciente: detecta al
// jugador por rango + angulo de vision + linea de vista, y se comporta segun
// una maquina de estados explicita. Toda la dificultad esta en `HunterConfig`,
// asi que L4 y L5 comparten exactamente el mismo codigo.
use crate::entities::{Entity, EntityKind};
use crate::maze::Maze;
use crate::pathfind::FlowField;

const HUNTER_RADIUS: f32 = 0.28;
/// Radio de dano por contacto.
pub const ATTACK_RADIUS: f32 = 0.8;
/// Cada cuanto se recalcula el BFS del monstruo.
const REPATH_INTERVAL: f32 = 0.35;
/// Distancia a la que se considera que llego a su objetivo.
const ARRIVE_DIST: f32 = 0.6;

#[derive(Clone, Copy)]
pub struct HunterConfig {
    pub speed: f32,
    pub detection_range: f32,
    /// Campo de vision total, en radianes.
    pub fov: f32,
    pub damage_per_second: f32,
    /// Cuanto sigue buscando tras perder de vista al jugador.
    pub search_time: f32,
    /// Cada cuanto cambia de objeto vigilado.
    pub guard_change_interval: f32,
    /// Nivel 5: puede ir a revisar un escondite donde vio entrar al jugador.
    pub can_check_seen_hideouts: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HunterState {
    /// Campea un objeto todavia no recolectado.
    Guard,
    /// Persigue al jugador (visible o hacia su ultima posicion conocida).
    Chase,
    /// Busca un rato alrededor de la ultima posicion conocida.
    Search,
    /// Solo nivel 5: va a revisar el escondite donde vio esconderse al jugador.
    CheckHideout,
}

pub struct Hunter {
    pub x: f32,
    pub y: f32,
    /// Hacia donde mira (direccion de su ultimo movimiento).
    pub angle: f32,
    pub state: HunterState,
    cfg: HunterConfig,
    flow: FlowField,
    sees_player: bool,
    last_known: Option<(f32, f32)>,
    guard_index: usize,
    guard_timer: f32,
    search_timer: f32,
    /// Escondite que va a revisar y su celda (nivel 5).
    hideout_target: Option<(usize, f32, f32)>,
    /// Escondite al que acaba de llegar; lo consume `Game` una sola vez.
    reached_hideout: Option<usize>,
}

impl Hunter {
    pub fn new(x: f32, y: f32, cfg: HunterConfig, maze: &Maze) -> Hunter {
        Hunter {
            x,
            y,
            angle: 0.0,
            state: HunterState::Guard,
            cfg,
            flow: FlowField::new(maze, REPATH_INTERVAL),
            sees_player: false,
            last_known: None,
            guard_index: 0,
            guard_timer: 0.0,
            search_timer: 0.0,
            hideout_target: None,
            reached_hideout: None,
        }
    }

    #[inline]
    pub fn dist2_to(&self, px: f32, py: f32) -> f32 {
        let dx = self.x - px;
        let dy = self.y - py;
        dx * dx + dy * dy
    }

    #[inline]
    pub fn sees_player(&self) -> bool {
        self.sees_player
    }

    #[inline]
    pub fn damage_per_second(&self) -> f32 {
        self.cfg.damage_per_second
    }

    /// Nivel 5: el monstruo vio al jugador entrar a un escondite.
    pub fn saw_player_hide(&mut self, hideout: usize, x: f32, y: f32) {
        if !self.cfg.can_check_seen_hideouts {
            return;
        }
        self.hideout_target = Some((hideout, x, y));
        self.state = HunterState::CheckHideout;
    }

    /// Escondite recien alcanzado (se consume una vez).
    pub fn take_reached_hideout(&mut self) -> Option<usize> {
        self.reached_hideout.take()
    }

    pub fn update(
        &mut self,
        maze: &Maze,
        entities: &[Entity],
        px: f32,
        py: f32,
        player_hidden: bool,
        dt: f32,
    ) {
        // Un jugador escondido no puede ser visto (en el nivel 5 el monstruo
        // solo puede aprovechar haberlo VISTO entrar, no verlo dentro).
        self.sees_player = !player_hidden && self.can_see(maze, px, py);

        if self.sees_player {
            self.last_known = Some((px, py));
            self.search_timer = self.cfg.search_time;
            self.state = HunterState::Chase;
            self.hideout_target = None;
        }

        let target = self.pick_target(entities, dt);
        if let Some((tx, ty)) = target {
            self.flow.update(maze, tx, ty, dt);
            self.step_towards(maze, tx, ty, dt);
        }
    }

    /// Devuelve el punto al que debe dirigirse segun el estado actual y hace
    /// las transiciones. Es la maquina de estados completa.
    fn pick_target(&mut self, entities: &[Entity], dt: f32) -> Option<(f32, f32)> {
        match self.state {
            HunterState::Chase => {
                let (lx, ly) = self.last_known?;
                if !self.sees_player && self.dist2_to(lx, ly) < ARRIVE_DIST * ARRIVE_DIST {
                    // Llego donde lo vio por ultima vez y no esta: buscar.
                    self.state = HunterState::Search;
                    self.search_timer = self.cfg.search_time;
                }
                Some((lx, ly))
            }

            HunterState::Search => {
                self.search_timer -= dt;
                if self.search_timer <= 0.0 {
                    // Abandona la busqueda y vuelve a campear objetos.
                    self.state = HunterState::Guard;
                    self.last_known = None;
                    return self.guard_target(entities, dt);
                }
                self.last_known
            }

            HunterState::CheckHideout => {
                let (idx, hx, hy) = self.hideout_target?;
                if self.dist2_to(hx, hy) < ARRIVE_DIST * ARRIVE_DIST {
                    self.reached_hideout = Some(idx);
                    self.hideout_target = None;
                    self.state = HunterState::Guard;
                    return self.guard_target(entities, dt);
                }
                Some((hx, hy))
            }

            HunterState::Guard => self.guard_target(entities, dt),
        }
    }

    /// Campea un objeto todavia no recolectado; cambia de objeto cada cierto
    /// tiempo o cuando el que vigilaba desaparece. Nunca cada frame.
    fn guard_target(&mut self, entities: &[Entity], dt: f32) -> Option<(f32, f32)> {
        let active: usize = entities
            .iter()
            .filter(|e| e.active && e.kind == EntityKind::Item)
            .count();
        if active == 0 {
            return None; // ya no queda nada que vigilar: se queda donde esta
        }

        self.guard_timer -= dt;
        if self.guard_timer <= 0.0 {
            self.guard_timer = self.cfg.guard_change_interval;
            self.guard_index = self.guard_index.wrapping_add(1);
        }

        entities
            .iter()
            .filter(|e| e.active && e.kind == EntityKind::Item)
            .nth(self.guard_index % active)
            .map(|e| (e.x, e.y))
    }

    fn step_towards(&mut self, maze: &Maze, tx: f32, ty: f32, dt: f32) {
        let cx = self.x.floor() as i32;
        let cy = self.y.floor() as i32;
        let (gx, gy) = match self.flow.next_cell(cx, cy) {
            Some((nx, ny)) => (nx as f32 + 0.5, ny as f32 + 0.5),
            None => (tx, ty), // misma celda o sin ruta: linea recta
        };

        let dx = gx - self.x;
        let dy = gy - self.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1.0e-4 {
            return;
        }
        self.angle = dy.atan2(dx);

        let step = self.cfg.speed * dt;
        let mx = dx / len * step;
        let my = dy / len * step;

        // Ejes por separado: se desliza contra las paredes en vez de clavarse.
        if !maze.circle_hits_wall(self.x + mx, self.y, HUNTER_RADIUS) {
            self.x += mx;
        }
        if !maze.circle_hits_wall(self.x, self.y + my, HUNTER_RADIUS) {
            self.y += my;
        }
    }

    /// Rango + angulo de vision + linea de vista. Se evalua una vez por frame.
    fn can_see(&self, maze: &Maze, px: f32, py: f32) -> bool {
        let dx = px - self.x;
        let dy = py - self.y;
        let dist2 = dx * dx + dy * dy;
        if dist2 > self.cfg.detection_range * self.cfg.detection_range {
            return false;
        }

        let mut diff = dy.atan2(dx) - self.angle;
        while diff > std::f32::consts::PI {
            diff -= std::f32::consts::TAU;
        }
        while diff < -std::f32::consts::PI {
            diff += std::f32::consts::TAU;
        }
        if diff.abs() > self.cfg.fov * 0.5 {
            return false;
        }

        line_of_sight(maze, self.x, self.y, px, py)
    }
}

/// Recorrido barato de celdas entre dos puntos: hay vista si ninguna celda
/// intermedia es solida. Suficiente para el tamano de estos mapas.
pub fn line_of_sight(maze: &Maze, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 1.0e-4 {
        return true;
    }
    // Un paso por cada media celda: no se puede colar por una pared de 1 celda.
    let steps = (dist * 2.0).ceil() as i32;
    let sx = dx / steps as f32;
    let sy = dy / steps as f32;

    let mut x = x0;
    let mut y = y0;
    for _ in 0..steps {
        x += sx;
        y += sy;
        if maze.is_solid(x.floor() as i32, y.floor() as i32) {
            return false;
        }
    }
    true
}
