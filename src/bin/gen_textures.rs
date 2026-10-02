// gen_textures.rs
// Herramienta auxiliar (binario independiente, sin dependencias nuevas)
// que genera las texturas propias del proyecto como archivos PPM (P3, ASCII)
// en `assets/`. Son diseños pixel-art de 16 px por bloque con paletas
// discretas (pocos colores fijos), inspirados en el estilo de Minecraft
// pero dibujados aquí; no se copia ningún recurso oficial.
//
// Algunas texturas son atlas verticales de 16 × 48 (tapa / lado / fondo),
// usados con `Cube::top_side_bottom`.
//
// Uso local (la estudiante lo ejecuta, no Claude):
//   cargo run --bin gen_textures

use std::fs;
use std::io::Write;

const SIZE: usize = 16;

type Rgb = (u8, u8, u8);

/// Hash entero determinista; solo se usa para escoger entre colores de una
/// paleta fija, nunca para generar tonos continuos.
fn hash(x: i32, y: i32, seed: i32) -> f32 {
    let mut n = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))
        .wrapping_add(seed.wrapping_mul(362_437));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    n ^= n >> 16;
    (n as u32) as f32 / u32::MAX as f32
}

fn pick(palette: &[Rgb], x: usize, y: usize, seed: i32) -> Rgb {
    let h = hash(x as i32, y as i32, seed);
    palette[((h * palette.len() as f32) as usize).min(palette.len() - 1)]
}

fn write_ppm(path: &str, w: usize, h: usize, pixel_fn: fn(usize, usize) -> Rgb) {
    let mut out = String::with_capacity(w * h * 12 + 32);
    out.push_str(&format!("P3\n{} {}\n255\n", w, h));
    for y in 0..h {
        for x in 0..w {
            let (r, g, b) = pixel_fn(x, y);
            out.push_str(&format!("{} {} {}\n", r, g, b));
        }
    }
    let mut file =
        fs::File::create(path).unwrap_or_else(|e| panic!("No se pudo crear '{}': {}", path, e));
    file.write_all(out.as_bytes()).unwrap();
    println!("Generado {}", path);
}

// ---------------------------------------------------------------------------
// Ladrillos de piedra
// ---------------------------------------------------------------------------
const STONE_MORTAR: Rgb = (74, 74, 77);
const STONE_SHADOW: Rgb = (100, 100, 103);
const STONE_LIGHT: Rgb = (150, 150, 153);
const STONE_MID: [Rgb; 3] = [(118, 118, 121), (126, 126, 129), (134, 134, 137)];

// Dos hileras de ladrillos de 16 × 8 px, la segunda desplazada medio ladrillo.
// Cada ladrillo: borde superior/izquierdo claro, inferior/derecho oscuro y
// junta de mortero.
fn stronghold_bricks_pixel(x: usize, y: usize) -> Rgb {
    let row = y / 8;
    let shifted = if row % 2 == 0 { x } else { (x + 8) % 16 };
    let lx = shifted % 16;
    let ly = y % 8;

    if ly == 7 || lx == 15 {
        STONE_MORTAR
    } else if ly == 0 || lx == 0 {
        STONE_LIGHT
    } else if ly == 6 || lx == 14 {
        STONE_SHADOW
    } else {
        pick(&STONE_MID, x, y, 11)
    }
}

const MOSS: [Rgb; 3] = [(66, 92, 40), (82, 112, 48), (98, 128, 56)];

// Mismo patrón de ladrillos; el musgo crece sobre las juntas y la parte
// baja de cada ladrillo, con algunos píxeles sueltos.
fn mossy_bricks_pixel(x: usize, y: usize) -> Rgb {
    let base = stronghold_bricks_pixel(x, y);
    let ly = y % 8;
    let h = hash(x as i32, y as i32, 13);
    let near_joint = ly >= 5 || base == STONE_MORTAR;
    if (near_joint && h > 0.55) || h > 0.93 {
        pick(&MOSS, x, y, 14)
    } else {
        base
    }
}

