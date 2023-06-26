use crate::state::entity::{
    Abilities, Full, FullEntity, Materials, Message, MovementType, Pos,
};

pub mod state;

fn main() {
    println!("Hello, world!");
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
            gun_damage: Some(1),
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
    let serialized = serde_json::to_string(&entity).unwrap();
    println!("Serialized = {}", serialized);
}
