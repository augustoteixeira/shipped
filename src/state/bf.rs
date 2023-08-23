extern crate rand;
extern crate rand_chacha;

use serde::{Deserialize, Serialize};

use crate::state::constants::{HEIGHT, NUM_TEMPLATES};
use crate::state::entity::{cost, FullEntity, MixEntity};
use crate::state::geometry::board_iterator;
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
  pub tokens: usize,
  pub min_tokens: usize,
  pub tiles: Vec<Tile>,
  pub entities: [EntityState; NUM_TEMPLATES],
}

impl BFState {
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
}
