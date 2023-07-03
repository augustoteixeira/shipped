use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::constants::NUM_SUB_ENTITIES;
use super::geometry::Pos;

pub type Id = usize;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MovementType {
    Still,
    Walk,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Materials {
    #[serde(rename = "c")]
    pub carbon: usize,
    #[serde(rename = "s")]
    pub silicon: usize,
    #[serde(rename = "p")]
    pub plutonium: usize,
    #[serde(rename = "o")]
    pub copper: usize,
}

impl PartialOrd for Materials {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut result = 0;
        match (self.carbon.cmp(&other.carbon), result) {
            (Ordering::Greater, -1) => return None,
            (Ordering::Less, 0) => result = -1,
            (Ordering::Greater, 0) => result = 1,
            (Ordering::Less, 1) => return None,
            _ => {}
        }
        match (self.silicon.cmp(&other.silicon), result) {
            (Ordering::Greater, -1) => return None,
            (Ordering::Less, 0) => result = -1,
            (Ordering::Greater, 0) => result = 1,
            (Ordering::Less, 1) => return None,
            _ => {}
        }
        match (self.plutonium.cmp(&other.plutonium), result) {
            (Ordering::Greater, -1) => return None,
            (Ordering::Less, 0) => result = -1,
            (Ordering::Greater, 0) => result = 1,
            (Ordering::Less, 1) => return None,
            _ => {}
        }
        match (self.copper.cmp(&other.copper), result) {
            (Ordering::Greater, -1) => return None,
            (Ordering::Less, 0) => result = -1,
            (Ordering::Greater, 0) => result = 1,
            (Ordering::Less, 1) => return None,
            _ => {}
        }
        return Some(result.cmp(&0));
    }
}

impl Add for Materials {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            carbon: self.carbon + other.carbon,
            silicon: self.silicon + other.silicon,
            plutonium: self.plutonium + other.plutonium,
            copper: self.copper + other.copper,
        }
    }
}

impl AddAssign for Materials {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            carbon: self.carbon + other.carbon,
            silicon: self.silicon + other.silicon,
            plutonium: self.plutonium + other.plutonium,
            copper: self.copper + other.copper,
        };
    }
}

impl Sub for Materials {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            carbon: self.carbon - other.carbon,
            silicon: self.silicon - other.silicon,
            plutonium: self.plutonium - other.plutonium,
            copper: self.copper - other.copper,
        }
    }
}

impl SubAssign for Materials {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            carbon: self.carbon - other.carbon,
            silicon: self.silicon - other.silicon,
            plutonium: self.plutonium - other.plutonium,
            copper: self.copper - other.copper,
        };
    }
}

impl Materials {
    pub fn new(
        carbon: usize,
        silicon: usize,
        plutonium: usize,
        copper: usize,
    ) -> Self {
        Materials {
            carbon,
            silicon,
            plutonium,
            copper,
        }
    }
    pub fn volume(&self) -> usize {
        self.carbon + self.silicon + self.plutonium + self.copper
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Abilities<T> {
    pub movement_type: MovementType,
    pub drill_damage: usize,
    pub gun_damage: usize,
    pub brain: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entity<T> {
    pub pos: Pos,
    pub hp: usize,
    pub max_hp: usize,
    pub inventory_size: usize,
    pub materials: Materials,
    pub abilities: Option<Abilities<T>>,
}

impl<T> Entity<T> {
    pub fn has_ability(&self) -> bool {
        self.abilities.is_some()
    }
    pub fn can_shoot(&self) -> bool {
        if let Some(a) = &self.abilities {
            return a.gun_damage > 0;
        } else {
            return false;
        }
    }
    pub fn can_move(&self) -> bool {
        if let Some(a) = &self.abilities {
            return a.movement_type == MovementType::Walk;
        } else {
            return false;
        }
    }
    pub fn get_gun_damage(&self) -> Option<usize> {
        if let Some(a) = &self.abilities {
            return Some(a.gun_damage);
        } else {
            return None;
        }
    }
    pub fn has_copper(&self) -> bool {
        self.materials.copper > 0
    }
}

pub type Half = [Option<u8>; NUM_SUB_ENTITIES];

pub type Code = Vec<u8>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Message {
    pub emotion: usize,
    pub pos: Pos,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Full {
    pub half: Half,
    pub message: Option<Message>,
    pub code_index: usize,
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

pub fn cost(body: FullEntity) -> Materials {
    let w = weight(&body);
    let mut result = body.materials;
    result.carbon += body.hp * body.hp;
    result.carbon += body.inventory_size * body.inventory_size;
    if let Some(a) = body.abilities {
        match a.movement_type {
            MovementType::Still => {}
            MovementType::Walk => result.plutonium += w,
        }
        result.plutonium += a.drill_damage;
        result.plutonium += a.gun_damage * a.gun_damage;
        result.plutonium += a.brain.gas / 10 + 1;
    }
    result
}
