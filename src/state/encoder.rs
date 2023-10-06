use crate::state::entity::{Action, ActiveEntity, Message, MovementType, Team};
use crate::state::geometry::{Direction, Displace, Neighbor, Pos};
use crate::state::materials::Materials;
use crate::state::state::Verb;
use std::cmp::min;

fn decode_direction(code: u8) -> Option<Direction> {
  match code {
    0 => Some(Direction::North),
    1 => Some(Direction::West),
    2 => Some(Direction::East),
    3 => Some(Direction::South),
    _ => None,
  }
}

fn decode_neighbor(code: u8) -> Option<Neighbor> {
  match code {
    0 => Some(Neighbor::Here),
    1 => Some(Neighbor::North),
    2 => Some(Neighbor::West),
    3 => Some(Neighbor::East),
    4 => Some(Neighbor::South),
    _ => None,
  }
}

pub fn encode_materials(mat: Materials) -> u32 {
  let carbon: u32 = min(mat.carbon, 255).try_into().unwrap();
  let silicon: u32 = min(mat.silicon, 255).try_into().unwrap();
  let plutonium: u32 = min(mat.plutonium, 255).try_into().unwrap();
  let copper: u32 = min(mat.copper, 255).try_into().unwrap();
  (copper << 24) + (plutonium << 16) + (silicon << 8) + carbon
}

fn decode_materials(code: u32) -> Materials {
  let carbon: usize = (code & 0x000000FF).try_into().unwrap();
  let silicon: usize = ((code & 0x0000FF00) >> 8).try_into().unwrap();
  let plutonium: usize = ((code & 0x00FF0000) >> 16).try_into().unwrap();
  let copper: usize = ((code & 0xFF000000) >> 24).try_into().unwrap();
  Materials {
    carbon,
    silicon,
    plutonium,
    copper,
  }
}

pub enum ViewAction {
  Wait,
  Move(Direction),
  GetMaterials(Neighbor),
  DropMaterials(Neighbor),
  Shoot(Displace),
  Drill(Direction),
  Construct(Direction),
  SetMessage(Message),
}

pub struct ViewedEntity {
  pub pos: Pos,                    // 00-15 16 bits
  pub hp: usize,                   // 16-23 8 bits
  pub gun_damage: usize,           // 24-27 4 bits
  pub drill_damage: usize,         // 28-31 4 bits
  pub team: Team,                  // 32-32 1 bit
  pub movement_type: MovementType, // 33-33 1 bit
  pub inventory_size: usize,       // 34-41 8 bits
  pub tokens: usize,               // 42-45 4 bits (total 46?)
  pub last_action: ViewAction,     // TODO: implement vieweing last action
}

impl From<ActiveEntity> for ViewedEntity {
  fn from(entity: ActiveEntity) -> Self {
    ViewedEntity {
      tokens: entity.tokens,
      team: entity.team,
      pos: entity.pos,
      hp: entity.hp,
      inventory_size: entity.inventory_size,
      movement_type: entity.movement_type,
      gun_damage: entity.gun_damage,
      drill_damage: entity.drill_damage,
      last_action: ViewAction::Wait, // TODO: implement vieweing last action
    }
  }
}

pub enum ViewResult {
  OutOfBounds,
  Empty,
  Entity(ViewedEntity),
  Error,
}

pub fn encode_pos(pos: Pos) -> u16 {
  let xprime: u8 = pos.x.try_into().unwrap();
  let yprime: u8 = pos.y.try_into().unwrap();
  ((xprime as u16) << 8) + (yprime as u16)
}

pub fn encode_entity(entity: ViewedEntity) -> i64 {
  let mut result: i64 = 0x0000000000000000;
  result += encode_pos(entity.pos) as i64;
  result += (min(entity.hp, 255) as i64) << 16;
  result += (min(entity.gun_damage, 16) as i64) << 24;
  result += (min(entity.drill_damage, 16) as i64) << 28;
  result += (match entity.team {
    Team::Blue => 0,
    Team::Red => 1,
  } as i64)
    << 32;
  result += (match entity.movement_type {
    MovementType::Still => 0,
    MovementType::Walk => 1,
  } as i64)
    << 33;
  result += (min(entity.inventory_size, 256) as i64) << 34;
  result += (min(entity.tokens, 16) as i64) << 42;
  result
}

pub fn encode_view(view_result: ViewResult) -> i64 {
  match view_result {
    ViewResult::OutOfBounds => 0x0000000000000000,
    ViewResult::Empty => 0x0100000000000000,
    ViewResult::Entity(entity) => 0x0200000000000000 + encode_entity(entity),
    ViewResult::Error => 0x0300000000000000,
  }
}

pub fn decode_displace(code: u16) -> Displace {
  let x: u8 = ((code & 0xFF00) >> 8).try_into().unwrap();
  let y: u8 = (code & 0x00FF).try_into().unwrap();
  let signed_x = i8::from_be_bytes([x]);
  let signed_y = i8::from_be_bytes([y]);
  Displace {
    x: signed_x.into(),
    y: signed_y.into(),
  }
}

pub fn decode_verb(opcode: i64) -> Verb {
  match (opcode & 0x00FF000000000000) >> 48 {
    1 => Verb::Wait,
    2 => {
      // AttemptMove
      if let Ok(code_direction) = ((opcode & 0x0000FF0000000000) >> 40).try_into() {
        if let Some(direction) = decode_direction(code_direction) {
          return Verb::AttemptMove(direction);
        }
      }
      Verb::Wait
    }
    3 => {
      // GetMaterials
      if let Ok(code_neighbor) = ((opcode & 0x0000FF0000000000) >> 40).try_into() {
        if let Some(neighbor) = decode_neighbor(code_neighbor) {
          if let Ok(code_mat) = (opcode & 0x000000FFFFFFFF).try_into() {
            return Verb::GetMaterials(neighbor, decode_materials(code_mat));
          }
        }
      }
      Verb::Wait
    }
    4 => {
      // DropMaterials
      if let Ok(code_neighbor) = ((opcode & 0x0000FF0000000000) >> 40).try_into() {
        if let Some(neighbor) = decode_neighbor(code_neighbor) {
          if let Ok(code_mat) = ((opcode & 0x000000FFFFFFFF0000) >> 16).try_into() {
            return Verb::DropMaterials(neighbor, decode_materials(code_mat));
          }
        }
      }
      Verb::Wait
    }
    5 => {
      // Shoot
      let code_displace: u16 = ((opcode & 0x0000FFFF00000000) >> 32).try_into().unwrap();
      Verb::Shoot(decode_displace(code_displace))
    }
    6 => {
      // Drill
      if let Ok(code_direction) = ((opcode & 0x0000FF0000000000) >> 40).try_into() {
        if let Some(direction) = decode_direction(code_direction) {
          return Verb::Drill(direction);
        }
      }
      Verb::Wait
    }
    7 => {
      // Construct
      if let Ok(template) = ((opcode & 0x0000FF0000000000) >> 40).try_into() {
        if let Ok(code_direction) = ((opcode & 0x000000FF00000000) >> 32).try_into() {
          if let Some(direction) = decode_direction(code_direction) {
            return Verb::Construct(template, direction);
          }
        }
      }
      Verb::Wait
    }
    // TODO set message
    _ => Verb::Wait,
  }
}

pub fn encode_coord(x: usize, y: usize) -> u32 {
  let xprime: u16 = x.try_into().unwrap();
  let yprime: u16 = y.try_into().unwrap();
  ((xprime as u32) << 16) + (yprime as u32)
}
