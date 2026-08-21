// Entidades del mundo: datos simples, sin ECS ni jerarquias.
// La logica (posicion/estado) vive aqui; la proyeccion esta en renderer.rs y
// la representacion visual provisional es geometrica (todavia no hay sprites).

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Coin,
    /// Objeto recolectable del nivel 3 (se entrega en la zona de deposito).
    Item,
    /// Zona de deposito: entidad fija, nunca se recoge.
    Deposit,
    /// Escondite de los niveles 4-5: entidad fija, se usa con la tecla E.
    Hideout,
}

#[derive(Clone, Copy)]
pub struct Entity {
    pub x: f32,
    pub y: f32,
    pub kind: EntityKind,
    pub active: bool,
}

impl Entity {
    pub fn new(x: f32, y: f32, kind: EntityKind) -> Entity {
        Entity {
            x,
            y,
            kind,
            active: true,
        }
    }

    /// Distancia al cuadrado hasta un punto (evita la raiz cuadrada).
    #[inline]
    pub fn dist2_to(&self, px: f32, py: f32) -> f32 {
        let dx = self.x - px;
        let dy = self.y - py;
        dx * dx + dy * dy
    }
}
