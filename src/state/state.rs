use snafu::prelude::*;

use super::constants::{HEIGHT, NUM_SUB_ENTITIES, WIDTH};
use super::entity::{Assets, FullEntity};

// https://wowpedia.fandom.com/wiki/Warcraft:_Orcs_%26_Humans_missions?file=WarCraft-Orcs%26amp%3BHumans-Orcs-Scenario9-SouthernElwynnForest.png

// pub struct Terrain {
//     pub walkable: bool,
//     pub flyable: bool,
//     pub walking_damage: usize,
//     pub flying_damage: usize,
// }

// pub type Geography = [Terrain; WIDTH * HEIGHT];

#[derive(Debug, Snafu, PartialEq)]
pub struct Pos {
    x: usize,
    y: usize,
}

impl Pos {
    fn to_index(&self) -> usize {
        self.x + self.y * WIDTH
    }
}

pub enum Team {
    Blue,
    Red,
}

pub struct Tile {
    pub assets: Assets,
    pub entity: Option<FullEntity>,
}

pub struct State {
    pub blue_constructs: [FullEntity; NUM_SUB_ENTITIES],
    pub red_constructs: [FullEntity; NUM_SUB_ENTITIES],
    pub entities: [Tile; WIDTH * HEIGHT],
}

// SEND TO ANOTHER FILE

#[derive(Debug, Snafu, PartialEq)]
pub enum UpdateError {
    #[snafu(display(
        "Attempted to move from {:?} to existing entity {:?}",
        origin,
        dest
    ))]
    MoveToNonEmpty { origin: Pos, dest: Pos },
}

pub enum Event {
    EntityMove(Pos, Pos),
    AssetsFloorToEntity(Pos, Pos),
    AssetsEntityToFloor(Pos, Pos),
    Shot(Attack),
    Drill(Attack),
    Construct(Construct),
}

pub struct Attack {
    origin: Pos,
    destination: Pos,
    damage: usize,
}

pub struct Construct {
    team: Team,
    builder: Pos,
    buildee: Pos,
}

pub fn update(state: &State, event: Event) -> Result<(), UpdateError> {
    match event {
        Event::EntityMove(from, to) => {
            //ensure!(state.entities[from.to_index().is_some()], )
            ensure!(
                state.entities[to.to_index()].entity.is_none(),
                MoveToNonEmptySnafu {
                    origin: from,
                    dest: to
                }
            );
        }
        Event::AssetsFloorToEntity(_from, _to) => {}
        Event::AssetsEntityToFloor(_from, _to) => {}
        Event::Shot(_a) => {}
        Event::Drill(_a) => {}
        Event::Construct(_c) => {}
    }
    Ok(())
}
