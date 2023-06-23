use super::constants::NUM_SUB_ENTITIES;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MovementType {
    Still,
    Walk,
    Fly,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Assets {
    pub carbon: usize,
    pub silicon: usize,
    pub plutonium: usize,
    pub zinc: usize,
    pub ammo: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entity<T> {
    pub hp: usize,
    pub inventory_size: usize,
    pub assets: Assets,
    pub abilities: Option<Abilities<T>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Abilities<T> {
    pub movement_type: MovementType,
    pub drill_damage: usize,
    pub gun_damage: Option<usize>,
    pub brain: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Half {
    pub sub_entities: [Option<u8>; 2],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Full {
    pub sub_entities: [Option<u8>; NUM_SUB_ENTITIES],
    pub code: Vec<u8>,
    pub gas: usize,
}

pub type BareEntity = Entity<()>;
pub type HalfEntity = Entity<Half>;
pub type FullEntity = Entity<Full>;

pub fn weight(body: &Entity<Full>) -> usize {
    let mut result = 0;
    result += body.hp;
    result += body.inventory_size;
    result
}

pub fn cost(body: FullEntity) -> Assets {
    let w = weight(&body);
    let mut result = body.assets;
    result.carbon += body.hp * body.hp;
    result.carbon += body.inventory_size * body.inventory_size;
    if let Some(a) = body.abilities {
        match a.movement_type {
            MovementType::Still => {}
            MovementType::Walk => result.plutonium += w,
            MovementType::Fly => result.plutonium += w * w,
        }
        result.plutonium += a.drill_damage;
        if let Some(d) = a.gun_damage {
            result.plutonium += d * d;
        }
        result.plutonium += a.brain.code.len();
        result.plutonium += a.brain.gas / 10 + 1;
    }
    result
}
