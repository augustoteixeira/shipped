extern crate rand;
extern crate rand_chacha;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use std::fs::File;
use std::io::prelude::*;

use macroquad::prelude::*;

pub mod state;

use crate::state::constants::{HEIGHT, WIDTH};
use crate::state::entity::{FullEntity, Materials, MovementType};
use crate::state::geometry::Pos;
use crate::state::replay::{implement_effect, Script};
use crate::state::state::State;

const HOR_DISPLACE: f32 = 150.;
const VER_DISPLACE: f32 = 25.;
const FRAME_TIME: f64 = 0.2;

fn window_conf() -> Conf {
    Conf {
        window_title: "Replay".to_owned(),
        window_height: 1020,
        window_width: 1300,
        ..Default::default()
    }
}

fn get_texture(e: &FullEntity) -> usize {
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
        (0, false, false, false, false) => 0,
        (_, false, false, false, false) => 1,
        (_, true, false, false, false) => 2,
        (_, true, false, true, false) => 3,
        (_, true, false, _, true) => 4,
        (_, true, true, false, false) => 5,
        (_, true, true, true, false) => 6,
        (_, true, true, _, true) => 7,
        _ => unreachable! {},
    }
}

async fn draw_entity(
    entity: Option<&FullEntity>,
    i: usize,
    j: usize,
    texture_vec: &Vec<Texture2D>,
) {
    if let Some(e) = entity {
        draw_texture(
            texture_vec[get_texture(&e)],
            HOR_DISPLACE + (j as f32) * 16.,
            VER_DISPLACE + (i as f32) * 16.,
            WHITE,
        );
    }
}

async fn draw_materials(mat: Materials, i: usize, j: usize) {
    let mut rng =
        ChaCha8Rng::seed_from_u64((i * HEIGHT + j).try_into().unwrap());
    for k in 0..mat.carbon {
        draw_rectangle(
            HOR_DISPLACE + ((16 * j) + rng.gen_range(0..13)) as f32,
            VER_DISPLACE + ((16 * i) + rng.gen_range(0..13)) as f32,
            2.0,
            2.0,
            BLACK,
        );
    }
    for k in 0..mat.silicon {
        draw_rectangle(
            HOR_DISPLACE + ((16 * j) + rng.gen_range(0..13)) as f32,
            VER_DISPLACE + ((16 * i) + rng.gen_range(0..13)) as f32,
            2.0,
            2.0,
            GRAY,
        );
    }
    for k in 0..mat.plutonium {
        draw_rectangle(
            HOR_DISPLACE + ((16 * j) + rng.gen_range(0..13)) as f32,
            VER_DISPLACE + ((16 * i) + rng.gen_range(0..13)) as f32,
            2.0,
            2.0,
            GREEN,
        );
    }
    for k in 0..mat.copper {
        draw_rectangle(
            HOR_DISPLACE + ((16 * j) + rng.gen_range(0..13)) as f32,
            VER_DISPLACE + ((16 * i) + rng.gen_range(0..13)) as f32,
            2.0,
            2.0,
            ORANGE,
        );
    }
}

async fn draw_map(state: &State, texture_vec: &Vec<Texture2D>) {
    let mut pos: Pos;
    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            pos = Pos::new(i, j);
            draw_materials(state.tiles[pos.to_index()].materials.clone(), i, j)
                .await;
            draw_entity(state.get_entity_option(pos), i, j, &texture_vec).await;
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() -> std::io::Result<()> {
    let mut texture_vec = vec![];
    texture_vec.push(load_texture("assets/wall.png").await.unwrap());
    texture_vec.push(load_texture("assets/crate.png").await.unwrap());
    texture_vec.push(load_texture("assets/arm_tower.png").await.unwrap());
    texture_vec.push(load_texture("assets/drill_tower.png").await.unwrap());
    texture_vec.push(load_texture("assets/gun_tower.png").await.unwrap());
    texture_vec.push(load_texture("assets/arm_tank.png").await.unwrap());
    texture_vec.push(load_texture("assets/drill_tank.png").await.unwrap());
    texture_vec.push(load_texture("assets/gun_tank.png").await.unwrap());

    let mut file = File::open("serialized/script_v1.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let script: Script = serde_json::from_str(&contents).unwrap();

    let mut state = script.genesis;

    let mut seconds = get_time();
    let mut frame_number = 0;
    let mut finished = false;

    loop {
        clear_background(BEIGE);
        draw_map(&state, &texture_vec).await;
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

        next_frame().await;
    }
    Ok(())
}
