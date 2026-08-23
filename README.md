# Proyecto 1 - Raycaster

Motor base de un videojuego en primera persona estilo retro (Wolfenstein 3D),
con raycasting implementado a mano.

## Tecnologia

- Rust + Cargo
- [raylib](https://crates.io/crates/raylib) (unica dependencia)
- Raycasting por DDA propio, sin renderizado 3D nativo ni shaders

## Compilacion

```bash
cargo build --release
```

## Ejecucion

```bash
cargo run --release
```

Los niveles se cargan desde `levels/level1.txt` .. `levels/level5.txt`, por lo
que el juego debe ejecutarse desde la raiz del proyecto.

## Controles

| Accion             | Tecla                |
| ------------------ | -------------------- |
| Avanzar            | `W` / `↑`            |
| Retroceder         | `S` / `↓`            |
| Girar a la izquierda | `A`                |
| Girar a la derecha | `D`                  |
| Strafe izquierda   | `←`                  |
| Strafe derecha     | `→`                  |
| Girar camara       | Mouse (horizontal)   |
| Esconderse / salir | `E` (cerca de un escondite) |
| Reiniciar nivel    | `R`                  |
| Menu / retroceder  | `ESC`                |
| Navegar menus      | `↑` / `↓` o mouse    |
| Confirmar en menus | `ENTER` o clic       |

## Flujo del juego

- **Main Menu**: `PLAY` abre el selector, `QUIT` cierra. `ESC` tambien cierra.
- **Level Select**: 5 niveles; solo se puede entrar a los desbloqueados
  (al inicio unicamente el 1). `ESC` vuelve al menu principal.
- **Gameplay**: el objetivo depende del nivel (ver abajo). `R` reinicia el
  nivel, `ESC` vuelve al selector.
- **Game Over**: si la salud llega a 0 en un nivel con enemigos.
  `R` reintenta, `ESC` vuelve al selector.
- **Level Complete**: `ENTER` vuelve al selector con el siguiente nivel ya
  desbloqueado, `R` repite el nivel, `ESC` vuelve al selector.

El progreso no se guarda en disco: al cerrar el programa se reinicia.

## Objetivos

- **Niveles 1 y 2**: recoger todas las monedas y despues llegar a `G`.
  HUD: `COINS: X / total`.
- **Nivel 3**: recoger los 3 objetos y entregarlos en la zona de deposito `D`.
  Dos enemigos simples persiguen al jugador. HUD: `ITEMS: X / 3` y `HEALTH`.
- **Niveles 4 y 5**: recoger todos los objetos (5 y 7) y entregarlos en `D`,
  evitando al monstruo. Hay escondites `H` que se usan con `E`.
  - Nivel 4: esconderse siempre protege.
  - Nivel 5: si el monstruo te VE entrar al escondite, va a revisarlo y te
    mata si sigues dentro. Hay que salir antes de que llegue.

## Formato del mapa

```
# = pared tipo 1     B = pared tipo 2     C = pared tipo 3
. = piso             P = jugador          G = meta
O = moneda        I = objeto        D = zona de deposito
E = enemigo simple    M = monstruo      H = escondite
(O, I, D, E, M y H dejan la celda transitable)

`G` solo tiene funcion en los niveles 1 y 2; en los demas permanece en el
archivo por compatibilidad del parser y no se dibuja en el minimapa.
```

## Assets

El juego funciona sin ningun archivo grafico: si una textura no existe se usa
el fallback geometrico (colores solidos). La carpeta `assets/` esta preparada
para incorporarlos despues:

```
assets/
├── textures/   wall_1.png, wall_2.png, wall_3.png
├── sprites/
│   ├── coins/  coin.png
│   ├── items/  item.png
│   ├── enemies/ enemy.png
│   └── hunter/ hunter.png, hunter_walk.png (sprite sheet)
├── ui/         deposit.png, hideout.png, menu_background.png,
│               menu_character.png, locker_overlay.png
└── audio/
```

Los recursos se cargan una sola vez al arrancar (`Assets::load`), nunca dentro
del bucle de render.

## Pruebas

Las reglas de objetivo y de escondites tienen pruebas automaticas que no
necesitan ventana:

```bash
cargo test
```
