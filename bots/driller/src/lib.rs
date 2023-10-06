use tools::abbrev::GO_NORTH;
use tools::encoder::{decode_view, encode_displace, encode_verb};
use tools::game::{Direction, Displace, Verb, ViewResult};

extern "C" {
  fn get_entity(_: u16) -> i64;
}

#[no_mangle]
pub fn execute() -> i64 {
  let code = unsafe { get_entity(encode_displace(Displace { x: 0, y: 1 })) };
  let entity: ViewResult = decode_view(code);
  match entity {
    ViewResult::Entity(_) => encode_verb(Verb::Drill(Direction::North)),
    _ => encode_verb(GO_NORTH),
  }
}
