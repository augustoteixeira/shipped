use snafu::prelude::*;
use std::collections::HashMap;

use super::constants::{HEIGHT, NUM_CODES, NUM_SUB_ENTITIES, WIDTH};
use super::entity::{Code, FullEntity, Materials, Message, Pos};

// https://wowpedia.fandom.com/wiki/Warcraft:_Orcs_%26_Humans_missions?file=WarCraft-Orcs%26amp%3BHumans-Orcs-Scenario9-SouthernElwynnForest.png

// pub struct Terrain {
//     pub walkable: bool,
//     pub flyable: bool,
//     pub walking_damage: usize,
//     pub flying_damage: usize,
// }

// pub type Geography = [Terrain; WIDTH * HEIGHT];

#[derive(Debug, Clone, Copy)]
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

pub struct State {
    pub codes: [Code; NUM_CODES],
    entities: HashMap<Id, FullEntity>,
    next_unique_id: usize,
    blue_templates: [FullEntity; NUM_SUB_ENTITIES],
    gray_templates: [FullEntity; NUM_SUB_ENTITIES],
    red_templates: [FullEntity; NUM_SUB_ENTITIES],
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
            template < NUM_SUB_ENTITIES,
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

// SEND TO ANOTHER FILE

#[derive(Debug)]
pub enum Event {
    EntityMove(Pos, Pos),
    AssetsFloorToEntity(Materials, Pos, Pos),
    AssetsEntityToFloor(Materials, Pos, Pos),
    Shoot(Attack),
    Drill(Attack),
    Construct(Construct),
    SendMessage(Pos, Message),
}

#[derive(Debug)]
pub struct Attack {
    pub origin: Pos,
    pub destination: Pos,
    pub damage: usize,
}

#[derive(Debug, Clone)]
pub struct Construct {
    pub team: Team,
    pub template_index: usize,
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
    #[snafu(display("Construct: {construct:?}"))]
    ConstructError {
        source: StateError,
        construct: Construct,
    },
    #[snafu(display("Set message error {} in {:?}", message.emotion, message.pos))]
    SetMessageError {
        source: StateError,
        pos: Pos,
        message: Message,
    },
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
        Event::Construct(c) => {
            state
                .build_entity_from_template(c.team, c.template_index, c.buildee)
                .context(ConstructSnafu {
                    construct: c.clone(),
                })?;
        }
        Event::SendMessage(pos, message) => {
            state.set_message(pos, &message).context(SetMessageSnafu {
                pos,
                message: message.clone(),
            })?
        }
    }
    Ok(())
}
