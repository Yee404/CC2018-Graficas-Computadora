use raylib::prelude::*;

const WIDTH: usize = 100;
const HEIGHT: usize = 100;

//solo evita repetir pq al inicio eran muchos de board[y][x] = true;
fn set_cell(
    board: &mut [[bool; WIDTH]; HEIGHT],
    x: usize,
    y: usize,
) {
    if x < WIDTH && y < HEIGHT {
        board[y][x] = true;
    }
}

fn glider(
    board: &mut [[bool; WIDTH]; HEIGHT],
    x: usize,
    y: usize,
) {
    set_cell(board, x + 1, y);
    set_cell(board, x + 2, y + 1);
    set_cell(board, x,     y + 2);
    set_cell(board, x + 1, y + 2);
    set_cell(board, x + 2, y + 2);
}





// M A I N
pub fn main() {

    let (mut rl, thread) = raylib::init()
        .size(800, 800)
        .title("Conway's Game of Life")
        .build();

    rl.set_target_fps(10);

    let mut board = [[false; WIDTH]; HEIGHT];


    while !rl.window_should_close() {

        board = update(&board);

        let mut image = Image::gen_image_color(
            WIDTH as i32,
            HEIGHT as i32,
            Color::BLACK,
        );

        render(
            &mut image,
            &board,
        );

        glider(&mut board, 10, 10);

        let texture = rl
            .load_texture_from_image(
                &thread,
                &image,
            )
            .unwrap();

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        d.draw_texture_ex(
            &texture,
            Vector2::new(0.0, 0.0),
            0.0,
            8.0,
            Color::WHITE,
        );
    }
}



//Renderiza el tablero a imagen
fn render(
    image: &mut Image,
    board: &[[bool; WIDTH]; HEIGHT],
) {

    for y in 0..HEIGHT {

        for x in 0..WIDTH {

            if board[y][x] {

                image.draw_pixel(
                    x as i32,
                    y as i32,
                    Color::WHITE,
                );

            } else {

                image.draw_pixel(
                    x as i32,
                    y as i32,
                    Color::BLACK,
                );

            }

        }

    }

}

fn count_neighbors(
    board: &[[bool; WIDTH]; HEIGHT],
    x: usize,
    y: usize,
) -> i32 {

    let mut vecinos = 0;

    for dy in -1..=1 {
        for dx in -1..=1 {

            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0
                && nx < WIDTH as i32
                && ny >= 0
                && ny < HEIGHT as i32
            {

                if board[ny as usize][nx as usize] {
                    vecinos += 1;
                }

            }

        }
    }

    vecinos
}


fn update(
    board: &[[bool; WIDTH]; HEIGHT],
) -> [[bool; WIDTH]; HEIGHT] {

    let mut next = [[false; WIDTH]; HEIGHT];

    for y in 0..HEIGHT {

        for x in 0..WIDTH {

            let vecinos = count_neighbors(board, x, y);

            if board[y][x] {

                // La célula está viva
                if vecinos == 2 || vecinos == 3 {
                    next[y][x] = true;
                }

            } else {

                // La célula está muerta
                if vecinos == 3 {
                    next[y][x] = true;
                }

            }

        }

    }

    next
}



// UNAS FUENTES CON DISEÑOS INCREÍBLES
// https://www.youtube.com/shorts/m15WKzgZ2hI
// el inicio de este video que me encanta y repito: https://www.youtube.com/watch?v=RRg38oNQ9vk&t=208s
// y luego está esta bestialidad jejejej: https://www.youtube.com/watch?v=C2vgICfQawE&t=188s