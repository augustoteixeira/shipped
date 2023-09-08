extern crate rand;
extern crate rand_chacha;

use init_array::init_array;
use serde::{Deserialize, Serialize};
use snafu::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::state::constants::{HEIGHT, NUM_TEMPLATES, WIDTH};
use crate::state::entity::{
  cost_template, ActiveEntity, Mix, MixTemplate, MovementType, Team, TemplateEntity,
};
use crate::state::geometry::{half_board_iterator, Pos};
use crate::state::materials::Materials;
use crate::state::state::{Id, State, Tile};

#[derive(Clone, Debug)]
pub enum MatName {
  Carbon,
  Silicon,
  Plutonium,
  Copper,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityState {
  Empty,
  Entity(MixTemplate, usize),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BFState {
  materials: Materials,
  tokens: usize,
  min_tokens: usize,
  tiles: Vec<Tile>,
  entities: [EntityState; NUM_TEMPLATES],
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
  #[snafu(display("Not enough tokens to validate {:}", tokens))]
  NotEnoughTokensToValidate { tokens: usize },
  #[snafu(display("Board not symmetric at {:?}", pos))]
  NotSymmetric { pos: Pos },
}

#[derive(Debug, Snafu)]
pub enum UpdateError {
  #[snafu(display("Tile {:?} is empty", pos))]
  EmptyTile { pos: Pos },
  #[snafu(display("Tile {:?} is occupied", pos))]
  TileOccupied { pos: Pos },
  #[snafu(display("No bots of type {:} owned", index))]
  NoBotsOwned { index: usize },
  #[snafu(display("Cannot add empty bot {:}", index))]
  CannotAddEmptyBot { index: usize },
  #[snafu(display("Not enough material to guarantee remainder"))]
  NotEnoughMaterialRemainder {},
  #[snafu(display("Not enough material {:?} to cover {:}", mat_name, amount))]
  NotEnoughMaterialUpdate { mat_name: MatName, amount: usize },
  #[snafu(display("Not enough tokens {:}", tokens))]
  NotEnoughTokens { tokens: usize },
  #[snafu(display("Not enough min tokens {:} to remove {:}", min_tokens, remove))]
  NotEnoughMinTokens { min_tokens: usize, remove: usize },
  #[snafu(display("Cannot sell empty bot {:}", index))]
  CannotSellEmptyBot { index: usize },
  #[snafu(display("Empty bot {:}", index))]
  EmptyBot { index: usize },
  #[snafu(display("Cannot sell with no bots of type {:}", index))]
  CannotSellWithZeroBot { index: usize },
  #[snafu(display("Not enough material to buy bot {:}", index))]
  NoMaterialToBuyBot { index: usize },
  #[snafu(display("Not enough tokens to buy bot {:}", index))]
  NoTokensToBuyBot { index: usize },
  #[snafu(display("Cannot initialize non-empty bot {:}", index))]
  InitTwice { index: usize },
  #[snafu(display("Out of bounds {:?}", pos))]
  OutOfBounds { pos: Pos },
}

pub fn load_level_file(n: usize) -> Option<BFState> {
  let path = Path::new("./levels");
  let dest_filename = format!("{:05}", n);
  let mut dest = path.join(dest_filename);
  dest.set_extension("lvl");
  if dest.exists() {
    let mut file = File::open(dest).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    Some(serde_json::from_str(&contents).unwrap())
  } else {
    None
  }
}

pub fn load_squad_file(level: usize, n: usize) -> Option<BFState> {
  let path = &Path::new("./squads/").join(format!("{:05}", level));
  if !path.exists() {
    return None;
  };
  let dest_filename = format!("{:05}", n);
  let mut dest = path.join(dest_filename);
  dest.set_extension("sqd");
  if dest.exists() {
    let mut file = File::open(dest).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    Some(serde_json::from_str(&contents).unwrap())
  } else {
    None
  }
}

pub fn join_tiles(blue: &BFState, red: &BFState) -> Vec<Tile> {
  let mut result: Vec<Tile> = Vec::with_capacity(HEIGHT * WIDTH);
  for _ in 0..HEIGHT * WIDTH {
    result.push(Tile {
      materials: Materials {
        carbon: 0,
        silicon: 0,
        plutonium: 0,
        copper: 0,
      },
      entity_id: None,
    });
  }
  for pos in half_board_iterator() {
    result[pos.to_index()] = blue.get_tiles()[pos.to_index()].clone();
    result[pos.invert().to_index()] = red.get_tiles()[pos.to_index()].clone();
  }
  result
}

// TODO: make this function return errors instead of unwrapping.
pub fn build_state(level: &BFState, blue: &BFState, red: &BFState) -> State {
  assert!(blue.is_compatible(level).unwrap());
  assert!(red.is_compatible(level).unwrap());
  let joined_tiles = join_tiles(&blue, &red);

  let mut state = State::new(
    level.min_tokens,
    init_array(|_| None),
    init_array(|_| None),
    HashMap::<Id, ActiveEntity>::new(),
    blue
      .entities
      .clone()
      .into_iter()
      .map(|es| match es {
        EntityState::Entity(e, _) => e.try_into().ok(),
        _ => None,
      })
      .collect::<Vec<Option<TemplateEntity>>>()
      .try_into()
      .unwrap(),
    red
      .entities
      .clone()
      .into_iter()
      .map(|es| match es {
        EntityState::Entity(e, _) => e.try_into().ok(),
        _ => None,
      })
      .collect::<Vec<Option<TemplateEntity>>>()
      .try_into()
      .unwrap(),
    (0..(WIDTH * HEIGHT))
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
  );
  for pos in half_board_iterator() {
    if let Some(id) = joined_tiles[pos.to_index()].entity_id {
      let blue_entity = blue.entities[id].clone();
      println!("{}", state.blue_tokens);
      match blue_entity {
        EntityState::Empty => unreachable!(),
        EntityState::Entity(e, _) => {
          state
            .build_entity_from_template(Team::Blue, e.tokens, id, pos)
            .unwrap();
        }
      }
    }
    if let Some(id) = joined_tiles[pos.invert().to_index()].entity_id {
      let red_entity = blue.entities[id].clone();
      match red_entity {
        EntityState::Empty => unreachable!(),
        EntityState::Entity(e, _) => {
          state
            .build_entity_from_template(Team::Red, e.tokens, id, pos.invert())
            .unwrap();
          if let EntityState::Entity(e, _) = &red.entities[id] {
            state.get_mut_entity(pos.invert()).unwrap().tokens = e.tokens;
          }
        }
      }
    }
  }
  state
}

impl BFState {
  pub fn get_tiles(&self) -> &Vec<Tile> {
    &self.tiles
  }

  pub fn get_entities(&self) -> &[EntityState; NUM_TEMPLATES] {
    &self.entities
  }

  pub fn get_materials(&self) -> Materials {
    self.materials.clone()
  }

  pub fn add_material(&mut self, mat_name: MatName, amount: usize) {
    match mat_name {
      MatName::Carbon => self.materials.carbon += amount,
      MatName::Silicon => self.materials.silicon += amount,
      MatName::Plutonium => self.materials.plutonium += amount,
      MatName::Copper => self.materials.copper += amount,
    }
  }

  pub fn add_materials(&mut self, materials: Materials) {
    self.materials += materials;
  }

  pub fn try_sub_materials(&mut self, materials: Materials) -> Result<(), UpdateError> {
    if !(self.materials >= materials) {
      Err(UpdateError::NotEnoughMaterialRemainder {})
    } else {
      self.materials -= materials;
      Ok(())
    }
  }

  pub fn insert_material_tile(
    &mut self,
    mat_name: MatName,
    pos: Pos,
    amount: usize,
  ) -> Result<(), UpdateError> {
    if !pos.is_withing_half() {
      return Err(UpdateError::OutOfBounds { pos });
    }
    self.try_sub_material(mat_name.clone(), amount)?;
    match mat_name {
      MatName::Carbon => {
        self.tiles[pos.to_index()].materials.carbon += amount;
      }
      MatName::Silicon => {
        self.tiles[pos.to_index()].materials.silicon += amount;
      }
      MatName::Plutonium => {
        self.tiles[pos.to_index()].materials.plutonium += amount;
      }
      MatName::Copper => {
        self.tiles[pos.to_index()].materials.copper += amount;
      }
    }
    Ok(())
  }

  pub fn erase_material_tile(&mut self, pos: Pos, remainder: Materials) -> Result<(), UpdateError> {
    if !pos.is_withing_half() {
      return Err(UpdateError::OutOfBounds { pos });
    }
    let tile = &mut self.tiles[pos.to_index()];
    if !(tile.materials >= remainder) {
      return Err(UpdateError::NotEnoughMaterialRemainder {});
    }
    let removal = tile.materials.clone() - remainder.clone();
    self.materials += removal;
    tile.materials = remainder.clone();
    Ok(())
  }

  pub fn add_bot_board(&mut self, bot_index: usize, pos: Pos) -> Result<(), UpdateError> {
    if !pos.is_withing_half() {
      return Err(UpdateError::OutOfBounds { pos });
    }
    match &mut self.entities[bot_index] {
      EntityState::Empty => {
        return Err(UpdateError::CannotAddEmptyBot { index: bot_index });
      }
      EntityState::Entity(_, k) => {
        if *k == 0 {
          return Err(UpdateError::NoBotsOwned { index: bot_index });
        } else {
          if self.tiles[pos.to_index()].entity_id.is_some() {
            return Err(UpdateError::TileOccupied { pos });
          } else {
            *k -= 1;
            self.tiles[pos.to_index()].entity_id = Some(bot_index);
          }
        }
      }
    }
    Ok(())
  }

  pub fn delete_bot(&mut self, index: usize) {
    self.entities[index] = EntityState::Empty;
  }

  pub fn initialize_bot(&mut self, index: usize) -> Result<(), UpdateError> {
    if let EntityState::Empty = self.entities[index] {
      self.entities[index] = EntityState::Entity(
        MixTemplate {
          tokens: 0,
          hp: 1,
          inventory_size: 0,
          materials: Materials {
            carbon: 0,
            silicon: 0,
            plutonium: 0,
            copper: 0,
          },
          movement_type: MovementType::Still,
          gun_damage: 0,
          drill_damage: 0,
          message: None,
          brain: Mix::Bare,
        },
        0,
      )
    } else {
      return Err(UpdateError::InitTwice { index });
    }
    Ok(())
  }

  pub fn update_bot(&mut self, index: usize, entity: MixTemplate) -> Result<(), UpdateError> {
    match &mut self.entities[index] {
      EntityState::Empty => {
        return Err(UpdateError::EmptyBot { index });
      }
      EntityState::Entity(e, _) => {
        *e = entity;
        return Ok(());
      }
    }
  }

  pub fn erase_bot_from_board(&mut self, pos: Pos) -> Result<(), UpdateError> {
    if !pos.is_withing_half() {
      return Err(UpdateError::OutOfBounds { pos });
    }
    let tile = &mut self.tiles[pos.to_index()];
    match &mut tile.entity_id {
      None => {
        return Err(UpdateError::EmptyTile { pos });
      }
      Some(i) => match &mut self.entities[*i] {
        EntityState::Empty => return Err(UpdateError::EmptyBot { index: *i }),
        EntityState::Entity(_, k) => {
          *k += 1;
          tile.entity_id = None;
        }
      },
    }
    Ok(())
  }

  pub fn try_sub_material(&mut self, mat_name: MatName, amount: usize) -> Result<(), UpdateError> {
    let material_ref = match mat_name {
      MatName::Carbon => &mut self.materials.carbon,
      MatName::Silicon => &mut self.materials.silicon,
      MatName::Plutonium => &mut self.materials.plutonium,
      MatName::Copper => &mut self.materials.copper,
    };
    if *material_ref >= amount {
      *material_ref -= amount;
      Ok(())
    } else {
      Err(UpdateError::NotEnoughMaterialUpdate { mat_name, amount })
    }
  }

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
      })
    }
  }

  pub fn get_min_tokens(&self) -> usize {
    self.min_tokens
  }

  pub fn try_add_min_tokens(&mut self, other: usize) -> Result<(), UpdateError> {
    let tokens = self.cost().1;
    if tokens >= self.min_tokens + other {
      self.min_tokens += other;
      return Ok(());
    } else {
      Err(UpdateError::NotEnoughTokens {
        tokens: self.tokens,
      })
    }
  }

  pub fn try_sub_min_tokens(&mut self, other: usize) -> Result<(), UpdateError> {
    if self.min_tokens >= other {
      self.min_tokens -= other;
      Ok(())
    } else {
      Err(UpdateError::NotEnoughMinTokens {
        min_tokens: self.min_tokens,
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
          self.materials += cost_template(&e);
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
        return Err(UpdateError::EmptyBot { index });
      }
      EntityState::Entity(e, j) => {
        if !(self.materials >= cost_template(&e)) {
          return Err(UpdateError::NoMaterialToBuyBot { index });
        } else {
          let entity = e.clone();
          if self.tokens < entity.tokens {
            return Err(UpdateError::NoTokensToBuyBot { index });
          } else {
            *j += 1;
            self.try_sub_tokens(entity.tokens)?;
            self.materials -= cost_template(&entity);
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
      tiles: (0..(WIDTH * HEIGHT / 2))
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
        for pos in half_board_iterator() {
          let tile_entity = self.tiles[pos.to_index()].entity_id;
          if tile_entity == Some(i) {
            num_entities += 1;
          }
        }
        let mut entities_cost = cost_template(&e);
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
    for pos in half_board_iterator() {
      let tile_entity = self.tiles[pos.to_index()].entity_id;
      if let Some(e) = tile_entity {
        entities[e] += 1;
      }
      let tile_material = &self.tiles[pos.to_index()].materials;
      material_cost += tile_material.clone();
    }
    // loop through templates, summing entities costs
    for i in 0..NUM_TEMPLATES {
      let entity = &self.entities[i];
      match entity {
        EntityState::Empty => {}
        EntityState::Entity(e, k) => {
          entities[i] += *k;
          let mut entities_cost = cost_template(&e);
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

  pub fn check_validity(&self) -> Result<(), ValidationError> {
    let tokens = self.cost().1;
    if tokens < self.min_tokens {
      return Err(ValidationError::NotEnoughTokensToValidate { tokens });
    } else {
      return Ok(());
    }
  }

  pub fn is_compatible(&self, reference: &BFState) -> Result<bool, ValidationError> {
    // verify that costs match
    let new_cost = self.cost();
    let ref_cost = reference.cost();
    if !(new_cost.0 <= ref_cost.0) {
      return Err(ValidationError::NotEnoughMaterial {});
    }
    if new_cost.1 > ref_cost.1 {
      return Err(ValidationError::NotEnoughTokensToValidate { tokens: new_cost.1 });
    }
    for i in 0..NUM_TEMPLATES {
      if new_cost.2[i] < ref_cost.2[i] {
        return Err(ValidationError::RemoveBotFromLevel { index: i });
      }
    }
    // loop through board, verify deletions
    for pos in half_board_iterator() {
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
