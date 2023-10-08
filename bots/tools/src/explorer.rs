use super::abbrev::{GO_EAST, GO_NORTH, GO_SOUTH, GO_WEST, WAIT};
use super::encoder::{decode_coord, encode_verb};
use super::game::Pos;
use std::cmp::Ordering;

extern "C" {
  fn get_coord() -> u32;
}

pub type ExplorerState = Pos;

pub struct Explorer {
  pointer: *mut Pos,
}

impl Explorer {
  pub fn new(pointer: *mut Pos) -> Self {
    Explorer { pointer }
  }
  pub fn next(&self) -> i64 {
    let code = unsafe { get_coord() };
    let (x, y) = decode_coord(code);
    let target: Pos = unsafe { (*(self.pointer as *mut Pos)).clone() };
    match x.cmp(&target.x) {
      Ordering::Less => encode_verb(GO_EAST),
      Ordering::Greater => encode_verb(GO_WEST),
      Ordering::Equal => match y.cmp(&target.y) {
        Ordering::Less => encode_verb(GO_NORTH),
        Ordering::Greater => encode_verb(GO_SOUTH),
        Ordering::Equal => encode_verb(WAIT),
      },
    }
  }
}
