// Enemigo basico del nivel 3: persigue al jugador siguiendo un campo de
// distancias BFS que se recalcula pocas veces por segundo (no cada frame).
// La IA es deliberadamente simple; sustituirla despues (nivel 4/5) solo
// requiere cambiar `Enemy::update` y/o el campo de distancias.
use crate::maze::Maze;
use crate::pathfind::FlowField;

pub const ENEMY_SPEED: f32 = 1.6; // celdas por segundo (el jugador va a 3.0)
pub const ENEMY_RADIUS: f32 = 0.25;
/// Radio de deteccion: mas alla de esto el enemigo se queda quieto.
pub const DETECT_RADIUS: f32 = 7.0;
/// Radio de dano por contacto/proximidad.
pub const ATTACK_RADIUS: f32 = 0.75;
pub const DAMAGE_PER_SECOND: f32 = 15.0;
/// Cada cuanto se recalcula el BFS (tambien se recalcula si el jugador
/// cambia de celda).
pub const REPATH_INTERVAL: f32 = 0.3;

pub struct Enemy {
    pub x: f32,
    pub y: f32,
}

impl Enemy {
    pub fn new(x: f32, y: f32) -> Enemy {
        Enemy { x, y }
    }

    #[inline]
    pub fn dist2_to(&self, px: f32, py: f32) -> f32 {
        let dx = self.x - px;
        let dy = self.y - py;
        dx * dx + dy * dy
    }

    pub fn update(&mut self, maze: &Maze, flow: &FlowField, px: f32, py: f32, dt: f32) {
        let dist2 = self.dist2_to(px, py);
        if dist2 > DETECT_RADIUS * DETECT_RADIUS {
            return; // fuera de rango: se queda quieto
        }

        let cx = self.x.floor() as i32;
        let cy = self.y.floor() as i32;

        // Objetivo: el centro de la celda vecina mas cercana al jugador segun
        // el BFS. Si no hay ruta (o ya estamos en la celda del jugador), se va
        // directo hacia el, que es lo correcto en linea recta.
        let (tx, ty) = match flow.next_cell(cx, cy) {
            Some((nx, ny)) => (nx as f32 + 0.5, ny as f32 + 0.5),
            None => (px, py),
        };

        let dx = tx - self.x;
        let dy = ty - self.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1.0e-4 {
            return;
        }
        let step = ENEMY_SPEED * dt;
        let mx = dx / len * step;
        let my = dy / len * step;

        // Ejes por separado: si uno choca, el enemigo se desliza por la pared
        // en lugar de quedarse clavado.
        if !maze.circle_hits_wall(self.x + mx, self.y, ENEMY_RADIUS) {
            self.x += mx;
        }
        if !maze.circle_hits_wall(self.x, self.y + my, ENEMY_RADIUS) {
            self.y += my;
        }
    }
}
