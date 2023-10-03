use crate::state::geometry::{Direction, Displace, Neighbor};
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

pub fn decode(opcode: i64) -> Verb {
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
          if let Ok(code_mat) = ((opcode & 0x00FFFFFFFF0000) >> 16).try_into() {
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
