extern crate sdl2;

mod tetrimino;
mod game_board;

use sdl2::event::Event;
use sdl2::image::{LoadTexture, INIT_JPG, INIT_PNG};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{TextureCreator, Canvas, Texture};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::Rect;

use std::time::{Duration, SystemTime};

use std::fs::File;
use std::io::{self, Read, Write};

use tetrimino::create_new_tetrimino;
use game_board::Tetris;
use game_board::LEVEL_TIMES;

const NB_HIGHSCORES: usize = 5;
const TETRIS_HEIGHT: usize = 40;
const HIGHSCORE_FILE: &'static str = "scores.txt";

fn write_to_file(contents: String, file_name: &str) -> io::Result<()> {
    let mut f = File::create(file_name)?;
    f.write_all(contents.as_bytes())
}

fn read_from_file(file_name: &str) -> io::Result<String> {
    let mut f = File::open(file_name)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    Ok(content)
}

fn slice_to_string(slice: &[u32]) -> String {
    slice
        .iter()
        .map(|highscore| highscore.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

fn line_to_slice(line: &str) -> Vec<u32> {
    line.split(" ")
        .filter_map(|nb| nb.parse::<u32>().ok())
        .collect()
}

fn save_highscores_and_lines(highscores: &[u32], number_of_lines: &[u32]) -> bool {
    let s_highscores = slice_to_string(highscores);
    let s_number_of_lines = slice_to_string(number_of_lines);

    write_to_file(format!("{}\n{}\n", s_highscores, s_number_of_lines), HIGHSCORE_FILE).is_ok()
}

fn load_highscores_and_lines() -> Option<(Vec<u32>, Vec<u32>)> {
    if let Ok(content) = read_from_file(HIGHSCORE_FILE) {
        let mut lines = content.splitn(2, "\n")
                               .map(|line| line_to_slice(line))
                               .collect::<Vec<_>>();
        if lines.len() == 2 {
            let (number_lines, highscores) = (lines.pop().unwrap(), lines.pop().unwrap());
            Some((highscores, number_lines))
        } else {
            None
        }
    } else {
        None
    }
}

fn update_vec(v: &mut Vec<u32>, value: u32) -> bool {
    if v.len() < NB_HIGHSCORES {
        v.push(value);
        v.sort();
        true
    } else {
        for entry in v.iter_mut() {
            if value > *entry {
                *entry = value;
                return true;
            }
        }
        false
    }

}

fn handle_events(tetris: &mut Tetris, quit: &mut bool, timer: &mut SystemTime, event_pump: &mut sdl2::EventPump) -> bool {
    let mut make_permanent = false;
    if let Some(ref mut piece) = tetris.current_piece {
        let mut tmp_x = piece.x;
        let mut tmp_y = piece.y;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    *quit = true;
                    break
                },
                Event::KeyDown {keycode: Some(Keycode::Down), ..} => {
                    *timer = SystemTime::now();
                    tmp_y += 1;
                },
                Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
                     tmp_x += 1;
                },
                Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
                    tmp_x -= 1;
                },
                Event::KeyDown {keycode: Some(Keycode::Up), ..} => {
                    piece.rotate(&tetris.game_map);
                },
                Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
                    let x = piece.x;
                    let mut y = piece.y;
                    while piece.change_position(&tetris.game_map, x, y + 1) {
                        y += 1;
                    }
                    make_permanent = true;
                },
                _ => {}
            }
        }
        if !make_permanent {
            if !piece.change_position(&tetris.game_map, tmp_x, tmp_y)
                && tmp_y != piece.y {
                make_permanent = true;
            }
        }
    }
    if make_permanent {
        tetris.make_permanent();
        *timer = SystemTime::now();
    }
    make_permanent
}

fn create_texture_rect<'a>(canvas: &mut Canvas<Window>,
                           texture_creator: &'a TextureCreator<WindowContext>,
                           color: Color, width: u32, height: u32) -> Option<Texture<'a>> {
    // We'll want to handle failures outside of this function.
    if let Ok(mut square_texture) =
    texture_creator.create_texture_target(None, width, height) {
        canvas.with_texture_canvas(&mut square_texture, |texture| {
            texture.set_draw_color(color);
            texture.clear();
        }).expect("Failed to color a texture");
        Some(square_texture)
    } else {
        None
    }
}

