use super::abbrev::{GO_EAST, GO_NORTH, GO_SOUTH, GO_WEST, WAIT};
use super::encoder::{decode_coord, decode_view, encode_displace};
use super::game::{Displace, Pos, Verb, ViewResult};
use std::cmp::Ordering;

extern "C" {
  fn get_coord() -> u32;
}

extern "C" {
  fn get_entity(_: u16) -> i64;
}

pub type MoverState = Pos;

pub struct Mover {
  pointer: *mut Pos,
}

impl Mover {
  pub fn new(pointer: *mut Pos) -> Self {
    Mover { pointer }
  }
  pub fn next(&self) -> Verb {
    let code = unsafe { get_coord() };
    let (x, y) = decode_coord(code);
    let target: Pos = unsafe { (*(self.pointer as *mut Pos)).clone() };
    match x.cmp(&target.x) {
      Ordering::Less => {
        let code = unsafe { get_entity(encode_displace(&Displace { x: 1, y: 0 })) };
        let viewed = decode_view(code);
        if let ViewResult::Empty = viewed {
          return GO_EAST;
        }
      }
      Ordering::Greater => {
        let code = unsafe { get_entity(encode_displace(&Displace { x: -1, y: 0 })) };
        let viewed = decode_view(code);
        if let ViewResult::Empty = viewed {
          return GO_WEST;
        }
      }
      Ordering::Equal => {}
    };
    match y.cmp(&target.y) {
      Ordering::Less => {
        let code = unsafe { get_entity(encode_displace(&Displace { x: 0, y: 1 })) };
        let viewed = decode_view(code);
        if let ViewResult::Empty = viewed {
          return GO_NORTH;
        }
      }
      Ordering::Greater => {
        let code = unsafe { get_entity(encode_displace(&Displace { x: 0, y: -1 })) };
        let viewed = decode_view(code);
        if let ViewResult::Empty = viewed {
          return GO_SOUTH;
        }
      }
      Ordering::Equal => {}
    };
    return WAIT;
  }
}
