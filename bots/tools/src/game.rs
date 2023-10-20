pub struct Message {
  pub emotion: usize,
  pub pos: Pos,
}

pub struct Materials {
  pub carbon: usize,
  pub silicon: usize,
  pub plutonium: usize,
  pub copper: usize,
}

#[derive(Clone)]
pub struct Pos {
  pub x: usize,
  pub y: usize,
}

pub enum Verb {
  Wait,
  AttemptMove(Direction),
  GetMaterials(Neighbor, Materials),
  DropMaterials(Neighbor, Materials),
  Shoot(Displace),
  Drill(Direction),
  Construct(usize, Direction),
  SetMessage(Message),
}

pub enum Direction {
  North,
  East,
  South,
  West,
}

pub enum Neighbor {
  North,
  East,
  South,
  West,
  Here,
}

impl Pos {
  pub fn new(x: usize, y: usize) -> Self {
    Pos { x, y }
  }
}

#[derive(Clone)]
pub struct Displace {
  pub x: i64,
  pub y: i64,
}

impl From<Direction> for Displace {
  fn from(d: Direction) -> Self {
    match d {
      Direction::North => Displace { x: 0, y: 1 },
      Direction::East => Displace { x: 1, y: 0 },
      Direction::South => Displace { x: 0, y: -1 },
      Direction::West => Displace { x: -1, y: 0 },
    }
  }
}

impl From<Neighbor> for Displace {
  fn from(d: Neighbor) -> Self {
    match d {
      Neighbor::North => Displace { x: 0, y: 1 },
      Neighbor::East => Displace { x: 1, y: 0 },
      Neighbor::South => Displace { x: 0, y: -1 },
      Neighbor::West => Displace { x: -1, y: 0 },
      Neighbor::Here => Displace { x: 0, y: 0 },
    }
  }
}

impl Displace {
  pub fn new(x: i64, y: i64) -> Self {
    Displace { x, y }
  }
  pub fn square_norm(&self) -> i64 {
    return self.x * self.x + self.y * self.y;
  }
}

pub fn difference(p1: Pos, p2: Pos) -> Displace {
  return Displace::new((p2.x as i64) - (p1.x as i64), (p2.y as i64) - (p1.y as i64));
}

pub fn are_neighbors(p1: Pos, p2: Pos) -> bool {
  return difference(p1, p2).square_norm() == 1;
}

pub enum ViewResult {
  OutOfBounds,
  Empty,
  Entity(ViewedEntity),
  Error,
}

pub struct ViewedEntity {
  pub pos: Pos,                    // 00-15 16 bits
  pub hp: usize,                   // 16-23 8 bits
  pub gun_damage: usize,           // 24-27 4 bits
  pub drill_damage: usize,         // 28-31 4 bits
  pub team: Team,                  // 32-32 1 bit
  pub movement_type: MovementType, // 33-33 1 bit
  pub inventory_size: usize,       // 34-41 8 bits
  pub tokens: usize,               // 42-45 4 bits (total 46?)
  pub last_action: ViewAction,     // TODO: implement vieweing last action
}

pub enum Team {
  Blue,
  Red,
}

pub enum MovementType {
  Still,
  Walk,
}

pub enum ViewAction {
  Wait,
  Move(Direction),
  GetMaterials(Neighbor),
  DropMaterials(Neighbor),
  Shoot(Displace),
  Drill(Direction),
  Construct(Direction),
  SetMessage(Message),
}