fn create_texture_from_text<'a>(texture_creator: &'a TextureCreator<WindowContext>,
                                font: &sdl2::ttf::Font,
                                text: &str,
                                color: Color) -> Option<Texture<'a>> {
    if let Ok(surface) = font.render(text).blended(color) {
        texture_creator.create_texture_from_surface(&surface).ok()
    } else {
        None
    }
}

fn get_rect_from_text(text: &str, x: i32, y: i32) -> Option<Rect> {
    Some(Rect::new(x, y, text.len() as u32 * 20, 30))
}

fn print_information(tetris: &Tetris) {
    let mut new_highest_highscore = true;
    let mut new_highest_lines_sent = true;

    if let Some((mut highscores, mut lines_sent)) = load_highscores_and_lines() {
        new_highest_highscore = update_vec(&mut highscores, tetris.score);
        new_highest_lines_sent = update_vec(&mut lines_sent, tetris.nb_lines);

        if new_highest_lines_sent || new_highest_highscore {
            save_highscores_and_lines(&highscores, &lines_sent);
        }
    } else {
        save_highscores_and_lines(&[tetris.score], &[tetris.nb_lines]);
    }
    println!("Game over!");
    println!("Score:           {}{}", tetris.score
                                    , if new_highest_highscore { " [NEW HIGHSCORE]" } else { "" });
    println!("Number of lines: {}{}", tetris.nb_lines
                                    , if new_highest_lines_sent { " [NEW HIGHSCORE]" } else { "" });
    println!("Current level:   {}", tetris.current_level);
}

fn display_game_information<'a>(tetris: &Tetris, canvas: &mut Canvas<Window>, texture_creator: &'a TextureCreator<WindowContext>, font: &sdl2::ttf::Font, start_x_point: i32) {
    let score_text = format!("Score: {}", tetris.score);
    let lines_sent_text = format!("Lines sent: {}", tetris.nb_lines);
    let level_text = format!("Level: {}", tetris.current_level);

    let score = create_texture_from_text(&texture_creator, &font, &score_text, Color::RGB(255, 255, 255))
        .expect("Couldn't render the score text");
    let lines_sent = create_texture_from_text(&texture_creator, &font, &lines_sent_text, Color::RGB(255, 255, 255))
        .expect("Couldn't render the lines sent text");
    let level = create_texture_from_text(&texture_creator, &font, &level_text, Color::RGB(255, 255, 255))
        .expect("Couldn't render the level text");

    canvas.copy(&score, None, get_rect_from_text(&score_text, start_x_point, 90))
        .expect("Couldn't copy the score text");
    canvas.copy(&lines_sent, None, get_rect_from_text(&lines_sent_text, start_x_point, 125))
        .expect("Couldn't copy the lines sent text");
    canvas.copy(&level, None, get_rect_from_text(&level_text, start_x_point, 160))
        .expect("Couldn't copy the level text");
}

fn is_time_over(tetris: &Tetris, timer: &SystemTime) -> bool{
    match timer.elapsed() {
        Ok(elapsed) => {
            let millis = elapsed.as_millis();
            millis > LEVEL_TIMES[tetris.current_level as usize] as u128
        },
        Err(_) => false,
    }
}

