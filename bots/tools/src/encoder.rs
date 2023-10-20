use super::game::{
  Direction, Displace, Materials, MovementType, Neighbor, Pos, Team, Verb, ViewAction, ViewResult,
  ViewedEntity,
};
use std::cmp::{max, min};

pub fn decode_coord(code: u32) -> (usize, usize) {
  let x = (code >> 16) as usize;
  let y = (code & 0x0000FFFF) as usize;
  (x, y)
}

pub fn encode_direction(dir: Direction) -> u8 {
  match dir {
    Direction::North => 0,
    Direction::West => 1,
    Direction::East => 2,
    Direction::South => 3,
  }
}

pub fn encode_neighbor(n: Neighbor) -> u8 {
  match n {
    Neighbor::Here => 0,
    Neighbor::North => 1,
    Neighbor::West => 2,
    Neighbor::East => 3,
    Neighbor::South => 4,
  }
}

pub fn encode_materials(mat: Materials) -> u32 {
  let carbon: u32 = min(mat.carbon, 255).try_into().unwrap();
  let silicon: u32 = min(mat.silicon, 255).try_into().unwrap();
  let plutonium: u32 = min(mat.plutonium, 255).try_into().unwrap();
  let copper: u32 = min(mat.copper, 255).try_into().unwrap();
  (copper << 24) + (plutonium << 16) + (silicon << 8) + carbon
}

pub fn encode_displace(disp: &Displace) -> u16 {
  let signed_x: i8 = min(max(disp.x, -127), 127).try_into().unwrap();
  let signed_y: i8 = min(max(disp.y, -127), 127).try_into().unwrap();
  let x: u16 = signed_x.to_be_bytes()[0] as u16;
  let y: u16 = signed_y.to_be_bytes()[0] as u16;
  (x << 8) + y
}

pub fn encode_verb(verb: Verb) -> i64 {
  match verb {
    Verb::Wait => 0x0001000000000000,
    Verb::AttemptMove(dir) => {
      let dir_code = encode_direction(dir) as i64;
      0x0002000000000000 + (dir_code << 40)
    }
    Verb::GetMaterials(neigh, mat) => {
      let neigh_code = encode_neighbor(neigh);
      let mat_code = encode_materials(mat);
      0x0003000000000000 + ((neigh_code as i64) << 40) + (mat_code as i64)
    }
    Verb::DropMaterials(neigh, mat) => {
      let neigh_code = encode_neighbor(neigh);
      let mat_code = encode_materials(mat);
      0x0004000000000000 + ((neigh_code as i64) << 40) + (mat_code as i64)
    }
    Verb::Shoot(displ) => {
      let displ_code = encode_displace(&displ);
      0x0005000000000000 + ((displ_code as i64) << 32)
    }
    Verb::Drill(dir) => {
      let dir_code = encode_direction(dir) as i64;
      0x0006000000000000 + (dir_code << 40)
    }
    Verb::Construct(template, direction) => {
      let template_code: u8 = template.try_into().unwrap();
      let dir_code = encode_direction(direction) as i64;
      0x0007000000000000 + ((template_code as i64) << 40) + (dir_code << 32)
    }
    _ => 0x0001000000000000,
  }
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

pub fn decode_tile_materials(code: i64) -> Option<Materials> {
  match (code & 0x00FF000000000000) >> 48 {
    0x00 => None,
    0x01 => Some(decode_materials(
      (code & 0x00000000FFFFFFFF).try_into().unwrap(),
    )),
    _ => None,
  }
}

pub fn decode_pos(code: u16) -> Pos {
  Pos {
    x: (code & 0x00FF).try_into().unwrap(),
    y: (code << 8).try_into().unwrap(),
  }
}

pub fn decode_view(code: i64) -> ViewResult {
  match (code & 0x0F00000000000000) >> 56 {
    0 => ViewResult::OutOfBounds,
    1 => ViewResult::Empty,
    2 => ViewResult::Entity(decode_entity(code & 0x0000FFFFFFFFFFFF)),
    _ => ViewResult::Error,
  }
}

pub fn decode_entity(code: i64) -> ViewedEntity {
  let pos = decode_pos((code & 0x000000000000FFFF) as u16);
  let hp: usize = ((code & 0x0000000000FF0000) >> 16) as usize;
  let gun_damage: usize = ((code & 0x000000000F000000) >> 24) as usize;
  let drill_damage: usize = ((code & 0x00000000F0000000) >> 28) as usize;
  let team: Team = match code & 0x0000000100000000 {
    0 => Team::Blue,
    _ => Team::Red,
  };
  let movement_type: MovementType = match code & 0x0000000200000000 {
    0 => MovementType::Still,
    _ => MovementType::Walk,
  };
  let inventory_size: usize = ((code & 0x0000003FC0000000) >> 34) as usize;
  let tokens: usize = ((code & 0x000003C000000000) >> 42) as usize;
  ViewedEntity {
    tokens,
    team,
    pos,
    hp,
    inventory_size,
    movement_type,
    gun_damage,
    drill_damage,
    last_action: ViewAction::Wait, // TODO: implement view last action
  }
}
