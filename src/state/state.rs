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
    #[snafu(display("Cannot move entity from empty tile {:?}", pos))]
    MoveFromEmptyTile { pos: Pos },
    #[snafu(display("Cannot move entity to occupied tile {:?}", pos))]
    MoveToOccupiedTile { pos: Pos },
    #[snafu(display("Entity in tile {:?}", pos))]
    EntityInTile { pos: Pos },
}

impl State {
    pub fn has_entity(&self, pos: Pos) -> bool {
        self.tiles[pos.to_index()].entity.is_some()
    }
    pub fn move_entity(
        &mut self,
        from: Pos,
        to: Pos,
    ) -> Result<(), StateError> {
        if !self.has_entity(from) {
            return Err(StateError::MoveFromEmptyTile { pos: from });
        }
        if self.has_entity(to) {
            return Err(StateError::MoveToOccupiedTile { pos: to });
        }
        let entity = self.tiles.get_mut(from.to_index()).unwrap().entity.take();
        self.tiles[from.to_index()].entity = None;
        self.tiles[to.to_index()].entity = entity;
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
    #[snafu(display("Error moving materials {from:?} to {to:?}: {message}"))]
    MaterialMove { from: Pos, to: Pos, message: String },
}

pub fn update(state: &mut State, event: Event) -> Result<(), UpdateError> {
    match event {
        Event::EntityMove(from, to) => {
            state
                .move_entity(from, to)
                .context(EntityMoveSnafu { from, to })?;
        }
        //         Event::AssetsEntityToFloor(payload, from, to) => {
        //             let entity = state.tiles[from.to_index()].entity.ok_or(
        //                 UpdateError::EntityMove {
        //                     from,
        //                     to,
        //                     message: "No entity in origin".to_string(),
        //                 },
        //             )?;
        //             let assets_on_entity = entity.materials.clone();
        //             ensure!(
        //                 assets_on_entity.ge(&payload),
        //                 MaterialMoveSnafu {
        //                     from: from,
        //                     to: to,
        //                     message: format!(
        //                         "Entity has {assets_on_entity:?}, not ({payload:?})"
        //                     )
        //                 }
        //             );
        //             entity.materials -= payload.clone();
        //             state.tiles[to.to_index()].materials += payload;
        //         }
        //         Event::AssetsFloorToEntity(payload, from, to) => {
        //             let assets_on_floor =
        //                 state.tiles[from.to_index()].materials.clone();
        //             let entity = state.tiles[to.to_index()].entity.ok_or(
        //                 UpdateError::MaterialMove {
        //                     from,
        //                     to,
        //                     message: "No entity in destination".to_string(),
        //                 },
        //             )?;
        //             ensure!(
        //                 assets_on_floor.ge(&payload),
        //                 MaterialMoveSnafu {
        //                     from: from,
        //                     to: to,
        //                     message: format!("Not enough material ({payload:?}) in {assets_on_floor:?}")
        //                 }
        //             );
        //             ensure!(
        //                 payload.volume() + entity.materials.volume()
        //                     <= entity.inventory_size,
        //                 MaterialMoveSnafu {
        //                     from: from,
        //                     to: to,
        //                     message: format!("Not enough capacity for {payload:?}")
        //                 }
        //             );
        //             entity.materials += payload.clone();
        //             state.tiles[to.to_index()].materials -= payload;
        //         }
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
