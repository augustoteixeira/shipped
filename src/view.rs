use std::fs::File;
use std::io::prelude::*;

use macroquad::prelude::*;

pub mod state;

use crate::state::constants::{HEIGHT, WIDTH};
use crate::state::entity::{FullEntity, MovementType, Pos};
use crate::state::replay::{replay_event, Script};
use crate::state::state::State;

const HOR_DISPLACE: f32 = 150.;
const VER_DISPLACE: f32 = 25.;

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

async fn draw_tile(
    entity: Option<&FullEntity>,
    i: usize,
    j: usize,
    texture_vec: &Vec<Texture2D>,
) {
    if let Some(e) = entity {
        draw_texture(
            texture_vec[get_texture(&e)],
            HOR_DISPLACE + (i as f32) * 16.,
            VER_DISPLACE + (j as f32) * 16.,
            WHITE,
        );
    }
}

async fn draw_map(state: &State, texture_vec: &Vec<Texture2D>) {
    let mut pos: Pos;
    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            pos = Pos::new(i, j);
            draw_tile(state.get_entity_option(pos), i, j, &texture_vec).await;
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

    println!("Bla");
    let mut file = File::open("serialized/script_v1.json")?;
    println!("Ble");
    let mut contents = String::new();
    println!("Bli");
    file.read_to_string(&mut contents)?;
    println!("Blo");

    // let mut deserializer = serde_json::Deserializer::from_str(&contents);
    // deserializer.disable_recursion_limit();
    // let deserializer = serde_stacker::Deserializer::new(&mut deserializer);

    let script: Script = serde_json::from_str(&contents).unwrap();

    println!("Blu");
    let mut state = script.genesis;
    println!("Bls");
    //let entity = state.entities.get(&1).unwrap();

    let mut seconds = get_time();

    loop {
        clear_background(LIGHTGRAY);
        draw_map(&state, &texture_vec).await;
        if is_key_pressed(KeyCode::Escape) | is_key_pressed(KeyCode::Q) {
            break;
        }

        if get_time() > seconds + 1. {
            seconds += 1.;
            println!("{:?}", script.frames[0][0].clone());
            replay_event(&mut state, script.frames[0][0].clone()).unwrap();
        }
        next_frame().await;
    }
    Ok(())
}
