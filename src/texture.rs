// texture.rs
// Cargador mínimo de texturas en formato PPM (P3, ASCII), usando solo la
// biblioteca estándar. No depende de la crate `image` ni de ninguna otra
// crate externa.

use std::fs;
use std::rc::Rc;

use crate::color::Color;

#[derive(Debug)]
pub struct Texture {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl Texture {
    /// Carga una textura PPM en formato P3 (ASCII) desde disco.
    /// Los comentarios `#` dentro del archivo son ignorados.
    pub fn from_ppm(path: &str) -> Rc<Self> {
        let contents = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("No se pudo leer la textura '{}': {}", path, e));

        // Elimina comentarios línea por línea antes de tokenizar.
        let cleaned: String = contents
            .lines()
            .map(|line| match line.find('#') {
                Some(idx) => &line[..idx],
                None => line,
            })
            .collect::<Vec<_>>()
            .join(" ");

        let mut tokens = cleaned.split_whitespace();

        let magic = tokens
            .next()
            .unwrap_or_else(|| panic!("Textura '{}' vacía o inválida", path));
        assert_eq!(
            magic, "P3",
            "Textura '{}': solo se soporta PPM P3 (ASCII)",
            path
        );

        let width: usize = tokens
            .next()
            .and_then(|t| t.parse().ok())
            .unwrap_or_else(|| panic!("Textura '{}': ancho inválido", path));
        let height: usize = tokens
            .next()
            .and_then(|t| t.parse().ok())
            .unwrap_or_else(|| panic!("Textura '{}': alto inválido", path));
        let maxval: f32 = tokens
            .next()
            .and_then(|t| t.parse().ok())
            .unwrap_or_else(|| panic!("Textura '{}': valor máximo inválido", path));
        let scale = 255.0 / maxval.max(1.0);

        let mut pixels = Vec::with_capacity(width * height);
        while let (Some(r), Some(g), Some(b)) = (tokens.next(), tokens.next(), tokens.next()) {
            let r: f32 = r.parse().unwrap_or(0.0);
            let g: f32 = g.parse().unwrap_or(0.0);
            let b: f32 = b.parse().unwrap_or(0.0);
            pixels.push(Color::new(
                (r * scale).clamp(0.0, 255.0) as u8,
                (g * scale).clamp(0.0, 255.0) as u8,
                (b * scale).clamp(0.0, 255.0) as u8,
            ));
        }

        assert_eq!(
            pixels.len(),
            width * height,
            "Textura '{}': la cantidad de píxeles no coincide con el encabezado",
            path
        );

        Rc::new(Texture {
            width,
            height,
            pixels,
        })
    }

    /// Muestrea la textura con coordenadas UV en [0, 1], repitiendo
    /// (wrap) cuando el valor cae fuera de rango.
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let u = u.rem_euclid(1.0);
        let v = v.rem_euclid(1.0);

        let x = ((u * self.width as f32) as usize).min(self.width - 1);
        // v=0 corresponde a la parte inferior de la cara; la primera fila
        // del PPM es la superior, por eso se invierte aquí.
        let y = (((1.0 - v) * self.height as f32) as usize).min(self.height - 1);

        self.pixels[y * self.width + x]
    }
}