// ---------------------------------------------------------------------------
// Césped y tierra
// ---------------------------------------------------------------------------
const GRASS: [Rgb; 4] = [(86, 138, 48), (98, 152, 54), (110, 166, 62), (76, 124, 42)];
const DIRT: [Rgb; 4] = [(118, 84, 58), (132, 94, 66), (102, 72, 50), (146, 106, 74)];
const DARK_DIRT: [Rgb; 4] = [(92, 64, 44), (104, 73, 50), (80, 56, 39), (116, 82, 56)];

// Atlas 16 × 48: tapa verde / lado con franja de césped irregular sobre
// tierra / fondo de tierra.
fn grass_pixel(x: usize, y: usize) -> Rgb {
    let ly = y % SIZE;
    match y / SIZE {
        0 => pick(&GRASS, x, y, 101),
        1 => {
            let drip = 3 + (hash(x as i32, 0, 102) * 3.0) as usize;
            if ly < drip {
                pick(&GRASS, x, y, 103)
            } else {
                pick(&DIRT, x, y, 104)
            }
        }
        _ => pick(&DIRT, x, y, 105),
    }
}

fn dirt_pixel(x: usize, y: usize) -> Rgb {
    pick(&DARK_DIRT, x, y, 111)
}

// ---------------------------------------------------------------------------
// End Portal Frame (atlas 16 × 48)
// ---------------------------------------------------------------------------
const FRAME_DARK_TEAL: Rgb = (36, 78, 70);
const FRAME_TEAL: Rgb = (58, 110, 92);
const FRAME_LIGHT_GREEN: Rgb = (122, 172, 112);
const FRAME_CREAM: [Rgb; 3] = [(222, 220, 170), (210, 208, 158), (232, 230, 184)];

