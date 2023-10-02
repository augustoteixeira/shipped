use super::encoder::Verb;
use super::game::Direction;

pub const GO_EAST: Verb = Verb::AttemptMove(Direction::East);
pub const GO_WEST: Verb = Verb::AttemptMove(Direction::West);
pub const GO_NORTH: Verb = Verb::AttemptMove(Direction::North);
pub const GO_SOUTH: Verb = Verb::AttemptMove(Direction::South);
pub const WAIT: Verb = Verb::Wait;
