use macroquad::rand::gen_range;
use std::collections::HashMap;

use crate::state::constants::{HEIGHT, NUM_TEMPLATES, WIDTH};
use crate::state::entity::{
    Abilities, Full, FullEntity, Materials, Message, MovementType, Pos,
};
use crate::state::replay::{Event, Frame, Script};
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

fn main() {
    let mut state = State::new(
        std::array::from_fn(|_| None),
        HashMap::new(),
        std::array::from_fn(|_| Some(random_entity())),
        std::array::from_fn(|_| Some(random_entity())),
        std::array::from_fn(|_| Some(random_entity())),
        (0..(WIDTH * HEIGHT))
            .map(|_| Tile {
                entity_id: None,
                materials: Materials::new(0, 1, 2, 3),
            })
            .collect(),
    );
    for i in 0..20 {
        let _ = state.build_entity_from_template(
            Team::Blue,
            gen_range(0, NUM_TEMPLATES),
            Pos::new(gen_range(0, WIDTH), gen_range(0, HEIGHT)),
        );
    }
    let mut frames = vec![];
    for f in 1..20 {
        let mut frame = vec![];
        for e in 1..2000 {
            let pos = Pos::new(gen_range(1, 59), gen_range(1, 59));
            let new_pos = Pos::new(pos.x + 1, pos.y);
            if state.has_entity(pos) & !state.has_entity(new_pos) {
                frame.push(Event::EntityMove(pos, new_pos));
                break;
            }
        }
        frames.push(frame);
    }
    let script: Script = Script {
        genesis: state,
        frames,
    };
    let serialized = serde_json::to_string(&script).unwrap();
    println!("{}", serialized);
}
