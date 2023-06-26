use serde::{Deserialize, Serialize};
use snafu::prelude::*;
use std::cmp::max;
use std::collections::HashMap;

use super::constants::{HEIGHT, NUM_CODES, NUM_TEMPLATES, WIDTH};
use super::entity::{Code, FullEntity, Id, Materials, Message, Pos};

// https://wowpedia.fandom.com/wiki/Warcraft:_Orcs_%26_Humans_missions?file=WarCraft-Orcs%26amp%3BHumans-Orcs-Scenario9-SouthernElwynnForest.png

// pub struct Terrain {
//     pub walkable: bool,
//     pub flyable: bool,
//     pub walking_damage: usize,
//     pub flying_damage: usize,
// }

// pub type Geography = [Terrain; WIDTH * HEIGHT];

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Team {
    Blue,
    Gray,
    Red,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tile {
    pub materials: Materials,
    pub entity_id: Option<Id>,
}

//pub struct Tiles<T, const N: usize>(pub [T; WIDTH * HEIGHT]);
//pub struct Tiles<const N: usize>(pub [Tile; WIDTH * HEIGHT]);
#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    codes: [Code; NUM_CODES],
    entities: HashMap<Id, FullEntity>,
    next_unique_id: usize,
    blue_templates: [FullEntity; NUM_TEMPLATES],
    gray_templates: [FullEntity; NUM_TEMPLATES],
    red_templates: [FullEntity; NUM_TEMPLATES],
    #[serde_as(as = "[_; WIDTH * HEIGHT]")]
    tiles: [Tile; WIDTH * HEIGHT],
}

#[derive(Debug, Snafu)]
pub enum StateError {
    #[snafu(display("No entity in {:?}", pos))]
    EmptyTile { pos: Pos },
    #[snafu(display("Occupied tile {:?}", pos))]
    OccupiedTile { pos: Pos },
    #[snafu(display("Floor at {pos:?} does not have {load:?}"))]
    NoMaterialFloor { pos: Pos, load: Materials },
    #[snafu(display("Entity at {pos:?} does not fit {load:?}"))]
    NoSpace { pos: Pos, load: Materials },
    #[snafu(display("Entity at {pos:?} does not have {load:?}"))]
    NoMaterialEntity { pos: Pos, load: Materials },
    #[snafu(display("Template index out of bounds {template}"))]
    TemplateOutOfBounds { template: usize },
    #[snafu(display("Entity in {pos} has no abilities"))]
    NoAbilities { pos: Pos },
}

impl State {
    pub fn new(
        codes: [Code; NUM_CODES],
        entities: HashMap<Id, FullEntity>,
        blue_templates: [FullEntity; NUM_TEMPLATES],
        gray_templates: [FullEntity; NUM_TEMPLATES],
        red_templates: [FullEntity; NUM_TEMPLATES],
        tiles: [Tile; WIDTH * HEIGHT],
    ) -> Self {
        let next_unique_id = entities.iter().fold(0, |a, (id, _)| max(a, *id));
        State {
            codes,
            entities,
            next_unique_id,
            blue_templates,
            gray_templates,
            red_templates,
            tiles,
        }
    }
    pub fn has_entity(&self, pos: Pos) -> bool {
        self.tiles[pos.to_index()].entity_id.is_some()
    }
    pub fn get_tile(&self, pos: Pos) -> &Tile {
        &self.tiles[pos.to_index()]
    }
    pub fn get_floor_mat(&self, pos: Pos) -> &Materials {
        &self.tiles[pos.to_index()].materials
    }
    pub fn build_entity_from_template(
        &mut self,
        team: Team,
        template: usize,
        pos: Pos,
    ) -> Result<(), StateError> {
        ensure!(!self.has_entity(pos), OccupiedTileSnafu { pos });
        ensure!(
            template < NUM_TEMPLATES,
            TemplateOutOfBoundsSnafu { template }
        );
        let mut entity = match team {
            Team::Blue => self.blue_templates[template].clone(),
            Team::Gray => self.gray_templates[template].clone(),
            Team::Red => self.red_templates[template].clone(),
        };
        entity.pos = pos;
        self.entities.insert(self.next_unique_id, entity);
        self.next_unique_id += 1;
        Ok(())
    }
    pub fn remove_entity(&mut self, pos: Pos) -> Result<(), StateError> {
        let id = self.tiles[pos.to_index()]
            .entity_id
            .ok_or(StateError::EmptyTile { pos: pos })?;
        self.entities.remove(&id);
        self.tiles[pos.to_index()].entity_id = None;
        Ok(())
    }
    pub fn get_mut_entity(
        &mut self,
        pos: Pos,
    ) -> Result<&mut FullEntity, StateError> {
        let id = self
            .get_tile(pos)
            .entity_id
            .ok_or(StateError::EmptyTile { pos: pos })?;
        Ok(self.entities.get_mut(&id).unwrap())
    }
    pub fn move_entity(
        &mut self,
        from: Pos,
        to: Pos,
    ) -> Result<(), StateError> {
        ensure!(self.has_entity(from), EmptyTileSnafu { pos: from });
        ensure!(!self.has_entity(to), OccupiedTileSnafu { pos: to });
        let id = self.tiles[from.to_index()].entity_id.unwrap();
        let entity = self.get_mut_entity(from).unwrap();
        entity.pos = to;
        self.tiles[from.to_index()].entity_id = None;
        self.tiles[to.to_index()].entity_id = Some(id);
        Ok(())
    }
    pub fn move_material_to_entity(
        &mut self,
        from: Pos,
        to: Pos,
        load: &Materials,
    ) -> Result<(), StateError> {
        ensure!(self.has_entity(to), EmptyTileSnafu { pos: to });
        ensure!(
            self.get_floor_mat(from).ge(load),
            NoMaterialFloorSnafu {
                pos: from,
                load: load.clone()
            }
        );
        let entity = self.get_mut_entity(to)?;
        ensure!(
            entity.inventory_size >= entity.materials.volume() + load.volume(),
            NoSpaceSnafu {
                pos: to,
                load: load.clone()
            }
        );
        entity.materials += load.clone();
        self.tiles[from.to_index()].materials -= load.clone();
        Ok(())
    }
    pub fn move_material_to_floor(
        &mut self,
        from: Pos,
        to: Pos,
        load: &Materials,
    ) -> Result<(), StateError> {
        ensure!(self.has_entity(from), EmptyTileSnafu { pos: from });
        let entity = self.get_mut_entity(to)?;
        ensure!(
            entity.inventory_size >= load.volume(),
            NoMaterialEntitySnafu {
                pos: from,
                load: load.clone()
            }
        );
        entity.materials -= load.clone();
        self.tiles[from.to_index()].materials += load.clone();
        Ok(())
    }
    pub fn attack(
        &mut self,
        pos: Pos,
        damage: usize,
    ) -> Result<(), StateError> {
        let entity = self.get_mut_entity(pos)?;
        if entity.hp > damage {
            entity.hp -= damage;
        } else {
            self.remove_entity(pos)?;
        }
        Ok(())
    }
    pub fn set_message(
        &mut self,
        pos: Pos,
        message: &Message,
    ) -> Result<(), StateError> {
        let entity = self.get_mut_entity(pos)?;
        let abilities = entity
            .abilities
            .as_mut()
            .ok_or(StateError::NoAbilities { pos })?;
        abilities.brain.message = message.clone();
        Ok(())
    }
}
