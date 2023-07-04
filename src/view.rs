extern crate rand;
extern crate rand_chacha;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use std::fs::File;
use std::io::prelude::*;

use macroquad::prelude::*;

pub mod state;

use crate::state::constants::{HEIGHT, WIDTH};
use crate::state::entity::{FullEntity, Materials, MovementType, Team};
use crate::state::geometry::Pos;
use crate::state::replay::{implement_effect, Script};
use crate::state::state::State;

const HOR_DISPLACE: f32 = 150.;
const VER_DISPLACE: f32 = 25.;
const FRAME_TIME: f64 = 0.05;

fn window_conf() -> Conf {
    Conf {
        window_title: "Replay".to_owned(),
        window_height: 1020,
        window_width: 1300,
        ..Default::default()
    }
}

async fn draw_materials(
    mat: Materials,
    i: usize,
    j: usize,
    tileset: &Texture2D,
) {
    let mut rng =
        ChaCha8Rng::seed_from_u64((i * HEIGHT + j).try_into().unwrap());
    for _ in 0..mat.carbon {
        let draw_params = DrawTextureParams {
            source: Some(Rect {
                x: 0.0,
                y: 64.0,
                w: 16.0,
                h: 16.0,
            }),
            ..Default::default()
        };
        draw_texture_ex(
            *tileset,
            HOR_DISPLACE + ((16 * j) + rng.gen_range(0..13)) as f32,
            VER_DISPLACE + ((16 * i) + rng.gen_range(0..13)) as f32,
            WHITE,
            draw_params,
        );
    }
    for _ in 0..mat.silicon {
        let draw_params = DrawTextureParams {
            source: Some(Rect {
                x: 16.0,
                y: 64.0,
                w: 16.0,
                h: 16.0,
            }),
            ..Default::default()
        };
        draw_texture_ex(
            *tileset,
            HOR_DISPLACE + ((16 * j) + rng.gen_range(0..13)) as f32,
            VER_DISPLACE + ((16 * i) + rng.gen_range(0..13)) as f32,
            WHITE,
            draw_params,
        );
    }
    for _ in 0..mat.plutonium {
        let draw_params = DrawTextureParams {
            source: Some(Rect {
                x: 32.0,
                y: 64.0,
                w: 16.0,
                h: 16.0,
            }),
            ..Default::default()
        };
        draw_texture_ex(
            *tileset,
            HOR_DISPLACE + ((16 * j) + rng.gen_range(0..13)) as f32,
            VER_DISPLACE + ((16 * i) + rng.gen_range(0..13)) as f32,
            WHITE,
            draw_params,
        );
    }
    for _ in 0..mat.copper {
        let draw_params = DrawTextureParams {
            source: Some(Rect {
                x: 48.0,
                y: 64.0,
                w: 16.0,
                h: 16.0,
            }),
            ..Default::default()
        };
        draw_texture_ex(
            *tileset,
            HOR_DISPLACE + ((16 * j) + rng.gen_range(0..13)) as f32,
            VER_DISPLACE + ((16 * i) + rng.gen_range(0..13)) as f32,
            WHITE,
            draw_params,
        );
    }
}

async fn draw_map(state: &State, tileset: &Texture2D) {
    let mut pos: Pos;
    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            pos = Pos::new(i, j);
            draw_materials(
                state.tiles[pos.to_index()].materials.clone(),
                i,
                j,
                &tileset,
            )
            .await;
            draw_entity(state.get_entity_option(pos), i, j, &tileset).await;
        }
    }
}

async fn draw_entity(
    entity: Option<&FullEntity>,
    i: usize,
    j: usize,
    tileset: &Texture2D,
) {
    if let Some(e) = entity {
        let x = get_texture_x(&e);
        let y = match e.team {
            Team::Gray => 0.0,
            Team::Blue => 16.0,
            Team::Red => 32.0,
        };
        let draw_params = DrawTextureParams {
            source: Some(Rect {
                x,
                y,
                w: 16.0,
                h: 16.0,
            }),
            ..Default::default()
        };
        draw_texture_ex(
            *tileset,
            HOR_DISPLACE + (j as f32) * 16.,
            VER_DISPLACE + (i as f32) * 16.,
            WHITE,
            draw_params,
        );
    }
}

fn get_texture_x(e: &FullEntity) -> f32 {
    let inventory = e.inventory_size;
    let mut abilities = false;
    let mut can_walk = false;
    let mut can_drill = false;
    let mut can_shoot = false;
    if let Some(a) = &e.abilities {
        abilities = true;
        can_walk = a.movement_type == MovementType::Walk;
        can_drill = a.drill_damage > 0;
        can_shoot = a.gun_damage > 0;
    }

    match (inventory, abilities, can_walk, can_drill, can_shoot) {
        (0, false, false, false, false) => 7.0 * 16.0, // wall
        (_, false, false, false, false) => 2.0 * 16.0, // crate
        (_, true, false, false, false) => 1.0 * 16.0,  // arm tower
        (_, true, false, true, false) => 4.0 * 16.0,   // drill tower
        (_, true, false, _, true) => 6.0 * 16.0,       // gun tower
        (_, true, true, false, false) => 0.0,          // arm tank
        (_, true, true, true, false) => 3.0 * 16.0,    // drill tank
        (_, true, true, _, true) => 5.0 * 16.0,        // gun tank
        _ => unreachable! {},
    }
}

#[macroquad::main(window_conf)]
async fn main() -> std::io::Result<()> {
    let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(25).try_into().unwrap();
    let tileset = load_texture("assets/tileset.png").await.unwrap();

    let mut file = File::open("serialized/script_v1.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let script: Script = serde_json::from_str(&contents).unwrap();

    let mut state = script.genesis;

    let mut seconds = get_time();
    let mut frame_number = 0;
    let mut finished = false;

    let mut floor = [0; WIDTH * HEIGHT];
    for i in 0..(WIDTH * HEIGHT) {
        floor[i] = rng.gen_range(0..7);
    }

    loop {
        clear_background(GRAY);
        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                let f = floor[i * WIDTH + j];
                let draw_params = DrawTextureParams {
                    source: Some(Rect {
                        x: 16.0 * (f as f32),
                        y: 48.0,
                        w: 16.0,
                        h: 16.0,
                    }),
                    ..Default::default()
                };
                draw_texture_ex(
                    tileset,
                    HOR_DISPLACE + (16 * j) as f32,
                    VER_DISPLACE + (16 * i) as f32,
                    WHITE,
                    draw_params,
                );
            }
        }
        draw_map(&state, &tileset).await;
        //draw_map(&state, &texture_vec).await;
        if is_key_pressed(KeyCode::Escape) | is_key_pressed(KeyCode::Q) {
            break;
        }

        if get_time() > seconds + FRAME_TIME {
            seconds += FRAME_TIME;
            let frame = &script.frames.get(frame_number);
            if let Some(f) = frame {
                frame_number += 1;
                for e in f.iter() {
                    implement_effect(&mut state, e.clone()).unwrap();
                }
            } else {
                finished = true;
            }
        }
        if finished {
            draw_rectangle(10., 10., 40.0, 40.0, RED);
        }
        draw_text(format!("FPS: {}", get_fps()).as_str(), 0., 16., 32., WHITE);
        next_frame().await;
    }
    Ok(())
}
