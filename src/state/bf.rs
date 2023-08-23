extern crate rand;
extern crate rand_chacha;

use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use crate::state::constants::{HEIGHT, NUM_TEMPLATES, WIDTH};
use crate::state::entity::{cost, FullEntity, MixEntity};
use crate::state::geometry::{board_iterator, Pos};
use crate::state::materials::Materials;
use crate::state::state::Tile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityState {
  Empty,
  Entity(MixEntity, usize),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BFState {
  pub materials: Materials,
  tokens: usize,
  pub min_tokens: usize,
  pub tiles: Vec<Tile>,
  pub entities: [EntityState; NUM_TEMPLATES],
}

#[derive(Debug, Snafu)]
pub enum ValidationError {
  #[snafu(display("Cannot remove entity from level ({:}, {:})", pos.x, pos.y))]
  RemoveEntityFromLevel { pos: Pos },
  #[snafu(display("Cannot remove material from level ({:}, {:})", pos.x, pos.y))]
  RemoveMaterialFromLevel { pos: Pos },
  #[snafu(display("Cannot delete bot {:} from level", index))]
  RemoveBotFromLevel { index: usize },
  #[snafu(display("Bot {:} needs to be compatible with level", index))]
  IncompatibleBot { index: usize },
  #[snafu(display("Not enough material"))]
  NotEnoughMaterial {},
  #[snafu(display("Not enough tokens"))]
  NotEnoughMaterialTokens {},
}

#[derive(Debug, Snafu)]
pub enum UpdateError {
  #[snafu(display("Not enough tokens {:} to remove {:}", tokens, remove))]
  NotEnoughTokens { tokens: usize, remove: usize },
  #[snafu(display("Cannot sell empty bot {:}", index))]
  CannotSellEmptyBot { index: usize },
  #[snafu(display("Cannot buy empty bot {:}", index))]
  CannotBuyEmptyBot { index: usize },
  #[snafu(display("Cannot sell with no bots of type {:}", index))]
  CannotSellWithZeroBot { index: usize },
  #[snafu(display("Not enough material to buy bot {:}", index))]
  NoMaterialToBuyBot { index: usize },
  #[snafu(display("Not enough tokens to buy bot {:}", index))]
  NoTokensToBuyBot { index: usize },
}

impl BFState {
  pub fn get_tokens(&self) -> usize {
    self.tokens
  }

  pub fn add_tokens(&mut self, other: usize) {
    self.tokens += other;
  }
  pub fn try_sub_tokens(&mut self, other: usize) -> Result<(), UpdateError> {
    if self.tokens >= other {
      self.tokens -= other;
      Ok(())
    } else {
      Err(UpdateError::NotEnoughTokens {
        tokens: self.tokens,
        remove: other,
      })
    }
  }

  pub fn sell_bot(&mut self, index: usize) -> Result<(), UpdateError> {
    match &mut self.entities[index] {
      EntityState::Empty => {
        return Err(UpdateError::CannotSellEmptyBot { index });
      }
      EntityState::Entity(e, j) => {
        if *j > 0 {
          *j -= 1;
          self.materials += cost(&FullEntity::try_from(e.clone()).unwrap());
          let tokens = e.tokens;
          self.add_tokens(tokens);
          return Ok(());
        } else {
          return Err(UpdateError::CannotSellWithZeroBot { index });
        }
      }
    }
  }

  pub fn buy_bot(&mut self, index: usize) -> Result<(), UpdateError> {
    match &mut self.entities[index] {
      EntityState::Empty => {
        return Err(UpdateError::CannotBuyEmptyBot { index });
      }
      EntityState::Entity(e, j) => {
        if !(self.materials >= cost(&FullEntity::try_from(e.clone()).unwrap())) {
          return Err(UpdateError::NoMaterialToBuyBot { index });
        } else {
          let entity = e.clone();
          if self.tokens < entity.tokens {
            return Err(UpdateError::NoTokensToBuyBot { index });
          } else {
            *j += 1;
            self.try_sub_tokens(entity.tokens)?;
            self.materials -= cost(&FullEntity::try_from(entity).unwrap());
            return Ok(());
          }
        }
      }
    }
  }

  pub fn new() -> Self {
    BFState {
      materials: Materials {
        carbon: 0,
        silicon: 0,
        plutonium: 0,
        copper: 0,
      },
      tokens: 0,
      min_tokens: 0,
      tiles: (0..(WIDTH * HEIGHT))
        .map(|_| Tile {
          entity_id: None,
          materials: Materials {
            carbon: 0,
            silicon: 0,
            plutonium: 0,
            copper: 0,
          },
        })
        .collect(),
      entities: [
        EntityState::Empty,
        EntityState::Empty,
        EntityState::Empty,
        EntityState::Empty,
      ],
    }
  }

  pub fn entity_cost(&self, i: usize) -> (Materials, usize) {
    let entity = &self.entities[i];
    match entity {
      EntityState::Empty => (
        Materials {
          carbon: 0,
          silicon: 0,
          plutonium: 0,
          copper: 0,
        },
        0,
      ),
      EntityState::Entity(e, k) => {
        let mut num_entities = *k;
        // loop through board, summing materials/entities
        for pos in board_iterator() {
          if pos.y >= HEIGHT / 2 {
            let tile_entity = self.tiles[pos.to_index()].entity_id;
            if tile_entity == Some(i) {
              num_entities += 1;
            }
          }
        }
        let mut entities_cost = cost(&FullEntity::try_from(e.clone()).unwrap());
        entities_cost.carbon *= num_entities;
        entities_cost.silicon *= num_entities;
        entities_cost.plutonium *= num_entities;
        entities_cost.copper *= num_entities;
        (entities_cost, e.tokens * num_entities)
      }
    }
  }

  pub fn cost(&self) -> (Materials, usize, [usize; NUM_TEMPLATES]) {
    let mut material_cost = self.materials.clone();
    let mut entities: [usize; 4] = [0; NUM_TEMPLATES];
    let mut tokens = self.tokens;
    // loop through board, summing materials/entities
    for pos in board_iterator() {
      if pos.y >= HEIGHT / 2 {
        let tile_entity = self.tiles[pos.to_index()].entity_id;
        if let Some(e) = tile_entity {
          entities[e] += 1;
        }
        let tile_material = &self.tiles[pos.to_index()].materials;
        material_cost += tile_material.clone();
      }
    }
    // loop through templates, summing entities costs
    for i in 0..NUM_TEMPLATES {
      let entity = &self.entities[i];
      match entity {
        EntityState::Empty => {}
        EntityState::Entity(e, k) => {
          entities[i] += *k;
          let mut entities_cost = cost(&FullEntity::try_from(e.clone()).unwrap());
          entities_cost.carbon *= entities[i];
          entities_cost.silicon *= entities[i];
          entities_cost.plutonium *= entities[i];
          entities_cost.copper *= entities[i];
          material_cost += entities_cost;
          tokens += e.tokens * entities[i];
        }
      }
    }
    (material_cost, tokens, entities)
  }

  pub fn is_compatible(&self, reference: BFState) -> Result<bool, ValidationError> {
    // verify that costs match
    let new_cost = self.cost();
    let ref_cost = reference.cost();
    if !(new_cost.0 <= ref_cost.0) {
      return Err(ValidationError::NotEnoughMaterial {});
    }
    if new_cost.1 > ref_cost.1 {
      return Err(ValidationError::NotEnoughMaterialTokens {});
    }
    for i in 0..NUM_TEMPLATES {
      if new_cost.2[i] < ref_cost.2[i] {
        return Err(ValidationError::RemoveBotFromLevel { index: i });
      }
    }
    // loop through board, verify deletions
    for pos in board_iterator() {
      let ref_entity = reference.tiles[pos.to_index()].entity_id;
      let new_entity = self.tiles[pos.to_index()].entity_id;
      if ref_entity.is_some() & (new_entity != ref_entity) {
        return Err(ValidationError::RemoveEntityFromLevel { pos });
      }
      let ref_mat = &reference.tiles[pos.to_index()].materials;
      let new_mat = &self.tiles[pos.to_index()].materials;
      if !(ref_mat <= new_mat) {
        return Err(ValidationError::RemoveMaterialFromLevel { pos });
      }
    }
    // loop through templates, verifying bots
    for i in 0..NUM_TEMPLATES {
      let new_entity = &self.entities[i];
      let ref_entity = &reference.entities[i];
      match new_entity {
        EntityState::Empty => {
          if !matches!(ref_entity, EntityState::Empty) {
            return Err(ValidationError::RemoveBotFromLevel { index: i });
          }
        }
        EntityState::Entity(e, _) => {
          if let EntityState::Entity(ref_e, _) = ref_entity {
            if !e.compatible(ref_e) {
              return Err(ValidationError::IncompatibleBot { index: i });
            }
          }
        }
      }
    }
    Ok(true)
  }
}
