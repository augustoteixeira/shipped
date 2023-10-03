use tools::abbrev::{GO_NORTH, GO_WEST};
use tools::encoder::{decode_tile_materials, encode_displace, encode_verb};
use tools::game::Displace;

extern "C" {
  fn get_materials(_: u16) -> i64;
}

#[no_mangle]
pub fn execute() -> i64 {
  let code = unsafe { get_materials(encode_displace(Displace { x: 0, y: 10 })) };
  let tile = decode_tile_materials(code);
  match tile {
    Some(_) => encode_verb(GO_NORTH),
    None => encode_verb(GO_WEST),
  }
}
