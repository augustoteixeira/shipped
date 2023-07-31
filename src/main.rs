extern crate rand;
extern crate rand_chacha;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

pub mod state;
pub mod ui;

use crate::state::constants::{HEIGHT, NUM_TEMPLATES, WIDTH};
use crate::state::entity::{
    Abilities, Full, Message, MovementType, Team, TemplateEntity,
};
use crate::state::geometry::{Direction, Displace, Neighbor, Pos};
use crate::state::materials::Materials;
use crate::state::squad::{build_state, Placement, Settings, Squad};
use crate::state::state::{Command, Frame, Script, Tile, Verb};

fn random_entity(rng: &mut ChaCha8Rng) -> TemplateEntity {
    let quarter_inventory_size = rng.gen_range(0..10);
    TemplateEntity {
        hp: rng.gen_range(1..4),
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
            message: Some(Message {
                emotion: 0,
                pos: Pos::new(0, 0),
            }),
            brain: Full {
                half: [0, 0],
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

fn random_squad(mut rng: &mut ChaCha8Rng, team: Team) -> Squad {
    let mut occupied = [false; WIDTH * HEIGHT];
    let mut placements = vec![];
    let mut pos;
    for _ in 0..40 {
        let template = rng.gen_range(0..NUM_TEMPLATES);
        loop {
            let y = rng.gen_range(0..HEIGHT / 2)
                + if team == Team::Red { HEIGHT / 2 } else { 0 };
            pos = Pos::new(rng.gen_range(0..WIDTH), y);
            if !occupied[pos.to_index()] {
                break;
            }
        }
        occupied[pos.to_index()] = true;
        placements.push(Placement {
            template,
            pos,
            grayed: false,
        });
    }
    for _ in 0..10 {
        let template = rng.gen_range(0..NUM_TEMPLATES);
        loop {
            let y = rng.gen_range(0..HEIGHT / 2)
                + if team == Team::Red { HEIGHT / 2 } else { 0 };
            pos = Pos::new(rng.gen_range(0..WIDTH), y);
            if !occupied[pos.to_index()] {
                break;
            }
        }
        occupied[pos.to_index()] = true;
        placements.push(Placement {
            template,
            pos,
            grayed: true,
        });
    }
    Squad {
        codes: std::array::from_fn(|_| None),
        templates: std::array::from_fn(|_| Some(random_entity(&mut rng))),
        placements,
    }
}

fn main() {
    let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(17).try_into().unwrap();
    let initial_state = build_state(
        random_squad(&mut rng, Team::Blue),
        random_squad(&mut rng, Team::Red),
        Settings {
            min_tokens: 15,
            tiles: (0..(WIDTH * HEIGHT))
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
        },
    )
    .unwrap();
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
