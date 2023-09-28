use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Materials {
  #[serde(rename = "c")]
  pub carbon: usize,
  #[serde(rename = "s")]
  pub silicon: usize,
  #[serde(rename = "p")]
  pub plutonium: usize,
  #[serde(rename = "o")]
  pub copper: usize,
}

impl PartialOrd for Materials {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    let mut result = 0;
    match (self.carbon.cmp(&other.carbon), result) {
      (Ordering::Greater, -1) => return None,
      (Ordering::Less, 0) => result = -1,
      (Ordering::Greater, 0) => result = 1,
      (Ordering::Less, 1) => return None,
      _ => {}
    }
    match (self.silicon.cmp(&other.silicon), result) {
      (Ordering::Greater, -1) => return None,
      (Ordering::Less, 0) => result = -1,
      (Ordering::Greater, 0) => result = 1,
      (Ordering::Less, 1) => return None,
      _ => {}
    }
    match (self.plutonium.cmp(&other.plutonium), result) {
      (Ordering::Greater, -1) => return None,
      (Ordering::Less, 0) => result = -1,
      (Ordering::Greater, 0) => result = 1,
      (Ordering::Less, 1) => return None,
      _ => {}
    }
    match (self.copper.cmp(&other.copper), result) {
      (Ordering::Greater, -1) => return None,
      (Ordering::Less, 0) => result = -1,
      (Ordering::Greater, 0) => result = 1,
      (Ordering::Less, 1) => return None,
      _ => {}
    }
    return Some(result.cmp(&0));
  }
}

impl Add for Materials {
  type Output = Self;
  fn add(self, other: Self) -> Self {
    Self {
      carbon: self.carbon + other.carbon,
      silicon: self.silicon + other.silicon,
      plutonium: self.plutonium + other.plutonium,
      copper: self.copper + other.copper,
    }
  }
}

impl AddAssign for Materials {
  fn add_assign(&mut self, other: Self) {
    *self = Self {
      carbon: self.carbon + other.carbon,
      silicon: self.silicon + other.silicon,
      plutonium: self.plutonium + other.plutonium,
      copper: self.copper + other.copper,
    };
  }
}

impl Sub for Materials {
  type Output = Self;
  fn sub(self, other: Self) -> Self {
    Self {
      carbon: self.carbon - other.carbon,
      silicon: self.silicon - other.silicon,
      plutonium: self.plutonium - other.plutonium,
      copper: self.copper - other.copper,
    }
  }
}

impl SubAssign for Materials {
  fn sub_assign(&mut self, other: Self) {
    *self = Self {
      carbon: self.carbon - other.carbon,
      silicon: self.silicon - other.silicon,
      plutonium: self.plutonium - other.plutonium,
      copper: self.copper - other.copper,
    };
  }
}

impl Materials {
  pub fn new(carbon: usize, silicon: usize, plutonium: usize, copper: usize) -> Self {
    Materials {
      carbon,
      silicon,
      plutonium,
      copper,
    }
  }
  pub fn volume(&self) -> usize {
    self.carbon + self.silicon + self.plutonium + self.copper
  }
}
