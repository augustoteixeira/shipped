use tools::encoder::{decode_coord, encode_verb, Verb};
use tools::game::Direction;

extern "C" {
  fn get_coord() -> u32;
}

#[no_mangle]
pub fn execute() -> i64 {
  let code = unsafe { get_coord() };
  let (x, y) = decode_coord(code);
  match x {
    0..=28 => return encode_verb(Verb::AttemptMove(Direction::East)),
    31..=64 => return encode_verb(Verb::AttemptMove(Direction::West)),
    _ => match y {
      0..=48 => return encode_verb(Verb::AttemptMove(Direction::North)),
      51..=64 => return encode_verb(Verb::AttemptMove(Direction::South)),
      _ => return encode_verb(Verb::Wait),
    },
  }
}
