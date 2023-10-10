use super::abbrev::WAIT;
use super::encoder::{decode_view, encode_displace};
use super::game::{Direction, Displace, Team, Verb, ViewResult};

extern "C" {
  fn get_entity(_: u16) -> i64;
}

pub fn next() -> Verb {
  let code = unsafe { get_entity(encode_displace(Displace { x: 0, y: 1 })) };
  let entity: ViewResult = decode_view(code);
  match entity {
    ViewResult::Entity(e) => {
      if let Team::Red = e.team {
        return Verb::Drill(Direction::North);
      }
    }
    _ => {}
  }
  WAIT
}