fn main() {
    let sdl_context = sdl2::init().expect("SDL initialization failed");

    let video_subsystem = sdl_context
        .video()
        .expect("Couldn't get SDL video subsystem");

    let width = 800;
    let height = 800;

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Failed to get SDL event pump");

    let mut tetris = Tetris::new();
    let mut timer = SystemTime::now();

    let grid_x = 20; //(width - TETRIS_HEIGHT as u32 * 10) as i32 / 2;
    let grid_y = (height - TETRIS_HEIGHT as u32 * 16) as i32 / 2;

    let window = video_subsystem
        .window("Tetris", width, height)
        .position_centered()
        .build()
        .expect("Failed to create the window");

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .expect("Couldn't get the window canvas");

    let texture_creator: TextureCreator<_> = canvas.texture_creator();

    let grid = create_texture_rect(&mut canvas, &texture_creator, Color::RGB(0, 0, 0)
                                   , TETRIS_HEIGHT as u32 * 10, TETRIS_HEIGHT as u32 * 16)
        .expect("Failed to create the grid texture");

    let border = create_texture_rect(&mut canvas, &texture_creator, Color::RGB(255, 255, 255)
                                    , TETRIS_HEIGHT as u32 * 10 + 20, TETRIS_HEIGHT as u32 * 16 + 20)
        .expect("Failed to create the border texture");

    macro_rules! texture {
        ($r:expr, $g: expr, $b: expr) => {
            create_texture_rect(&mut canvas, &texture_creator, Color::RGB($r, $g, $b)
                               , TETRIS_HEIGHT as u32, TETRIS_HEIGHT as u32)
                .unwrap()
        }
    }

    let textures = [texture!(255,  69,  69),
                          texture!(255, 220,  69),
                          texture!(237, 150,  37),
                          texture!(171,  99, 137),
                          texture!( 77, 149, 239),
                          texture!( 39, 218, 225),
                          texture!( 45, 216,  47)];

    let ttf_context = sdl2::ttf::init().expect("SDL TTF initialization failed");
    let font = ttf_context
        .load_font("assets/lucon.ttf", 64)
        .expect("Couldn't load the font");



    loop {
        if is_time_over(&tetris, &timer){
            let mut make_permanent = false;
            if let Some(ref mut piece) = tetris.current_piece {
                let x = piece.x;
                let y = piece.y + 1;
                make_permanent = !piece.change_position(&tetris.game_map, x, y);
            }
            if make_permanent {
                tetris.make_permanent();
            }
            timer = SystemTime::now();
        }

        // draw the tetris grid here
        canvas.set_draw_color(Color::RGB(0, 0, 255));
        canvas.clear();
        canvas.copy(&border, None,
                    Rect::new(grid_x - 10,
                              grid_y - 10,
                              TETRIS_HEIGHT as u32 * 10 + 20,
                              TETRIS_HEIGHT as u32 * 16 + 20))
            .expect("Couldn't copy border texture into window");

        canvas.copy(&grid, None,
                    Rect::new(grid_x,
                              grid_y,
                              TETRIS_HEIGHT as u32 * 10,
                              TETRIS_HEIGHT as u32 * 16 ))
            .expect("Couldn't copy grid texture into window");



        if tetris.current_piece.is_none() {
            let current_piece = create_new_tetrimino();
            if !current_piece.test_current_position(&tetris.game_map) {
                print_information(&tetris);
                break
            }
            tetris.current_piece = Some(current_piece);
        }

        let mut quit = false;
        if !handle_events(&mut tetris, &mut quit, &mut timer, &mut event_pump) {
            if let Some(ref mut piece) = tetris.current_piece {
                //draw the current tetrimino here
                for (line_nb, line) in piece.states[piece.current_state as usize].iter().enumerate() {
                    for (case_nb, case) in line.iter().enumerate() {
                        if *case == 0 {
                            continue
                        }
                        canvas.copy(&textures[*case as usize - 1], None,
                                    Rect::new(grid_x + (piece.x + case_nb as isize) as i32 * TETRIS_HEIGHT as i32,
                                                    grid_y + (piece.y + line_nb) as i32 * TETRIS_HEIGHT as i32,
                                                    TETRIS_HEIGHT as u32,
                                                    TETRIS_HEIGHT as u32))
                            .expect("Couldn't copy the current piece texture into canvas");
                    }
                }
            }
        }

        if quit {
            print_information(&tetris);
            break
        }

        // draw the game map here
        for (line_nb, line) in tetris.game_map.iter().enumerate(){
            for (case_nb, case) in line.iter().enumerate() {
                if *case == 0 {
                    continue
                }
                canvas.copy(&textures[*case as usize - 1], None,
                            Rect::new(grid_x + case_nb as i32 * TETRIS_HEIGHT as i32,
                                            grid_y + line_nb as i32 * TETRIS_HEIGHT as i32,
                                            TETRIS_HEIGHT as u32,
                                            TETRIS_HEIGHT as u32 ))
                    .expect("Couldn't copy the placed piece texture into canvas");
            }
        }
        display_game_information(&tetris, &mut canvas, &texture_creator, &font, width as i32 - grid_x - 300);
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
