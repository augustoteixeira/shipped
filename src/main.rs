extern crate rand;
extern crate rand_chacha;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

pub mod state;

use crate::state::constants::{HEIGHT, NUM_TEMPLATES, WIDTH};
use crate::state::entity::{
    Abilities, Full, FullEntity, Materials, Message, MovementType, Team,
};
use crate::state::geometry::{Direction, Displace, Neighbor, Pos};
use crate::state::replay::{Frame, Script};
use crate::state::state::{Command, State, Tile, Verb};

fn random_entity(rng: &mut ChaCha8Rng, team: Team) -> FullEntity {
    let max_hp = rng.gen_range(1..4);
    let quarter_inventory_size = rng.gen_range(0..10);
    FullEntity {
        team,
        pos: Pos::new(0, 0),
        hp: rng.gen_range(1..(max_hp + 1)),
        max_hp,
        inventory_size: 4 * quarter_inventory_size,
        materials: Materials {
            carbon: rng.gen_range(0..(quarter_inventory_size + 1)),
            silicon: rng.gen_range(0..(quarter_inventory_size + 1)),
            plutonium: rng.gen_range(0..(quarter_inventory_size + 1)),
            copper: rng.gen_range(0..(quarter_inventory_size + 1)),
        },
        abilities: Some(Abilities {
            movement_type: match rng.gen_range(0..2) {
                0 => MovementType::Still,
                _ => MovementType::Walk,
            },
            drill_damage: rng.gen_range(0..2),
            gun_damage: rng.gen_range(0..2) + 4 * rng.gen_range(0..2),
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

fn random_direction(rng: &mut ChaCha8Rng) -> Direction {
    match rng.gen_range(0..4) {
        0 => Direction::North,
        1 => Direction::East,
        2 => Direction::South,
        _ => Direction::West,
    }
}

fn random_neighbor(rng: &mut ChaCha8Rng) -> Neighbor {
    match rng.gen_range(0..5) {
        0 => Neighbor::North,
        1 => Neighbor::East,
        2 => Neighbor::South,
        3 => Neighbor::West,
        _ => Neighbor::Here,
    }
}

fn random_vicinity(rng: &mut ChaCha8Rng) -> Displace {
    Displace::new(
        rng.gen_range(0..11) as i64 - 5,
        rng.gen_range(0..11) as i64 - 5,
    )
}

fn random_material(rng: &mut ChaCha8Rng) -> Materials {
    let material_type = rng.gen_range(0..4);
    Materials {
        carbon: if material_type == 0 { 1 } else { 0 },
        silicon: if material_type == 1 { 1 } else { 0 },
        plutonium: if material_type == 2 { 1 } else { 0 },
        copper: if material_type == 3 { 1 } else { 0 },
    }
}

fn random_verb(rng: &mut ChaCha8Rng) -> Verb {
    match rng.gen_range(0..7) {
        0 => Verb::AttemptMove(random_direction(rng)),
        1 => Verb::GetMaterials(random_neighbor(rng), random_material(rng)),
        2 => Verb::DropMaterials(random_neighbor(rng), random_material(rng)),
        3 => Verb::Shoot(random_vicinity(rng)),
        4 => Verb::Construct(
            rng.gen_range(0..NUM_TEMPLATES),
            random_direction(rng),
        ),
        _ => Verb::Drill(random_direction(rng)),
    }
}

fn main() {
    let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(17).try_into().unwrap();

    let mut initial_state = State::new(
        std::array::from_fn(|_| None),
        HashMap::new(),
        std::array::from_fn(|_| Some(random_entity(&mut rng, Team::Blue))),
        std::array::from_fn(|_| Some(random_entity(&mut rng, Team::Gray))),
        std::array::from_fn(|_| Some(random_entity(&mut rng, Team::Red))),
        (0..(WIDTH * HEIGHT))
            .map(|_| Tile {
                entity_id: None,
                materials: Materials {
                    carbon: rng.gen_range(0..20) / 13,
                    silicon: rng.gen_range(0..20) / 13,
                    plutonium: rng.gen_range(0..20) / 13,
                    copper: rng.gen_range(0..20) / 13,
                },
            })
            .collect(),
    );
    for _ in 0..100 {
        let pos = Pos::new(rng.gen_range(0..WIDTH), rng.gen_range(0..HEIGHT));
        let _ = initial_state.build_entity_from_template(
            match rng.gen_range(0..3) {
                0 => Team::Blue,
                1 => Team::Gray,
                _ => Team::Red,
            },
            rng.gen_range(0..NUM_TEMPLATES),
            pos,
        );
    }
    let mut state = initial_state.clone();
    let mut frames: Vec<Frame> = vec![];
    for _ in 1..1000 {
        let mut frame = vec![];
        let id_vec = state.get_entities_ids();
        for id in id_vec {
            let command = Command {
                entity_id: id,
                verb: random_verb(&mut rng),
            };
            if let Ok(_) = state.execute_command(command.clone()) {
                frame.push(command.clone());
            }
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
