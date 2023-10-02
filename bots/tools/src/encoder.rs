use super::game::{Direction, Displace, Materials, Message, Neighbor, ViewingTile};
use std::cmp::{max, min};

pub enum Verb {
  Wait,
  AttemptMove(Direction),
  GetMaterials(Neighbor, Materials),
  DropMaterials(Neighbor, Materials),
  Shoot(Displace),
  Drill(Direction),
  Construct(usize, Direction),
  SetMessage(Message),
}

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

pub fn encode_displace(disp: Displace) -> u16 {
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
    _ => 0x0001000000000000,
  }
}

pub fn decode_tile(code: i64) -> Option<ViewingTile> {
  match code {
    0x0000000000000000 => None,
    0x0000000000000001 => Some(ViewingTile {}),
    _ => None,
  }
}
