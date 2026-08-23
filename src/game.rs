// Estado del juego y separacion INPUT / UPDATE / RENDER.
use raylib::prelude::*;

use crate::assets::Assets;
use crate::enemy::{self, Enemy, REPATH_INTERVAL};
use crate::entities::{Entity, EntityKind};
use crate::hunter::{self, Hunter};
use crate::levels::{LevelDefinition, LevelObjective};
use crate::maze::{Maze, TILE_GOAL};
use crate::pathfind::FlowField;
use crate::player::Player;
use crate::raycaster::{self, Hit};
use crate::{minimap, renderer, MOUSE_SENSITIVITY, RAY_COUNT};

/// Entrada ya traducida a intenciones, para poder cambiar de dispositivo
/// (teclado, gamepad, controles de una consola portatil) sin tocar la logica.
pub struct Input {
    pub forward: f32, // -1 atras, +1 adelante
    pub turn: f32,    // -1 izquierda, +1 derecha
    pub strafe: f32,  // -1 izquierda, +1 derecha (sin girar)
    pub mouse_dx: f32,
    /// Tecla E: entrar o salir de un escondite.
    pub interact: bool,
}

pub fn read_input(rl: &RaylibHandle) -> Input {
    let mut forward = 0.0;
    let mut turn = 0.0;
    let mut strafe = 0.0;

    if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
        forward += 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN) {
        forward -= 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_A) {
        turn -= 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        turn += 1.0;
    }
    // Las flechas laterales hacen strafe, ya no rotan.
    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        strafe -= 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        strafe += 1.0;
    }

    Input {
        forward,
        turn,
        strafe,
        mouse_dx: rl.get_mouse_delta().x * MOUSE_SENSITIVITY,
        interact: rl.is_key_pressed(KeyboardKey::KEY_E),
    }
}

/// Distancia (al cuadrado) a la que se recoge una entidad automaticamente.
const PICKUP_DIST2: f32 = 0.35 * 0.35;
/// Distancia (al cuadrado) a la que se entrega en la zona de deposito.
const DEPOSIT_DIST2: f32 = 0.9 * 0.9;
/// Distancia (al cuadrado) a la que se puede usar un escondite.
const HIDEOUT_DIST2: f32 = 0.8 * 0.8;
const MAX_HEALTH: f32 = 100.0;

pub struct Game {
    level: &'static LevelDefinition,
    maze: Maze,
    player: Player,
    entities: Vec<Entity>,
    enemies: Vec<Enemy>,
    hunter: Option<Hunter>,
    /// Campo BFS hacia el jugador, compartido por los enemigos simples.
    flow: FlowField,
    hits: Vec<Hit>, // buffer reutilizado en cada frame
    /// Orden de dibujo (lejos -> cerca), reutilizado. Los indices menores que
    /// `entities.len()` son entidades; el resto, enemigos y monstruo.
    order: Vec<(f32, usize)>,
    /// Indice (en `entities`) de cada escondite del nivel.
    hideouts: Vec<usize>,
    /// Escondite ocupado ahora mismo, si el jugador esta escondido.
    hidden_in: Option<usize>,
    /// Escondite alcanzable con la tecla E en este momento.
    hideout_near: Option<usize>,
    coins_total: u32,
    coins_collected: u32,
    items_total: u32,
    items_carried: u32,
    items_deposited: bool,
    health: f32,
    damage_flash: f32,
    /// Tiempo acumulado con delta time; de aqui sale el frame de animacion.
    anim_time: f32,
    level_complete: bool,
    at_goal: bool,
    near_deposit: bool,
}

