use super::abbrev::WAIT;
use super::encoder::{decode_view, encode_displace};
use super::game::{Direction, Displace, Team, Verb, ViewResult};

extern "C" {
  fn get_entity(_: u16) -> i64;
}

pub fn next() -> Verb {
  for disp in [
    Displace { x: 0, y: 1 },
    Displace { x: 0, y: -1 },
    Displace { x: 1, y: 0 },
    Displace { x: -1, y: 0 },
  ] {
    let code = unsafe { get_entity(encode_displace(&disp)) };
    let entity: ViewResult = decode_view(code);
    match entity {
      ViewResult::Entity(e) => {
        if let Team::Red = e.team {
          return match disp {
            Displace { x: 1, y: 0 } => Verb::Drill(Direction::East),
            Displace { x: -1, y: 0 } => Verb::Drill(Direction::West),
            Displace { x: 0, y: 1 } => Verb::Drill(Direction::North),
            Displace { x: 0, y: -1 } => Verb::Drill(Direction::South),
            _ => WAIT,
          };
        }
      }
      _ => {}
    }
  }
  WAIT
}
