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

pub struct Pos {
  pub x: usize,
  pub y: usize,
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

pub struct ViewingTile {}
