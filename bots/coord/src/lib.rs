use tools::abbrev::{GO_EAST, GO_NORTH, GO_SOUTH, GO_WEST, WAIT};
use tools::encoder::{decode_coord, encode_verb};

extern "C" {
  fn get_coord() -> u32;
}

#[no_mangle]
pub fn execute() -> i64 {
  let code = unsafe { get_coord() };
  let (x, y) = decode_coord(code);
  match x {
    0..=28 => return encode_verb(GO_EAST),
    31..=64 => return encode_verb(GO_WEST),
    _ => match y {
      0..=48 => return encode_verb(GO_NORTH),
      51..=64 => return encode_verb(GO_SOUTH),
      _ => return encode_verb(WAIT),
    },
  }
}
