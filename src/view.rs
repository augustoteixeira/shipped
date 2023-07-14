extern crate rand;
extern crate rand_chacha;
use macroquad::prelude::*;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::fs::File;
use std::io::prelude::*;

pub mod state;
pub mod ui;

use crate::state::constants::{HEIGHT, WIDTH};
use crate::state::state::{
    Command, GameStatus, Script, State, StateError, Verb,
};
use crate::ui::canvas::{draw_floor, draw_map};

const HOR_DISPLACE: f32 = 150.;
const VER_DISPLACE: f32 = 25.;
const FRAME_TIME: f64 = 0.05;

fn window_conf() -> Conf {
    Conf {
        window_title: "Replay".to_owned(),
        window_height: 1020,
        window_width: 1500,
        ..Default::default()
    }
}

async fn draw_command(
    state: &State,
    command: &Command,
) -> Result<(), StateError> {
    let entity = state.get_entity_by_id(command.entity_id)?;
    match command.verb.clone() {
        Verb::Shoot(disp) => {
            let from = entity.pos;
            let to = State::add_displace(from, &disp)?;
            if let Some(damage) = entity.get_gun_damage() {
                draw_line(
                    HOR_DISPLACE + (16 * from.x) as f32 + 8.0,
                    VER_DISPLACE + (16 * from.y) as f32 + 8.0,
                    HOR_DISPLACE + (16 * to.x) as f32 + 8.0,
                    VER_DISPLACE + (16 * to.y) as f32 + 8.0,
                    6.0 - (5.0 / (damage as f32)),
                    RED,
                );
                draw_circle(
                    HOR_DISPLACE + (16 * to.x) as f32 + 8.0,
                    VER_DISPLACE + (16 * to.y) as f32 + 8.0,
                    12.0 - (11.0 / (damage as f32)),
                    RED,
                );
            }
            Ok(())
        }
        Verb::Drill(dir) => {
            let from = entity.pos;
            let to = State::add_displace(from, &dir.into())?;
            if let Some(damage) = entity.get_gun_damage() {
                draw_line(
                    HOR_DISPLACE + (16 * from.x) as f32 + 8.0,
                    VER_DISPLACE + (16 * from.y) as f32 + 8.0,
                    HOR_DISPLACE + (16 * to.x) as f32 + 8.0,
                    VER_DISPLACE + (16 * to.y) as f32 + 8.0,
                    6.0 - (3.0 / (damage as f32)),
                    BLUE,
                );
                draw_circle(
                    HOR_DISPLACE + (16 * to.x) as f32 + 8.0,
                    VER_DISPLACE + (16 * to.y) as f32 + 8.0,
                    12.0 - (6.0 / (damage as f32)),
                    BLUE,
                );
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[macroquad::main(window_conf)]
async fn main() -> std::io::Result<()> {
    let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(25).try_into().unwrap();
    let tileset = load_texture("assets/tileset.png").await.unwrap();
    // deserialize script
    let mut file = File::open("serialized/script_v1.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let script: Script = serde_json::from_str(&contents).unwrap();
    let mut state = script.genesis;
    let mut frames = script.frames.into_iter();
    // time constants
    let mut seconds = get_time();
    let mut finished = false;
    // setup floor tiles
    let mut floor = [0; WIDTH * HEIGHT];
    for i in 0..(WIDTH * HEIGHT) {
        floor[i] = rng.gen_range(0..7);
    }
    // main loop
    loop {
        // exit logic
        if is_key_pressed(KeyCode::Escape) | is_key_pressed(KeyCode::Q) {
            break;
        }
        // drawing
        clear_background(GRAY);
        draw_floor(HOR_DISPLACE, VER_DISPLACE, &tileset, &floor).await;
        draw_map(&state, HOR_DISPLACE, VER_DISPLACE, &tileset).await;
        draw_text(format!("FPS: {}", get_fps()).as_str(), 0., 16., 32., WHITE);
        draw_text(
            format!("Blue Tokens: {}", state.blue_tokens).as_str(),
            1200.,
            16.,
            32.,
            WHITE,
        );
        draw_text(
            format!("Red_Tokens: {}", state.red_tokens).as_str(),
            1200.,
            116.,
            32.,
            WHITE,
        );
        draw_text(
            format!("St: {:?}, min {}", state.game_status, state.min_tokens)
                .as_str(),
            1200.,
            216.,
            32.,
            WHITE,
        );
        // update
        if (get_time() > seconds + FRAME_TIME)
            & (state.game_status == GameStatus::Running)
        {
            seconds += FRAME_TIME;
            if let Some(f) = frames.next() {
                for command in f.iter() {
                    if state.execute_command(command.clone()).is_ok() {
                        let _ = draw_command(&state, command).await;
                    }
                }
            } else {
                finished = true;
            }
        }
        if finished {
            draw_rectangle(10., 40., 40.0, 40.0, BLACK);
        }
        if state.game_status == GameStatus::BlueWon {
            draw_rectangle(10., 40., 40.0, 40.0, BLUE);
        }
        if state.game_status == GameStatus::RedWon {
            draw_rectangle(10., 40., 40.0, 40.0, RED);
        }
        next_frame().await;
    }
    Ok(())
}
