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

pub type EntityArray = Vec<FullEntity>;

pub struct Tile {
    pub materials: Materials,
    pub entity_index: Option<usize>,
}

pub struct State {
    pub codes: [Code; NUM_CODES],
    pub entities: EntityArray,
    pub blue_templates: [FullEntity; NUM_SUB_ENTITIES],
    pub red_templates: [FullEntity; NUM_SUB_ENTITIES],
    pub tiles: [Tile; WIDTH * HEIGHT],
}

// SEND TO ANOTHER FILE

#[derive(Debug, Snafu)]
pub enum StateError {
    #[snafu(display("No entity in tile {:?}", pos))]
    NoEntityInTile { pos: Pos },
    #[snafu(display("Entity in tile {:?}", pos))]
    EntityInTile { pos: Pos },
}

#[derive(Debug)]
pub enum Event {
    EntityMove(Pos, Pos),
    AssetsFloorToEntity(Materials, Pos, Pos),
    AssetsEntityToFloor(Materials, Pos, Pos),
    Shot(Attack),
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
    #[snafu(display("Moving entity from {from:?} to {to:?}: {message}"))]
    EntityMove { from: Pos, to: Pos, message: String },
    #[snafu(display("Error moving materials {from:?} to {to:?}: {message}"))]
    MaterialMove { from: Pos, to: Pos, message: String },
}

pub fn update(state: &mut State, event: Event) -> Result<(), UpdateError> {
    match event {
        Event::EntityMove(from, to) => {
            let entity: usize = state.tiles[from.to_index()]
                .entity_index
                .ok_or(UpdateError::EntityMove {
                    from,
                    to,
                    message: "No entity in origin".to_string(),
                })?;
            if let None = state.tiles[to.to_index()].entity_index {
                return Err(UpdateError::EntityMove {
                    from,
                    to,
                    message: "Moving to occupied tile".to_string(),
                });
            }
            state.tiles[to.to_index()].entity_index = Some(entity);
            state.tiles[from.to_index()].entity_index = None;
        }
        Event::AssetsEntityToFloor(payload, from, to) => {
            let entity: usize = state.tiles[from.to_index()]
                .entity_index
                .ok_or(UpdateError::EntityMove {
                    from,
                    to,
                    message: "No entity in origin".to_string(),
                })?;
            let assets_on_entity = state.entities[entity].materials.clone();
            ensure!(
                assets_on_entity.ge(&payload),
                MaterialMoveSnafu {
                    from: from,
                    to: to,
                    message: format!(
                        "Entity has {assets_on_entity:?}, not ({payload:?})"
                    )
                }
            );
            state.entities[entity].materials -= payload.clone();
            state.tiles[to.to_index()].materials += payload;
        }
        Event::AssetsFloorToEntity(payload, from, to) => {
            let assets_on_floor =
                state.tiles[from.to_index()].materials.clone();
            let entity: usize = state.tiles[to.to_index()].entity_index.ok_or(
                UpdateError::MaterialMove {
                    from,
                    to,
                    message: "No entity in destination".to_string(),
                },
            )?;
            ensure!(
                assets_on_floor.ge(&payload),
                MaterialMoveSnafu {
                    from: from,
                    to: to,
                    message: format!("Not enough material ({payload:?}) in {assets_on_floor:?}")
                }
            );
            ensure!(
                payload.volume() + state.entities[entity].materials.volume()
                    <= state.entities[entity].inventory_size,
                MaterialMoveSnafu {
                    from: from,
                    to: to,
                    message: format!("Not enough capacity for {payload:?}")
                }
            );
            state.entities[entity].materials += payload.clone();
            state.tiles[to.to_index()].materials -= payload;
        }
        Event::Shot(_a) => {}
        Event::Drill(_a) => {}
        Event::Construct(_c) => {}
    }
    Ok(())
}
