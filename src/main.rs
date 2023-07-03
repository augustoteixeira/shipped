use macroquad::rand::gen_range;
use std::collections::HashMap;

use crate::state::actions::{validate_command, Command, Verb};
use crate::state::constants::{HEIGHT, NUM_TEMPLATES, WIDTH};
use crate::state::entity::{
    Abilities, Full, FullEntity, Materials, Message, MovementType,
};
use crate::state::geometry::{Direction, Pos};
use crate::state::replay::{implement_effect, Frame, Script};
use crate::state::state::{State, Team, Tile};

pub mod state;

fn random_entity() -> FullEntity {
    let max_hp = gen_range(1, 4);
    let quarter_inventory_size = gen_range(0, 10);
    FullEntity {
        pos: Pos::new(0, 0),
        hp: gen_range(1, max_hp),
        max_hp,
        inventory_size: 4 * quarter_inventory_size,
        materials: Materials {
            carbon: gen_range(0, quarter_inventory_size),
            silicon: gen_range(0, quarter_inventory_size),
            plutonium: gen_range(0, quarter_inventory_size),
            copper: gen_range(0, quarter_inventory_size),
        },
        abilities: Some(Abilities {
            movement_type: match gen_range(0, 2) {
                0 => MovementType::Still,
                _ => MovementType::Walk,
            },
            drill_damage: gen_range(0, 2),
            gun_damage: gen_range(0, 2),
            brain: Full {
                half: [None, None, None, None],
                message: Some(Message {
                    emotion: 0,
                    pos: Pos::new(0, 0),
                }),
                code_index: 2,
                gas: 2000,
            },
        }),
    }
}

fn random_direction() -> Verb {
    match gen_range(0, 4) {
        0 => Verb::AttemptMove(Direction::North),
        1 => Verb::AttemptMove(Direction::East),
        2 => Verb::AttemptMove(Direction::South),
        3 => Verb::AttemptMove(Direction::West),
        _ => unreachable!(),
    }
}

fn main() {
    let mut initial_state = State::new(
        std::array::from_fn(|_| None),
        HashMap::new(),
        std::array::from_fn(|_| Some(random_entity())),
        std::array::from_fn(|_| Some(random_entity())),
        std::array::from_fn(|_| Some(random_entity())),
        (0..(WIDTH * HEIGHT))
            .map(|_| Tile {
                entity_id: None,
                materials: Materials {
                    carbon: gen_range(0, 2),
                    silicon: gen_range(0, 2),
                    plutonium: gen_range(0, 2),
                    copper: gen_range(0, 2),
                },
            })
            .collect(),
    );
    for _ in 0..100 {
        let _ = initial_state.build_entity_from_template(
            Team::Blue,
            gen_range(0, NUM_TEMPLATES),
            Pos::new(gen_range(0, WIDTH), gen_range(0, HEIGHT)),
        );
    }
    let mut state = initial_state.clone();
    let mut frames: Vec<Frame> = vec![];
    for _ in 1..200 {
        let mut frame = vec![];
        let id_vec = state.get_entities_ids();
        for id in id_vec {
            let entity = state.get_entity_by_id(id).unwrap();
            eprintln!("Entity {} at {:?}", id, entity.pos);
            match validate_command(
                &state,
                Command {
                    entity_id: id,
                    verb: random_direction(),
                },
            ) {
                Ok(Some(e)) => {
                    eprintln!("Effect {:?}", e.clone());
                    frame.push(e.clone());
                    implement_effect(&mut state, e).unwrap();
                }
                Ok(None) => {}
                Err(e) => eprintln! {"Error {:}", e},
            }

            // let pos = Pos::new(gen_range(1, 59), gen_range(1, 59));
            // let new_pos = Pos::new(pos.x + 1, pos.y);
            // if state.has_entity(pos) & !state.has_entity(new_pos) {
            //     frame.push(Effect::EntityMove(pos, new_pos));
            //     break;
        }
        frames.push(frame);
    }
    let script: Script = Script {
        genesis: initial_state,
        frames,
    };
    let serialized = serde_json::to_string(&script).unwrap();
    println!("{}", serialized);
}