use crate::state::entity::{Abilities, Assets, Full, FullEntity, MovementType};

pub mod state;

fn main() {
    println!("Hello, world!");
    let entity = FullEntity {
        hp: 3,
        inventory_size: 4,
        assets: Assets {
            carbon: 0,
            silicon: 1,
            plutonium: 23,
            zinc: 5235,
            ammo: 23,
        },
        abilities: Some(Abilities {
            movement_type: MovementType::Still,
            drill_damage: 2,
            gun_damage: Some(1),
            brain: Full {
                sub_entities: [None, None, None, None],
                code: [0, 0, 2].into(),
                gas: 2000,
            },
        }),
    };
    let serialized = serde_json::to_string(&entity).unwrap();
    println!("Serialized = {}", serialized);
}