impl Game {
    pub fn new(level: &'static LevelDefinition) -> Result<Game, String> {
        let maze = Maze::load(level.map_path)?;
        let player = Player::new(maze.start_x, maze.start_y);

        let mut entities: Vec<Entity> = Vec::new();
        for &(x, y) in &maze.coin_spawns {
            entities.push(Entity::new(
                x as f32 + 0.5,
                y as f32 + 0.5,
                EntityKind::Coin,
            ));
        }
        for &(x, y) in &maze.item_spawns {
            entities.push(Entity::new(
                x as f32 + 0.5,
                y as f32 + 0.5,
                EntityKind::Item,
            ));
        }
        if let Some((x, y)) = maze.deposit {
            entities.push(Entity::new(
                x as f32 + 0.5,
                y as f32 + 0.5,
                EntityKind::Deposit,
            ));
        }
        let mut hideouts = Vec::with_capacity(maze.hideouts.len());
        for &(x, y) in &maze.hideouts {
            hideouts.push(entities.len());
            entities.push(Entity::new(
                x as f32 + 0.5,
                y as f32 + 0.5,
                EntityKind::Hideout,
            ));
        }

        let enemies: Vec<Enemy> = maze
            .enemy_spawns
            .iter()
            .map(|&(x, y)| Enemy::new(x as f32 + 0.5, y as f32 + 0.5))
            .collect();

        // El monstruo solo existe si el nivel lo configura y el mapa lo situa.
        let hunter = match (level.hunter, maze.hunter_spawn) {
            (Some(cfg), Some((x, y))) => {
                Some(Hunter::new(x as f32 + 0.5, y as f32 + 0.5, cfg, &maze))
            }
            _ => None,
        };

        let coins_total = maze.coin_spawns.len() as u32;
        let items_total = maze.item_spawns.len() as u32;
        let order = Vec::with_capacity(entities.len() + enemies.len() + 1);
        let flow = FlowField::new(&maze, REPATH_INTERVAL);

        Ok(Game {
            level,
            maze,
            player,
            entities,
            enemies,
            hunter,
            flow,
            hits: vec![Hit::FAR; RAY_COUNT],
            order,
            hideouts,
            hidden_in: None,
            hideout_near: None,
            coins_total,
            coins_collected: 0,
            items_total,
            items_carried: 0,
            items_deposited: false,
            health: MAX_HEALTH,
            damage_flash: 0.0,
            anim_time: 0.0,
            level_complete: false,
            at_goal: false,
            near_deposit: false,
        })
    }

    pub fn update(&mut self, input: &Input, dt: f32) {
        // Escondido: el jugador puede mirar pero no desplazarse.
        if self.hidden_in.is_some() {
            let look = Input {
                forward: 0.0,
                turn: input.turn,
                strafe: 0.0,
                mouse_dx: input.mouse_dx,
                interact: input.interact,
            };
            self.player.update(&self.maze, &look, dt);
        } else {
            self.player.update(&self.maze, input, dt);
            self.update_pickups();
        }

        self.update_hideouts(input);
        self.update_enemies(dt);
        self.update_hunter(dt);
        self.check_objective();

        self.damage_flash = (self.damage_flash - dt * 2.0).max(0.0);
        self.anim_time += dt;

        raycaster::cast_all(
            &self.maze,
            self.player.x,
            self.player.y,
            self.player.angle,
            &mut self.hits,
        );
        self.build_draw_order();
    }

    /// Recoleccion automatica por proximidad (monedas y objetos).
    fn update_pickups(&mut self) {
        for e in self.entities.iter_mut() {
            if !e.active || e.dist2_to(self.player.x, self.player.y) >= PICKUP_DIST2 {
                continue;
            }
            match e.kind {
                EntityKind::Coin => {
                    e.active = false;
                    self.coins_collected += 1;
                }
                EntityKind::Item => {
                    e.active = false;
                    self.items_carried += 1;
                }
                // Deposito y escondites no se recogen.
                EntityKind::Deposit | EntityKind::Hideout => {}
            }
        }
    }