fn end_portal_frame_pixel(x: usize, y: usize) -> Rgb {
    let ly = y % SIZE;
    let edge = x == 0 || ly == 0 || x == SIZE - 1 || ly == SIZE - 1;

    match y / SIZE {
        // Tapa: simétrica (se ve igual desde el centro del portal en los 4
        // lados), con anillo verde claro y hueco central verde azulado.
        0 => {
            let ring = x.min(ly).min(SIZE - 1 - x).min(SIZE - 1 - ly);
            if edge {
                FRAME_DARK_TEAL
            } else if ring == 1 {
                FRAME_TEAL
            } else if ring == 2 {
                FRAME_LIGHT_GREEN
            } else if ring >= 4 {
                FRAME_DARK_TEAL
            } else {
                pick(&FRAME_CREAM, x, y, 121)
            }
        }
        // Lado: crema con una franja verde azulada. El bloque mide 13/16,
        // así que las tres filas superiores no se ven; la franja empieza
        // en la fila 3.
        1 => {
            if x == 0 || x == SIZE - 1 || ly == SIZE - 1 {
                FRAME_DARK_TEAL
            } else if (3..=5).contains(&ly) {
                if (x + ly) % 3 == 0 {
                    FRAME_LIGHT_GREEN
                } else {
                    FRAME_TEAL
                }
            } else if ly == 6 {
                FRAME_DARK_TEAL
            } else {
                pick(&FRAME_CREAM, x, y, 122)
            }
        }
        _ => {
            if edge {
                FRAME_DARK_TEAL
            } else {
                pick(&FRAME_CREAM, x, y, 123)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Eye of Ender: anillos verde oscuro / verde lima / aqua y centro negro
// ---------------------------------------------------------------------------
fn eye_of_ender_pixel(x: usize, y: usize) -> Rgb {
    let dx = x as f32 - 7.5;
    let dy = y as f32 - 7.5;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 2.0 {
        (10, 10, 12)
    } else if dist < 3.5 {
        (40, 190, 170) // aqua
    } else if dist < 5.5 {
        (130, 210, 60) // verde lima
    } else if dist < 7.0 {
        (24, 84, 52) // verde oscuro
    } else {
        (16, 56, 36)
    }
}

// ---------------------------------------------------------------------------
// Rejas de hierro: columnas con brillo y sombra, bandas horizontales
// ---------------------------------------------------------------------------
fn iron_bars_pixel(x: usize, y: usize) -> Rgb {
    match x % 4 {
        0 => (60, 62, 66),
        1 => (196, 198, 204),
        _ => {
            if y % 8 == 0 {
                (110, 112, 118)
            } else {
                (150, 152, 158)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The End
// ---------------------------------------------------------------------------
const END_STONE: [Rgb; 4] = [
    (219, 222, 158),
    (228, 230, 170),
    (208, 210, 148),
    (235, 236, 182),
];

fn end_stone_pixel(x: usize, y: usize) -> Rgb {
    let h = hash(x as i32, y as i32, 41);
    if h < 0.07 {
        (186, 184, 126) // hoyuelo
    } else if h < 0.2 {
        (198, 198, 138)
    } else {
        pick(&END_STONE, x, y, 42)
    }
}

const OBSIDIAN_BLACK: [Rgb; 3] = [(14, 10, 20), (20, 15, 30), (26, 20, 38)];

// Negra con vetas diagonales y destellos morado oscuro.
fn obsidian_pixel(x: usize, y: usize) -> Rgb {
    let h = hash(x as i32, y as i32, 51);
    let streak = (x + 2 * y) % 7 == 0;
    if h > 0.96 {
        (86, 56, 122)
    } else if streak && h > 0.35 {
        (58, 34, 88)
    } else {
        pick(&OBSIDIAN_BLACK, x, y, 52)
    }
}

const BEDROCK: [Rgb; 4] = [(18, 18, 18), (48, 48, 48), (84, 84, 84), (122, 122, 122)];

// Gris y negro en grupos de 2 × 1 píxeles.
fn bedrock_pixel(x: usize, y: usize) -> Rgb {
    pick(&BEDROCK, x / 2, y, 81)
}

// Superficie del portal: casi negra con puntos aqua y azules.
fn end_portal_pixel(x: usize, y: usize) -> Rgb {
    let star = hash(x as i32, y as i32, 72);
    if star > 0.92 {
        (70, 210, 200)
    } else if star > 0.86 {
        (35, 100, 140)
    } else {
        pick(&[(6, 10, 20), (10, 14, 26)], x, y, 71)
    }
}

// Dragón: escamas oscuras con líneas diagonales levemente más claras.
fn dragon_pixel(x: usize, y: usize) -> Rgb {
    if (x + y) % 4 == 0 {
        (42, 36, 52)
    } else {
        pick(&[(22, 18, 28), (28, 24, 34)], x, y, 91)
    }
}

fn main() {
    fs::create_dir_all("assets").expect("No se pudo crear la carpeta 'assets'");

    write_ppm(
        "assets/stronghold_bricks.ppm",
        SIZE,
        SIZE,
        stronghold_bricks_pixel,
    );
    write_ppm("assets/mossy_bricks.ppm", SIZE, SIZE, mossy_bricks_pixel);
    write_ppm("assets/grass.ppm", SIZE, SIZE * 3, grass_pixel);
    write_ppm("assets/dirt.ppm", SIZE, SIZE, dirt_pixel);
    write_ppm(
        "assets/end_portal_frame.ppm",
        SIZE,
        SIZE * 3,
        end_portal_frame_pixel,
    );
    write_ppm("assets/eye_of_ender.ppm", SIZE, SIZE, eye_of_ender_pixel);
    write_ppm("assets/iron_bars.ppm", SIZE, SIZE, iron_bars_pixel);
    write_ppm("assets/end_stone.ppm", SIZE, SIZE, end_stone_pixel);
    write_ppm("assets/obsidian.ppm", SIZE, SIZE, obsidian_pixel);
    write_ppm("assets/bedrock.ppm", SIZE, SIZE, bedrock_pixel);
    write_ppm("assets/end_portal.ppm", SIZE, SIZE, end_portal_pixel);
    write_ppm("assets/dragon.ppm", SIZE, SIZE, dragon_pixel);

    println!("Listo. Ahora puedes ejecutar: cargo run --release");
}
