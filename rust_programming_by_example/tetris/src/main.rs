extern crate sdl2;

mod tetrimino;
mod game_board;

use sdl2::event::Event;
use sdl2::image::{LoadTexture, INIT_JPG, INIT_PNG};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;

use std::time::{Duration, SystemTime};

use std::fs::File;
use std::io::{self, Read, Write};

use tetrimino::create_new_tetrimino;
use game_board::Tetris;
use game_board::LEVEL_TIMES;

const NB_HIGHSCORES: usize = 5;

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

    write_to_file(format!("{}\n{}\n", s_highscores, s_number_of_lines), "scores.txt").is_ok()
}

fn load_highscores_and_lines() -> Option<(Vec<u32>, Vec<u32>)> {
    if let Ok(content) = read_from_file("scores.txt") {
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

    sdl2::image::init(INIT_PNG | INIT_JPG).expect("Couldn't initialize image context");

    let window = video_subsystem
        .window("rust-sdl2 image demo", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .expect("Failed to create window");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("Failed to convert window into canvas");

    let texture_creator: TextureCreator<_> = canvas.texture_creator();
    let image_texture = texture_creator
        .load_texture("assets/my_image.jpg")
        .expect("Couldn't load image");

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Failed to get SDL event pump");

    let mut tetris = Tetris::new();
    let mut timer = SystemTime::now();

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
            }
        }

        if quit {
            print_information(&tetris);
            break
        }

        // draw the game map here

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