    /// Entrar y salir de un escondite con la tecla E (interaccion explicita).
    fn update_hideouts(&mut self, input: &Input) {
        self.hideout_near = None;
        if self.hidden_in.is_some() {
            if input.interact {
                self.hidden_in = None;
            }
            return;
        }

        for &idx in &self.hideouts {
            let e = &self.entities[idx];
            if e.dist2_to(self.player.x, self.player.y) < HIDEOUT_DIST2 {
                self.hideout_near = Some(idx);
                break;
            }
        }

        if let (Some(idx), true) = (self.hideout_near, input.interact) {
            let e = self.entities[idx];
            self.player.x = e.x;
            self.player.y = e.y;
            self.hidden_in = Some(idx);

            // Nivel 5: si el monstruo lo estaba viendo, recuerda el escondite.
            if let Some(h) = self.hunter.as_mut() {
                if h.sees_player() {
                    h.saw_player_hide(idx, e.x, e.y);
                }
            }
        }
    }

    fn update_enemies(&mut self, dt: f32) {
        if self.enemies.is_empty() {
            return;
        }
        // Un solo campo BFS compartido, recalculado pocas veces por segundo.
        self.flow
            .update(&self.maze, self.player.x, self.player.y, dt);

        let mut damage = 0.0;
        for e in self.enemies.iter_mut() {
            e.update(&self.maze, &self.flow, self.player.x, self.player.y, dt);
            if e.dist2_to(self.player.x, self.player.y) < enemy::ATTACK_RADIUS.powi(2) {
                damage += enemy::DAMAGE_PER_SECOND * dt; // dano por segundo, no por frame
            }
        }
        self.apply_damage(damage);
    }

    fn update_hunter(&mut self, dt: f32) {
        let Some(h) = self.hunter.as_mut() else {
            return;
        };
        let hidden = self.hidden_in.is_some();
        h.update(
            &self.maze,
            &self.entities,
            self.player.x,
            self.player.y,
            hidden,
            dt,
        );

        // Nivel 5: llego a revisar el escondite donde vio entrar al jugador.
        if let Some(idx) = h.take_reached_hideout() {
            if self.hidden_in == Some(idx) {
                self.hidden_in = None;
                self.health = 0.0; // lo encuentra: muerte inmediata
                self.damage_flash = 1.0;
                return;
            }
        }

        if !hidden && h.dist2_to(self.player.x, self.player.y) < hunter::ATTACK_RADIUS.powi(2) {
            let damage = h.damage_per_second() * dt;
            self.apply_damage(damage);
        }
    }

    fn apply_damage(&mut self, damage: f32) {
        if damage > 0.0 && self.health > 0.0 {
            self.health = (self.health - damage).max(0.0);
            self.damage_flash = 1.0;
        }
    }

    /// Unico lugar donde se decide si el nivel esta ganado.
    fn check_objective(&mut self) {
        let cx = self.player.x.floor() as i32;
        let cy = self.player.y.floor() as i32;
        self.at_goal = self.maze.tile(cx, cy) == TILE_GOAL;
        self.near_deposit = false;

        match self.level.objective {
            LevelObjective::CollectCoinsAndExit => {
                if self.at_goal && self.coins_collected >= self.coins_total {
                    self.level_complete = true;
                }
            }
            LevelObjective::CollectItemsAndDeposit
            | LevelObjective::CollectItemsAndDepositWithHunter => {
                let Some((dx, dy)) = self.maze.deposit else {
                    return;
                };
                let ddx = dx as f32 + 0.5 - self.player.x;
                let ddy = dy as f32 + 0.5 - self.player.y;
                self.near_deposit = ddx * ddx + ddy * ddy < DEPOSIT_DIST2;

                if self.near_deposit && self.items_carried >= self.items_total {
                    self.items_deposited = true;
                    self.level_complete = true;
                }
            }
        }
    }

