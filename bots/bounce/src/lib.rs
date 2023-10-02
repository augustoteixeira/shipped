use tools::abbrev::{GO_NORTH, GO_WEST};
use tools::encoder::{decode_tile, encode_displace, encode_verb};
use tools::game::Displace;

extern "C" {
  fn get_tile(_: u16) -> i64;
}

#[no_mangle]
pub fn execute() -> i64 {
  let code = unsafe { get_tile(encode_displace(Displace { x: 0, y: 10 })) };
  let tile = decode_tile(code);
  match tile {
    Some(_) => encode_verb(GO_NORTH),
    None => encode_verb(GO_WEST),
  }
}
