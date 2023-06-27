use macroquad::prelude::*;

pub mod state;

use crate::state::constants::{HEIGHT, WIDTH};
use crate::state::entity::{
    Abilities, Full, FullEntity, Materials, Message, MovementType, Pos,
};

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

#[macroquad::main(window_conf)]
async fn main() {
    let mut texture_vec = vec![];
    texture_vec.push(load_texture("assets/wall.png").await.unwrap());
    texture_vec.push(load_texture("assets/crate.png").await.unwrap());
    texture_vec.push(load_texture("assets/arm_tower.png").await.unwrap());
    texture_vec.push(load_texture("assets/drill_tower.png").await.unwrap());
    texture_vec.push(load_texture("assets/gun_tower.png").await.unwrap());
    texture_vec.push(load_texture("assets/arm_tank.png").await.unwrap());
    texture_vec.push(load_texture("assets/drill_tank.png").await.unwrap());
    texture_vec.push(load_texture("assets/gun_tank.png").await.unwrap());

    let entity = FullEntity {
        pos: Pos::new(0, 0),
        hp: 3,
        max_hp: 3,
        inventory_size: 4,
        materials: Materials {
            carbon: 0,
            silicon: 1,
            plutonium: 23,
            copper: 5235,
        },
        abilities: Some(Abilities {
            movement_type: MovementType::Still,
            drill_damage: 2,
            gun_damage: 1,
            brain: Full {
                half: [None, None, None, None],
                message: Message {
                    emotion: 0,
                    pos: Pos::new(0, 0),
                },
                code_index: 2,
                gas: 2000,
            },
        }),
    };

    #[rustfmt::skip]
    let get_texture = |e: &FullEntity| {
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

        match (
        inventory, abilities, can_walk, can_drill, can_shoot,
    ) {
        (0, false, false, false, false) => 0,
        (_, false, false, false, false) => 1,
        (_, true,  false, false, false) => 2,
        (_, true,  false, true,  false) => 3,
        (_, true,  false, _,     true)  => 4,
        (_, true,  true,  false, false) => 5,
        (_, true,  true,  true,  false) => 6,
        (_, true,  true,  _,     true)  => 7,
        _ => unreachable!{}
        }
    };

    loop {
        clear_background(LIGHTGRAY);
        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                draw_texture(
                    texture_vec[get_texture(&entity)],
                    HOR_DISPLACE + (i as f32) * 16.,
                    VER_DISPLACE + (j as f32) * 16.,
                    WHITE,
                );
            }
        }

        if is_key_pressed(KeyCode::Escape) | is_key_pressed(KeyCode::Q) {
            break;
        }

        next_frame().await
    }
}
