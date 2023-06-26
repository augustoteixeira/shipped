use snafu::prelude::*;

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
    Red,
}

pub struct SerialTile {
    pub materials: Materials,
    pub entity_index: Option<usize>,
}

pub struct Tile {
    pub materials: Materials,
    pub entity: Option<Box<FullEntity>>,
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
    #[snafu(display("Floor at {pos:?} does not have {payload:?}"))]
    NotEnoughMaterialsFloor { pos: Pos, payload: Materials },
    #[snafu(display("Entity at {pos:?} does not fit {payload:?}"))]
    NotEnoughSpace { pos: Pos, payload: Materials },
    #[snafu(display("Entity at {pos:?} does not have {payload:?}"))]
    NotEnoughMaterialsInEntity { pos: Pos, payload: Materials },
}

impl State {
    pub fn has_entity(&self, pos: Pos) -> bool {
        self.tiles[pos.to_index()].entity.is_some()
    }
    pub fn get_mut_entity(
        &mut self,
        pos: Pos,
    ) -> Result<&mut Box<FullEntity>, StateError> {
        self.tiles[pos.to_index()]
            .entity
            .as_mut()
            .ok_or(StateError::EmptyTile { pos: pos })
    }
    pub fn move_entity(
        &mut self,
        from: Pos,
        to: Pos,
    ) -> Result<(), StateError> {
        if !self.has_entity(from) {
            return Err(StateError::EmptyTile { pos: from });
        }
        if self.has_entity(to) {
            return Err(StateError::OccupiedTile { pos: to });
        }
        let entity = self.tiles.get_mut(from.to_index()).unwrap().entity.take();
        self.tiles[from.to_index()].entity = None;
        self.tiles[to.to_index()].entity = entity;
        Ok(())
    }
    pub fn move_material_to_entity(
        &mut self,
        from: Pos,
        to: Pos,
        payload: &Materials,
    ) -> Result<(), StateError> {
        ensure!(self.has_entity(to), EmptyTileSnafu { pos: to });
        ensure!(
            self.tiles[from.to_index()].materials.ge(payload),
            NotEnoughMaterialsFloorSnafu {
                pos: from,
                payload: payload.clone()
            }
        );
        let entity = self.get_mut_entity(to)?;
        ensure!(
            entity.inventory_size - entity.materials.volume()
                >= payload.volume(),
            NotEnoughSpaceSnafu {
                pos: to,
                payload: payload.clone()
            }
        );
        entity.materials += payload.clone();
        self.tiles[from.to_index()].materials -= payload.clone();
        Ok(())
    }
    pub fn move_material_to_floor(
        &mut self,
        from: Pos,
        to: Pos,
        payload: &Materials,
    ) -> Result<(), StateError> {
        ensure!(self.has_entity(from), EmptyTileSnafu { pos: to });
        let entity = self.get_mut_entity(to)?;
        ensure!(
            entity.inventory_size >= payload.volume(),
            NotEnoughMaterialsInEntitySnafu {
                pos: from,
                payload: payload.clone()
            }
        );
        entity.materials -= payload.clone();
        self.tiles[from.to_index()].materials += payload.clone();
        Ok(())
    }
}

// SEND TO ANOTHER FILE

#[derive(Debug)]
pub enum Event {
    EntityMove(Pos, Pos),
    AssetsFloorToEntity(Materials, Pos, Pos),
    AssetsEntityToFloor(Materials, Pos, Pos),
    Shoot(Attack),
    Drill(Attack),
    Construct(Construct),
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
    #[snafu(display("Moving {payload:?} from {from:?} to entity {to:?}"))]
    MaterialMoveToEntity {
        source: StateError,
        from: Pos,
        to: Pos,
        payload: Materials,
    },
    #[snafu(display("Moving {payload:?} from entity {from:?} to {to:?}"))]
    MaterialMoveToFloor {
        source: StateError,
        from: Pos,
        to: Pos,
        payload: Materials,
    },
}

pub fn update(state: &mut State, event: Event) -> Result<(), UpdateError> {
    match event {
        Event::EntityMove(from, to) => {
            state
                .move_entity(from, to)
                .context(EntityMoveSnafu { from, to })?;
        }
        Event::AssetsFloorToEntity(payload, from, to) => {
            state.move_material_to_entity(from, to, &payload).context(
                MaterialMoveToEntitySnafu {
                    from,
                    to,
                    payload: payload.clone(),
                },
            )?;
        }
        Event::AssetsEntityToFloor(payload, from, to) => {
            state.move_material_to_floor(from, to, &payload).context(
                MaterialMoveToFloorSnafu {
                    from,
                    to,
                    payload: payload.clone(),
                },
            )?;
        }
        //         Event::Shoot(a) => {
        //             if let Some(entity) = state.tiles[a.destination.to_index()].entity {
        //                 if entity.max_hp <= a.damage {
        //                     //state.entities[e]
        //                 }
        //             }
        //         }
        //         Event::Drill(_a) => {}
        //         Event::Construct(_c) => {}
        _ => {}
    }
    Ok(())
}
