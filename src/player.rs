// Jugador: estado, movimiento con delta time y colisiones contra la cuadricula.
use crate::game::Input;
use crate::maze::Maze;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub angle: f32, // radianes
    pub move_speed: f32,
    pub rotation_speed: f32,
    pub radius: f32,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Player {
        Player {
            x,
            y,
            angle: 0.0,
            move_speed: 3.0,     // celdas por segundo
            rotation_speed: 2.4, // radianes por segundo
            radius: 0.22,
        }
    }

    pub fn update(&mut self, maze: &Maze, input: &Input, dt: f32) {
        // Rotacion: teclado (A/D) y mouse funcionan a la vez.
        self.angle += input.turn * self.rotation_speed * dt;
        self.angle += input.mouse_dx;
        self.angle = wrap_angle(self.angle);

        let (sin, cos) = self.angle.sin_cos();
        let mut dx = 0.0;
        let mut dy = 0.0;

        if input.forward != 0.0 {
            let step = input.forward * self.move_speed * dt;
            dx += cos * step;
            dy += sin * step;
        }
        if input.strafe != 0.0 {
            // Perpendicular a la mirada; (-sin, cos) apunta a la derecha.
            let step = input.strafe * self.move_speed * dt;
            dx += -sin * step;
            dy += cos * step;
        }

        // Ejes por separado: permite deslizarse a lo largo de las paredes.
        if dx != 0.0 && !self.collides(maze, self.x + dx, self.y) {
            self.x += dx;
        }
        if dy != 0.0 && !self.collides(maze, self.x, self.y + dy) {
            self.y += dy;
        }
    }

    #[inline]
    fn collides(&self, maze: &Maze, x: f32, y: f32) -> bool {
        maze.circle_hits_wall(x, y, self.radius)
    }
}

#[inline]
fn wrap_angle(a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut a = a % tau;
    if a < 0.0 {
        a += tau;
    }
    a
}