    fn build_draw_order(&mut self) {
        self.order.clear();
        for (i, e) in self.entities.iter().enumerate() {
            if e.active {
                self.order
                    .push((e.dist2_to(self.player.x, self.player.y), i));
            }
        }
        let offset = self.entities.len();
        for (i, e) in self.enemies.iter().enumerate() {
            self.order
                .push((e.dist2_to(self.player.x, self.player.y), offset + i));
        }
        if let Some(h) = self.hunter.as_ref() {
            self.order.push((
                h.dist2_to(self.player.x, self.player.y),
                offset + self.enemies.len(),
            ));
        }
        self.order
            .sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Condicion de victoria del nivel. Cambiarla despues no obliga a tocar la
    /// maquina de estados de `App`.
    pub fn is_complete(&self) -> bool {
        self.level_complete
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    /// Resumen para la pantalla de nivel completado.
    pub fn summary(&self) -> String {
        match self.level.objective {
            LevelObjective::CollectCoinsAndExit => {
                format!("COINS: {} / {}", self.coins_collected, self.coins_total)
            }
            _ => {
                let delivered = if self.items_deposited {
                    self.items_total
                } else {
                    0
                };
                format!("ITEMS DELIVERED: {} / {}", delivered, self.items_total)
            }
        }
    }

    /// Mensaje central segun la situacion (objetivo, deposito, escondite).
    fn message(&self) -> Option<&'static str> {
        if self.level_complete {
            return None;
        }
        if self.hidden_in.is_some() {
            return Some("HIDDEN");
        }
        if self.at_goal && self.level.objective == LevelObjective::CollectCoinsAndExit {
            return Some("COLLECT ALL COINS FIRST");
        }
        if self.near_deposit {
            return Some("FIND ALL ITEMS");
        }
        None
    }

    pub fn render(&self, d: &mut RaylibDrawHandle, fps: u32, assets: &Assets) {
        renderer::draw_world(d, &self.hits, self.level, assets);
        renderer::draw_billboards(
            d,
            &self.hits,
            &self.player,
            &self.entities,
            &self.enemies,
            self.hunter.as_ref(),
            &self.order,
            assets,
            self.anim_time,
        );
        minimap::draw(
            d,
            &self.maze,
            &self.player,
            &self.entities,
            &self.enemies,
            self.hunter.as_ref(),
            self.level.objective == LevelObjective::CollectCoinsAndExit,
        );

        // Orden: mundo -> billboards -> minimapa -> overlay -> HUD.
        // El overlay va antes del HUD para que HIDDEN y E - LEAVE se lean.
        if self.hidden_in.is_some() {
            renderer::draw_hideout_overlay(d, assets);
        }

        let hint = if self.hidden_in.is_some() {
            Some("E - LEAVE")
        } else if self.hideout_near.is_some() {
            Some("E - HIDE")
        } else {
            None
        };

        let hud = renderer::Hud {
            fps,
            objective: self.level.objective,
            coins: (self.coins_collected, self.coins_total),
            items: (self.items_carried, self.items_total),
            health: if self.enemies.is_empty() && self.hunter.is_none() {
                None
            } else {
                Some(self.health)
            },
            message: self.message(),
            hint,
            damage_flash: self.damage_flash,
        };
        renderer::draw_hud(d, &hud);
    }
}

// --- Pruebas logicas (sin ventana): reglas de objetivo y de escondites ------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hunter::HunterState;
    use crate::levels::LEVELS;

    const DT: f32 = 1.0 / 60.0;

    fn idle() -> Input {
        Input {
            forward: 0.0,
            turn: 0.0,
            strafe: 0.0,
            mouse_dx: 0.0,
            interact: false,
        }
    }

    fn press_e() -> Input {
        Input {
            interact: true,
            ..idle()
        }
    }

    fn game(level: usize) -> Game {
        Game::new(&LEVELS[level]).expect("el nivel debe cargar")
    }

    fn run(g: &mut Game, seconds: f32) {
        for _ in 0..(seconds / DT) as i32 {
            g.update(&idle(), DT);
        }
    }

