use super::constants::{HEIGHT, WIDTH};
use std::{convert::TryFrom, num::TryFromIntError};

use serde::{Deserialize, Serialize};
use snafu::prelude::*;

#[derive(Serialize, Deserialize, Debug, Snafu, PartialEq, Clone, Copy)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

impl Pos {
    pub fn invert(&self) -> Self {
        Pos::new(WIDTH - self.x - 1, HEIGHT - self.y - 1)
    }
}

pub struct BoardIterator {
    i: usize,
    j: usize,
}

impl Iterator for BoardIterator {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        let (i, j) = (self.i, self.j);
        if i < WIDTH - 1 {
            self.i += 1;
        } else {
            self.i = 0;
            self.j += 1;
        }

        if j == HEIGHT {
            return None;
        };
        return Some(Pos::new(i, j));
    }
}

pub fn board_iterator() -> BoardIterator {
    BoardIterator { i: 0, j: 0 }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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
    pub fn to_index(&self) -> usize {
        self.x + self.y * WIDTH
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Displace {
    pub x: i64,
    pub y: i64,
}

impl From<Direction> for Displace {
    fn from(d: Direction) -> Self {
        match d {
            Direction::North => Displace { x: 0, y: -1 },
            Direction::East => Displace { x: 1, y: 0 },
            Direction::South => Displace { x: 0, y: 1 },
            Direction::West => Displace { x: -1, y: 0 },
        }
    }
}

impl From<Neighbor> for Displace {
    fn from(d: Neighbor) -> Self {
        match d {
            Neighbor::North => Displace { x: 0, y: -1 },
            Neighbor::East => Displace { x: 1, y: 0 },
            Neighbor::South => Displace { x: 0, y: 1 },
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

#[derive(Debug, Snafu)]
pub enum GeometryError {
    #[snafu(display("Displace {} by {:?} out of bounds", pos, d))]
    DisplacedOutOfBounds {
        source: TryFromIntError,
        pos: Pos,
        d: Displace,
    },
    #[snafu(display("Displace {} by {:?} out of bounds large", pos, d))]
    DisplaceOutOfBoundsLarge { pos: Pos, d: Displace },
}

pub fn add_displace(pos: Pos, disp: &Displace) -> Result<Pos, GeometryError> {
    let x = usize::try_from((pos.x as i64) + disp.x).context(
        DisplacedOutOfBoundsSnafu {
            pos,
            d: disp.clone(),
        },
    )?;
    let y = usize::try_from((pos.y as i64) + disp.y).context(
        DisplacedOutOfBoundsSnafu {
            pos,
            d: disp.clone(),
        },
    )?;
    let new_pos = Pos::new(x, y);
    ensure!(
        is_within_bounds(new_pos),
        DisplaceOutOfBoundsLargeSnafu {
            pos,
            d: disp.clone()
        }
    );
    Ok(new_pos)
}

pub fn difference(p1: Pos, p2: Pos) -> Displace {
    return Displace::new(
        (p2.x as i64) - (p1.x as i64),
        (p2.y as i64) - (p1.y as i64),
    );
}

pub fn is_within_bounds(pos: Pos) -> bool {
    return (pos.x < WIDTH) & (pos.y < HEIGHT);
}

pub fn is_within_bounds_signed(x: i64, y: i64) -> bool {
    return (x >= 0) & (x < WIDTH as i64) & (y >= 0) & (y < HEIGHT as i64);
}

pub fn are_neighbors(p1: Pos, p2: Pos) -> bool {
    return difference(p1, p2).square_norm() == 1;
}
