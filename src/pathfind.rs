// Busqueda de rutas sobre la cuadricula del mapa: un BFS barato compartido por
// los enemigos simples (nivel 3) y por el monstruo (niveles 4-5).
use crate::maze::Maze;

const UNREACHED: u16 = u16::MAX;

/// Campo de distancias BFS hacia una celda objetivo sobre la cuadricula.
/// Bajar por el gradiente lleva al objetivo. Los buffers se reutilizan y el
/// BFS solo se recalcula por intervalo o cuando el objetivo cambia de celda.
pub struct FlowField {
    dist: Vec<u16>,
    queue: Vec<u32>, // cola BFS reutilizada (indices de celda)
    width: usize,
    height: usize,
    src: (i32, i32),
    timer: f32,
    interval: f32,
}

impl FlowField {
    pub fn new(maze: &Maze, interval: f32) -> FlowField {
        FlowField {
            dist: vec![UNREACHED; maze.width * maze.height],
            queue: Vec::with_capacity(maze.width * maze.height),
            width: maze.width,
            height: maze.height,
            src: (i32::MIN, i32::MIN),
            timer: 0.0,
            interval,
        }
    }

    /// Recalcula solo si toca por tiempo o si el objetivo cambio de celda.
    pub fn update(&mut self, maze: &Maze, tx: f32, ty: f32, dt: f32) {
        self.timer -= dt;
        let cell = (tx.floor() as i32, ty.floor() as i32);
        if self.timer > 0.0 && cell == self.src {
            return;
        }
        self.timer = self.interval;
        self.src = cell;
        self.rebuild(maze, cell);
    }

    fn rebuild(&mut self, maze: &Maze, src: (i32, i32)) {
        for d in self.dist.iter_mut() {
            *d = UNREACHED;
        }
        self.queue.clear();

        if maze.is_solid(src.0, src.1) {
            return;
        }
        let start = src.1 as usize * self.width + src.0 as usize;
        self.dist[start] = 0;
        self.queue.push(start as u32);

        let mut head = 0;
        while head < self.queue.len() {
            let idx = self.queue[head] as usize;
            head += 1;
            let d = self.dist[idx];
            if d == UNREACHED - 1 {
                continue;
            }
            let x = (idx % self.width) as i32;
            let y = (idx / self.width) as i32;

            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx as usize >= self.width || ny as usize >= self.height {
                    continue;
                }
                if maze.is_solid(nx, ny) {
                    continue;
                }
                let nidx = ny as usize * self.width + nx as usize;
                if self.dist[nidx] == UNREACHED {
                    self.dist[nidx] = d + 1;
                    self.queue.push(nidx as u32);
                }
            }
        }
    }

    /// Celda vecina con menor distancia al objetivo, si existe alguna mejor.
    pub fn next_cell(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        let here = self.at(x, y)?;
        if here == 0 {
            return None; // ya estamos en la celda objetivo
        }
        let mut best = here;
        let mut best_cell = None;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if let Some(d) = self.at(x + dx, y + dy) {
                if d < best {
                    best = d;
                    best_cell = Some((x + dx, y + dy));
                }
            }
        }
        best_cell
    }

    #[inline]
    fn at(&self, x: i32, y: i32) -> Option<u16> {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return None;
        }
        let d = self.dist[y as usize * self.width + x as usize];
        if d == UNREACHED {
            None
        } else {
            Some(d)
        }
    }
}
