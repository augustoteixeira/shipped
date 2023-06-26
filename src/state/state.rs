use snafu::prelude::*;
use std::collections::HashMap;

use super::constants::{HEIGHT, NUM_CODES, NUM_SUB_ENTITIES, WIDTH};
use super::entity::{Code, FullEntity, Materials};

// https://wowpedia.fandom.com/wiki/Warcraft:_Orcs_%26_Humans_missions?file=WarCraft-Orcs%26amp%3BHumans-Orcs-Scenario9-SouthernElwynnForest.png

// pub struct Terrain {
//     pub walkable: bool,
//     pub flyable: bool,
//     pub walking_damage: usize,
//     pub flying_damage: usize,
// }

// pub type Geography = [Terrain; WIDTH * HEIGHT];

#[derive(Debug, Snafu, PartialEq, Clone, Copy)]
pub struct Pos {
    x: usize,
    y: usize,
}

impl Pos {
    fn to_index(&self) -> usize {
        self.x + self.y * WIDTH
    }
}

#[derive(Debug)]
pub enum Team {
    Blue,
    Gray,
    Red,
}

pub struct SerialTile {
    pub materials: Materials,
    pub entity_index: Option<usize>,
}

pub type Id = usize;

pub struct Tile {
    pub materials: Materials,
    pub entity_id: Option<Id>,
}

pub struct SerialState {
    pub codes: [Code; NUM_CODES],
    pub entities: Vec<FullEntity>,
    pub blue_templates: [FullEntity; NUM_SUB_ENTITIES],
    pub red_templates: [FullEntity; NUM_SUB_ENTITIES],
    pub tiles: [SerialTile; WIDTH * HEIGHT],
}

pub struct State {
    pub codes: [Code; NUM_CODES],
    pub entities: HashMap<Id, FullEntity>,
    pub blue_templates: [FullEntity; NUM_SUB_ENTITIES],
    pub red_templates: [FullEntity; NUM_SUB_ENTITIES],
    pub tiles: [Tile; WIDTH * HEIGHT],
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
}

impl State {
    pub fn has_entity(&self, pos: Pos) -> bool {
        self.tiles[pos.to_index()].entity_id.is_some()
    }
    pub fn get_tile(&self, pos: Pos) -> &Tile {
        &self.tiles[pos.to_index()]
    }
    pub fn get_floor_mat(&self, pos: Pos) -> &Materials {
        &self.tiles[pos.to_index()].materials
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
        ensure!(self.has_entity(pos), EmptyTileSnafu { pos });
        let entity = self.get_mut_entity(pos)?;
        if entity.hp > damage {
            entity.hp -= damage;
        } else {
            self.remove_entity(pos)?;
        }
        Ok(())
    }
}

// SEND TO ANOTHER FILE

#[derive(Debug)]
pub struct Message {}

#[derive(Debug)]
pub enum Event {
    EntityMove(Pos, Pos),
    AssetsFloorToEntity(Materials, Pos, Pos),
    AssetsEntityToFloor(Materials, Pos, Pos),
    Shoot(Attack),
    Drill(Attack),
    Construct(Construct),
    //Message(Pos, Team, ?!?)
}

#[derive(Debug)]
pub struct Attack {
    pub origin: Pos,
    pub destination: Pos,
    pub damage: usize,
}

#[derive(Debug)]
pub struct Construct {
    pub team: Team,
    pub builder: Pos,
    pub buildee: Pos,
}

#[derive(Debug, Snafu)]
pub enum UpdateError {
    #[snafu(display("Moving entity from {from:?} to {to:?}"))]
    EntityMove {
        source: StateError,
        from: Pos,
        to: Pos,
    },
    #[snafu(display("Moving {load:?} from {from:?} to entity {to:?}"))]
    MaterialMoveToEntity {
        source: StateError,
        from: Pos,
        to: Pos,
        load: Materials,
    },
    #[snafu(display("Moving {load:?} from entity {from:?} to {to:?}"))]
    MaterialMoveToFloor {
        source: StateError,
        from: Pos,
        to: Pos,
        load: Materials,
    },
    #[snafu(display("Attacking: {attack:?}"))]
    AttackUnit { source: StateError, attack: Attack },
}

// replay does not try to check logic (like fov). Only the basic necesary
// for its continued good behavior. the other logic was tested during the
// generation of the logs.
pub fn replay(state: &mut State, event: Event) -> Result<(), UpdateError> {
    match event {
        Event::EntityMove(from, to) => {
            state
                .move_entity(from, to)
                .context(EntityMoveSnafu { from, to })?;
        }
        Event::AssetsFloorToEntity(load, from, to) => {
            state.move_material_to_entity(from, to, &load).context(
                MaterialMoveToEntitySnafu {
                    from,
                    to,
                    load: load.clone(),
                },
            )?;
        }
        Event::AssetsEntityToFloor(load, from, to) => {
            state.move_material_to_floor(from, to, &load).context(
                MaterialMoveToFloorSnafu {
                    from,
                    to,
                    load: load.clone(),
                },
            )?;
        }
        Event::Shoot(a) => {
            state
                .attack(a.destination, a.damage)
                .context(AttackUnitSnafu { attack: a })?;
        }
        Event::Drill(a) => {
            state
                .attack(a.destination, a.damage)
                .context(AttackUnitSnafu { attack: a })?;
        }
        //         Event::Construct(_c) => {}
        //         Event::Message(_m)
        _ => {}
    }
    Ok(())
}
