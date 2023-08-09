use serde::{Deserialize, Serialize};

use super::constants::NUM_SUB_ENTITIES;
use super::geometry::Pos;
use super::materials::Materials;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MovementType {
    Still,
    Walk,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Message {
    pub emotion: usize,
    pub pos: Pos,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Abilities<T> {
    pub movement_type: MovementType,
    pub gun_damage: usize,
    pub drill_damage: usize,
    pub message: Option<Message>,
    pub brain: T,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Team {
    Blue,
    BlueGray,
    Red,
    RedGray,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entity<T> {
    pub tokens: usize,
    pub team: Team,
    pub pos: Pos,
    pub hp: usize,
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
    pub fn get_drill_damage(&self) -> Option<usize> {
        if let Some(a) = &self.abilities {
            return Some(a.drill_damage);
        } else {
            return None;
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

pub type Half = [u8; NUM_SUB_ENTITIES];

pub type Code = Vec<u8>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Full {
    pub half: Half,
    pub code_index: usize,
    pub gas: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Mix {
    Bare,
    Half(Half),
    Full(Full),
}

pub type BareEntity = Entity<()>;
pub type HalfEntity = Entity<Half>;
pub type FullEntity = Entity<Full>;
pub type MixEntity = Entity<Mix>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateEntity {
    pub hp: usize,
    pub inventory_size: usize,
    pub materials: Materials,
    pub abilities: Option<Abilities<Full>>,
}

impl TryFrom<Entity<Mix>> for Entity<Full> {
    type Error = &'static str;

    fn try_from(mix: MixEntity) -> Result<Self, Self::Error> {
        if let None = mix.abilities {
            return Ok(FullEntity {
                tokens: mix.tokens,
                team: mix.team,
                pos: mix.pos,
                hp: mix.hp,
                inventory_size: mix.inventory_size,
                materials: mix.materials,
                abilities: None,
            });
        }
        if let Some(a) = mix.abilities {
            if let Abilities {
                brain: Mix::Full(f),
                ..
            } = a
            {
                return Ok(FullEntity {
                    tokens: mix.tokens,
                    team: mix.team,
                    pos: mix.pos,
                    hp: mix.hp,
                    inventory_size: mix.inventory_size,
                    materials: mix.materials,
                    abilities: Some(Abilities {
                        movement_type: a.movement_type,
                        gun_damage: a.gun_damage,
                        drill_damage: a.drill_damage,
                        message: a.message,
                        brain: f,
                    }),
                });
            }
        }
        Err("Mix entity is not full to convert.")
    }
}

impl TemplateEntity {
    pub fn upgrade(self, tokens: usize, team: Team, pos: Pos) -> FullEntity {
        FullEntity {
            tokens,
            team,
            pos,
            hp: self.hp,
            inventory_size: self.inventory_size,
            materials: self.materials,
            abilities: self.abilities,
        }
    }
}

pub fn max_weight(body: &FullEntity) -> usize {
    let mut result = 0;
    result += body.hp;
    result += body.inventory_size;
    result
}

pub fn cost(body: &FullEntity) -> Materials {
    let w = max_weight(&body);
    let mut result = body.materials.clone();
    result.carbon += body.hp * body.hp;
    result.carbon += body.inventory_size * body.inventory_size;
    if let Some(a) = &body.abilities {
        match a.movement_type {
            MovementType::Still => {}
            MovementType::Walk => result.plutonium += w,
        }
        result.plutonium += a.drill_damage;
        result.plutonium += a.gun_damage * a.gun_damage;
        result.plutonium += a.brain.gas / 10 + 1;
    }
    // TODO remove this:
    result.carbon = 1;
    result.plutonium = 1;
    result
}
