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
    #[snafu(display(
        "Point {} displaced by {:?} falls out of bounds",
        pos,
        d
    ))]
    DisplacedOutOfBounds {
        source: TryFromIntError,
        pos: Pos,
        d: Displace,
    },
}

pub fn add_displace(pos: Pos, d: &Displace) -> Result<Pos, GeometryError> {
    let x = usize::try_from((pos.x as i64) + d.x)
        .context(DisplacedOutOfBoundsSnafu { pos, d: d.clone() })?;
    let y = usize::try_from((pos.y as i64) + d.y)
        .context(DisplacedOutOfBoundsSnafu { pos, d: d.clone() })?;
    Ok(Pos::new(x, y))
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

pub fn are_neighbors(p1: Pos, p2: Pos) -> bool {
    return difference(p1, p2).square_norm() == 1;
}
