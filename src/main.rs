use std::collections::HashMap;

use crate::state::constants::{HEIGHT, NUM_TEMPLATES, WIDTH};
use crate::state::entity::{
    Abilities, Full, FullEntity, Materials, Message, MovementType, Pos,
};
use crate::state::replay::{Event, Frame, Script};
use crate::state::state::{State, Team, Tile};

pub mod state;

fn main() {
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
    //let entities =
    //entities.insert(1, entity.clone());
    let template: [Option<FullEntity>; NUM_TEMPLATES] =
        std::array::from_fn(|_| Some(entity.clone()));
    let mut state = State::new(
        std::array::from_fn(|_| None),
        HashMap::new(),
        template.clone(),
        template.clone(),
        template.clone(),
        (0..(WIDTH * HEIGHT))
            .map(|_| Tile {
                entity_id: None,
                materials: Materials::new(0, 1, 2, 3),
            })
            .collect(),
    );
    state
        .build_entity_from_template(Team::Blue, 0, Pos::new(0, 0))
        .unwrap();
    state
        .build_entity_from_template(Team::Blue, 0, Pos::new(1, 1))
        .unwrap();

    let frame: Frame = vec![Event::EntityMove(Pos::new(0, 0), Pos::new(0, 1))];
    let script: Script = Script {
        genesis: state,
        frames: vec![frame],
    };
    let serialized = serde_json::to_string(&script).unwrap();
    println!("{}", serialized);
}
