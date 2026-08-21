// Carga y consulta del mapa en cuadricula.
use std::fs;

pub const TILE_FLOOR: u8 = b'.';
pub const TILE_GOAL: u8 = b'G';
/// Marcador de moneda en el archivo de nivel: la celda queda transitable y
/// se convierte en una entidad al cargar.
pub const TILE_COIN_MARKER: u8 = b'O';
/// Marcadores del nivel 3: objeto, zona de deposito y spawn de enemigo.
/// Los tres dejan la celda transitable.
pub const TILE_ITEM_MARKER: u8 = b'I';
pub const TILE_DEPOSIT_MARKER: u8 = b'D';
pub const TILE_ENEMY_MARKER: u8 = b'E';
/// Marcadores de los niveles 4-5: monstruo y escondite.
pub const TILE_HUNTER_MARKER: u8 = b'M';
pub const TILE_HIDEOUT_MARKER: u8 = b'H';

pub struct Maze {
    tiles: Vec<u8>, // fila por fila, width * height
    pub width: usize,
    pub height: usize,
    pub start_x: f32,
    pub start_y: f32,
    pub goal_x: usize,
    pub goal_y: usize,
    /// Celdas donde el nivel pide crear una moneda (centro de la celda).
    pub coin_spawns: Vec<(usize, usize)>,
    pub item_spawns: Vec<(usize, usize)>,
    pub enemy_spawns: Vec<(usize, usize)>,
    /// Zona de deposito (si el mapa la define).
    pub deposit: Option<(usize, usize)>,
    /// Spawn del monstruo de los niveles 4-5.
    pub hunter_spawn: Option<(usize, usize)>,
    pub hideouts: Vec<(usize, usize)>,
}

impl Maze {
    pub fn load(path: &str) -> Result<Maze, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;

        let lines: Vec<&str> = text
            .lines()
            .map(|l| l.trim_end_matches('\r'))
            .filter(|l| !l.is_empty())
            .collect();

        if lines.is_empty() {
            return Err("el mapa esta vacio".to_string());
        }

        let width = lines.iter().map(|l| l.len()).max().unwrap_or(0);
        let height = lines.len();

        let mut tiles = vec![b'#'; width * height];
        let mut start = None;
        let mut goal = None;
        let mut coin_spawns = Vec::new();
        let mut item_spawns = Vec::new();
        let mut enemy_spawns = Vec::new();
        let mut deposit = None;
        let mut hunter_spawn = None;
        let mut hideouts = Vec::new();

        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.bytes().enumerate() {
                let tile = match ch {
                    b'P' => {
                        start = Some((x, y));
                        TILE_FLOOR
                    }
                    b'G' => {
                        goal = Some((x, y));
                        TILE_GOAL
                    }
                    TILE_COIN_MARKER => {
                        coin_spawns.push((x, y));
                        TILE_FLOOR
                    }
                    TILE_ITEM_MARKER => {
                        item_spawns.push((x, y));
                        TILE_FLOOR
                    }
                    TILE_ENEMY_MARKER => {
                        enemy_spawns.push((x, y));
                        TILE_FLOOR
                    }
                    TILE_HUNTER_MARKER => {
                        hunter_spawn = Some((x, y));
                        TILE_FLOOR
                    }
                    TILE_HIDEOUT_MARKER => {
                        hideouts.push((x, y));
                        TILE_FLOOR
                    }
                    TILE_DEPOSIT_MARKER => {
                        deposit = Some((x, y));
                        TILE_FLOOR
                    }
                    b' ' => TILE_FLOOR,
                    other => other,
                };
                tiles[y * width + x] = tile;
            }
        }

        let (sx, sy) = start.ok_or("no se encontro el jugador 'P'")?;
        let (gx, gy) = goal.ok_or("no se encontro la meta 'G'")?;

        Ok(Maze {
            tiles,
            width,
            height,
            start_x: sx as f32 + 0.5,
            start_y: sy as f32 + 0.5,
            goal_x: gx,
            goal_y: gy,
            coin_spawns,
            item_spawns,
            enemy_spawns,
            deposit,
            hunter_spawn,
            hideouts,
        })
    }

    /// Celda del mapa. Fuera de los limites se considera pared solida.
    #[inline]
    pub fn tile(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return b'#';
        }
        self.tiles[y as usize * self.width + x as usize]
    }

    #[inline]
    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        let t = self.tile(x, y);
        t != TILE_FLOOR && t != TILE_GOAL
    }

    /// Colision de un circulo contra las celdas solidas. Se prueban las celdas
    /// que toca su caja envolvente: barato y correcto tambien en esquinas.
    /// La usan el jugador y los enemigos.
    pub fn circle_hits_wall(&self, x: f32, y: f32, radius: f32) -> bool {
        let min_x = (x - radius).floor() as i32;
        let max_x = (x + radius).floor() as i32;
        let min_y = (y - radius).floor() as i32;
        let max_y = (y + radius).floor() as i32;

        for cy in min_y..=max_y {
            for cx in min_x..=max_x {
                if self.is_solid(cx, cy) {
                    return true;
                }
            }
        }
        false
    }
}