    /// Celda libre junto a un punto, para colocar al monstruo en las pruebas.
    fn free_near(g: &Game, x: f32, y: f32) -> (f32, f32) {
        for (dx, dy) in [(2.0, 0.0), (-2.0, 0.0), (0.0, 2.0), (0.0, -2.0)] {
            if !g.maze.circle_hits_wall(x + dx, y + dy, 0.3) {
                return (x + dx, y + dy);
            }
        }
        (x, y)
    }

    fn hideout_pos(g: &Game) -> (usize, f32, f32) {
        let idx = g.hideouts[0];
        (idx, g.entities[idx].x, g.entities[idx].y)
    }

    /// Teletransporta al jugador encima de cada objeto para recogerlos.
    fn collect_all_items(g: &mut Game) {
        let spots: Vec<(f32, f32)> = g
            .entities
            .iter()
            .filter(|e| e.kind == EntityKind::Item)
            .map(|e| (e.x, e.y))
            .collect();
        for (x, y) in spots {
            g.player.x = x;
            g.player.y = y;
            g.update(&idle(), DT);
        }
    }

    #[test]
    fn deposito_solo_completa_con_todos_los_objetos() {
        let mut g = game(3); // Level 4
        let (dx, dy) = g.maze.deposit.expect("level4 tiene deposito");

        // Llegar al deposito sin objetos no completa el nivel.
        g.player.x = dx as f32 + 0.5;
        g.player.y = dy as f32 + 0.5;
        g.update(&idle(), DT);
        assert!(!g.is_complete());
        assert!(g.near_deposit);

        collect_all_items(&mut g);
        assert_eq!(g.items_carried, g.items_total);
        assert_eq!(g.items_total, 5);

        g.player.x = dx as f32 + 0.5;
        g.player.y = dy as f32 + 0.5;
        g.update(&idle(), DT);
        assert!(g.is_complete());
    }

    #[test]
    fn level4_esconderse_impide_deteccion_y_dano() {
        let mut g = game(3); // Level 4
        let (_, hx, hy) = hideout_pos(&g);

        g.player.x = hx;
        g.player.y = hy;
        g.update(&press_e(), DT);
        assert!(g.hidden_in.is_some(), "el jugador deberia esconderse con E");

        // El monstruo pegado al escondite no lo ve ni le hace dano.
        let h = g.hunter.as_mut().unwrap();
        h.x = hx + 0.4;
        h.y = hy;
        run(&mut g, 2.0);

        assert!(!g.hunter.as_ref().unwrap().sees_player());
        assert_eq!(g.health, 100.0);
        assert!(!g.is_dead());
    }

    #[test]
    fn level4_el_monstruo_abandona_la_busqueda() {
        let mut g = game(3); // Level 4
        let (_, hx, hy) = hideout_pos(&g);
        g.player.x = hx;
        g.player.y = hy;

        // Primero lo ve (sin esconderse): pasa a Chase.
        let (mx, my) = free_near(&g, hx, hy);
        {
            let h = g.hunter.as_mut().unwrap();
            h.x = mx;
            h.y = my;
            h.angle = (hy - my).atan2(hx - mx);
        }
        g.update(&idle(), DT);
        assert_eq!(g.hunter.as_ref().unwrap().state, HunterState::Chase);

        // Ahora se esconde: deja de verlo y acaba volviendo a campear objetos.
        g.update(&press_e(), DT);
        assert!(g.hidden_in.is_some());
        run(&mut g, 12.0);
        assert_eq!(g.hunter.as_ref().unwrap().state, HunterState::Guard);
        assert!(!g.is_dead(), "en level 4 el escondite siempre protege");
    }

