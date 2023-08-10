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
  pub movement_type: MovementType,
  pub gun_damage: usize,
  pub drill_damage: usize,
  pub message: Option<Message>,
  pub brain: T,
}

impl<T> Entity<T> {
  pub fn can_shoot(&self) -> bool {
    self.gun_damage > 0
  }
  pub fn can_move(&self) -> bool {
    self.movement_type == MovementType::Walk
  }
  pub fn get_drill_damage(&self) -> usize {
    self.drill_damage
  }
  pub fn get_gun_damage(&self) -> usize {
    self.gun_damage
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
  pub movement_type: MovementType,
  pub gun_damage: usize,
  pub drill_damage: usize,
  pub message: Option<Message>,
  pub brain: Full,
}

impl TryFrom<Entity<Mix>> for Entity<Full> {
  type Error = &'static str;

  fn try_from(mix: MixEntity) -> Result<Self, Self::Error> {
    if let Mix::Full(f) = mix.brain {
      return Ok(FullEntity {
        tokens: mix.tokens,
        team: mix.team,
        pos: mix.pos,
        hp: mix.hp,
        inventory_size: mix.inventory_size,
        materials: mix.materials,
        movement_type: mix.movement_type,
        gun_damage: mix.gun_damage,
        drill_damage: mix.drill_damage,
        message: mix.message,
        brain: f,
      });
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
      movement_type: self.movement_type,
      gun_damage: self.gun_damage,
      drill_damage: self.drill_damage,
      message: self.message,
      brain: self.brain,
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
  match body.movement_type {
    MovementType::Still => {}
    MovementType::Walk => result.plutonium += w,
  }
  result.plutonium += body.drill_damage;
  result.plutonium += body.gun_damage * body.gun_damage;
  result.plutonium += body.brain.gas / 10 + 1;
  // TODO remove this:
  result.carbon = 1;
  result.plutonium = 1;
  result
}
