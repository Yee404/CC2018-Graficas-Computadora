// Raycasting clasico por DDA sobre la cuadricula.
use crate::maze::Maze;
use crate::{FOV, RAY_COUNT};

#[derive(Clone, Copy)]
pub struct Hit {
    pub dist: f32, // distancia perpendicular (sin efecto ojo de pez)
    pub tile: u8,  // caracter de la pared golpeada
    pub side: u8,  // 0 = cara vertical (eje X), 1 = cara horizontal (eje Y)
}

impl Hit {
    pub const FAR: Hit = Hit {
        dist: 1.0e6,
        tile: b'#',
        side: 0,
    };
}

/// Lanza RAY_COUNT rayos y escribe el resultado en `out` (buffer reutilizado,
/// nunca se asigna memoria dentro del game loop).
pub fn cast_all(maze: &Maze, px: f32, py: f32, angle: f32, out: &mut [Hit]) {
    let half_fov = FOV * 0.5;
    let dir_x = angle.cos();
    let dir_y = angle.sin();
    // Plano de camara perpendicular a la direccion: da correccion de ojo de
    // pez "gratis" porque medimos la distancia perpendicular.
    let plane_len = half_fov.tan();
    let plane_x = -dir_y * plane_len;
    let plane_y = dir_x * plane_len;

    let n = out.len().min(RAY_COUNT);
    for i in 0..n {
        let camera = 2.0 * (i as f32 + 0.5) / n as f32 - 1.0;
        let rdx = dir_x + plane_x * camera;
        let rdy = dir_y + plane_y * camera;
        out[i] = cast_ray(maze, px, py, rdx, rdy);
    }
}

pub fn cast_ray(maze: &Maze, px: f32, py: f32, rdx: f32, rdy: f32) -> Hit {
    let mut map_x = px.floor() as i32;
    let mut map_y = py.floor() as i32;

    // Proteccion contra division por cero: rayo paralelo a un eje.
    let delta_x = if rdx.abs() < 1.0e-6 {
        f32::MAX
    } else {
        (1.0 / rdx).abs()
    };
    let delta_y = if rdy.abs() < 1.0e-6 {
        f32::MAX
    } else {
        (1.0 / rdy).abs()
    };

    let (step_x, mut side_dist_x) = if rdx < 0.0 {
        (-1, (px - map_x as f32) * delta_x)
    } else {
        (1, (map_x as f32 + 1.0 - px) * delta_x)
    };
    let (step_y, mut side_dist_y) = if rdy < 0.0 {
        (-1, (py - map_y as f32) * delta_y)
    } else {
        (1, (map_y as f32 + 1.0 - py) * delta_y)
    };

    let mut side;
    // Cota dura de pasos: evita cualquier posibilidad de bucle infinito.
    let max_steps = (maze.width + maze.height) * 2 + 8;

    for _ in 0..max_steps {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_x;
            map_x += step_x;
            side = 0;
        } else {
            side_dist_y += delta_y;
            map_y += step_y;
            side = 1;
        }

        // Fuera del mapa cuenta como pared (el perimetro deberia cerrarlo,
        // pero esto lo garantiza aunque el nivel este mal hecho).
        if maze.is_solid(map_x, map_y) {
            let dist = if side == 0 {
                side_dist_x - delta_x
            } else {
                side_dist_y - delta_y
            };
            if !dist.is_finite() || dist.is_nan() {
                return Hit::FAR;
            }
            return Hit {
                dist: dist.max(0.0001), // nunca cero: evita divisiones invalidas
                tile: maze.tile(map_x, map_y),
                side,
            };
        }
    }

    Hit::FAR
}