    #[test]
    fn level5_si_lo_ve_esconderse_revisa_el_escondite_y_lo_mata() {
        let mut g = game(4); // Level 5
        let (_, hx, hy) = hideout_pos(&g);
        g.player.x = hx;
        g.player.y = hy;

        let (mx, my) = free_near(&g, hx, hy);
        {
            let h = g.hunter.as_mut().unwrap();
            h.x = mx;
            h.y = my;
            h.angle = (hy - my).atan2(hx - mx);
        }
        g.update(&idle(), DT); // lo ve
        assert!(g.hunter.as_ref().unwrap().sees_player());

        g.update(&press_e(), DT); // se esconde delante del monstruo
        assert!(g.hidden_in.is_some());
        assert_eq!(
            g.hunter.as_ref().unwrap().state,
            HunterState::CheckHideout,
            "el monstruo del level 5 debe recordar el escondite"
        );

        run(&mut g, 8.0);
        assert!(g.is_dead(), "revisar el escondite ocupado mata al jugador");
    }

    #[test]
    fn level5_salir_a_tiempo_permite_escapar() {
        let mut g = game(4); // Level 5
        let (_, hx, hy) = hideout_pos(&g);
        g.player.x = hx;
        g.player.y = hy;

        let (mx, my) = free_near(&g, hx, hy);
        {
            let h = g.hunter.as_mut().unwrap();
            h.x = mx;
            h.y = my;
            h.angle = (hy - my).atan2(hx - mx);
        }
        g.update(&idle(), DT);
        g.update(&press_e(), DT);
        assert_eq!(g.hunter.as_ref().unwrap().state, HunterState::CheckHideout);

        // Sale del escondite y huye al otro extremo del mapa.
        g.update(&press_e(), DT);
        assert!(g.hidden_in.is_none());
        let (dx, dy) = g.maze.deposit.unwrap();
        g.player.x = dx as f32 + 0.5;
        g.player.y = dy as f32 + 0.5;
        run(&mut g, 3.0);
        assert!(!g.is_dead(), "salir a tiempo debe permitir escapar");
    }

    #[test]
    fn level5_esconderse_sin_ser_visto_es_seguro() {
        let mut g = game(4); // Level 5
        let (_, hx, hy) = hideout_pos(&g);
        g.player.x = hx;
        g.player.y = hy;

        // El monstruo mira hacia el lado contrario y esta lejos.
        {
            let h = g.hunter.as_mut().unwrap();
            h.angle = 0.0;
        }
        g.update(&press_e(), DT);
        assert!(g.hidden_in.is_some());
        assert_ne!(
            g.hunter.as_ref().unwrap().state,
            HunterState::CheckHideout,
            "sin haberlo visto no puede conocer el escondite"
        );
        run(&mut g, 6.0);
        assert!(!g.is_dead());
    }

    #[test]
    fn todos_los_niveles_cargan_con_sus_cantidades() {
        for level in 0..LEVELS.len() {
            let g = game(level);
            assert!(g.maze.width > 0 && g.maze.height > 0);
            match level {
                3 => {
                    assert_eq!(g.items_total, 5);
                    assert_eq!(g.hideouts.len(), 6);
                    assert!(g.maze.deposit.is_some());
                    assert!(g.hunter.is_some());
                    assert_eq!(g.coins_total, 0);
                }
                4 => {
                    assert_eq!(g.items_total, 7);
                    assert_eq!(g.hideouts.len(), 3);
                    assert!(g.maze.deposit.is_some());
                    assert!(g.hunter.is_some());
                    assert_eq!(g.coins_total, 0);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn niveles_de_monedas_no_cambiaron() {
        for level in [0usize, 1] {
            let g = game(level);
            assert!(g.coins_total > 0);
            assert_eq!(g.items_total, 0);
            assert!(g.hunter.is_none());
            assert!(g.enemies.is_empty());
            assert!(g.hideouts.is_empty());
        }
        let g3 = game(2); // Level 3 intacto
        assert_eq!(g3.items_total, 3);
        assert_eq!(g3.enemies.len(), 2);
        assert!(g3.hunter.is_none());
    }
}
