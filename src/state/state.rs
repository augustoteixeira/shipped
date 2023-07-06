use line_drawing::Bresenham;
use serde::{Deserialize, Serialize};
use snafu::prelude::*;
use std::cmp::max;
use std::collections::HashMap;

use super::constants::{HEIGHT, NUM_CODES, NUM_TEMPLATES, WIDTH};
use super::entity::{cost, Code, FullEntity, Id, Materials, Message, Team};
use super::geometry::{
    add_displace, is_within_bounds_signed, Direction, Displace, GeometryError,
    Neighbor, Pos,
};
use super::replay::{Construct, Effect};

// https://wowpedia.fandom.com/wiki/Warcraft:_Orcs_%26_Humans_missions?file=WarCraft-Orcs%26amp%3BHumans-Orcs-Scenario9-SouthernElwynnForest.png

// pub struct Terrain {
//     pub walkable: bool,
//     pub flyable: bool,
//     pub walking_damage: usize,
//     pub flying_damage: usize,
// }

// pub type Geography = [Terrain; WIDTH * HEIGHT];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tile {
    pub materials: Materials,
    pub entity_id: Option<Id>,
}

//pub struct Tiles<T, const N: usize>(pub [T; WIDTH * HEIGHT]);
//pub struct Tiles<const N: usize>(pub [Tile; WIDTH * HEIGHT]);
//#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    codes: [Option<Code>; NUM_CODES],
    pub entities: HashMap<Id, FullEntity>,
    next_unique_id: usize,
    blue_templates: [Option<FullEntity>; NUM_TEMPLATES],
    gray_templates: [Option<FullEntity>; NUM_TEMPLATES],
    red_templates: [Option<FullEntity>; NUM_TEMPLATES],
    pub tiles: Vec<Tile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub entity_id: usize,
    pub verb: Verb,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Verb {
    Wait,
    AttemptMove(Direction),
    GetMaterials(Neighbor, Materials),
    DropMaterials(Neighbor, Materials),
    Shoot(Displace),
    Drill(Direction),
    Construct(usize, Direction),
    SetMessage(Option<Message>),
}

#[derive(Debug, Snafu)]
pub enum StateError {
    #[snafu(display("Displace {:?} from {:?} out of bounds", disp, pos))]
    DisplaceOutOfBounds {
        source: GeometryError,
        pos: Pos,
        disp: Displace,
    },
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
    #[snafu(display("Entity in {pos} cannot shoot"))]
    NoShoot { pos: Pos },
    #[snafu(display("Entity in {pos} has no copper"))]
    NoCopper { pos: Pos },
    #[snafu(display("Entity in {pos} cannot walk"))]
    NoWalk { pos: Pos },
    #[snafu(display("Displacement {:?} too far", disp))]
    DisplaceTooFar { disp: Displace },
    #[snafu(display("No entity in {team:?} with template{template}"))]
    NoTemplate { team: Team, template: usize },
    #[snafu(display("Error implementing effect {:?}", effect))]
    ImplementationError { effect: Effect },
    #[snafu(display("No entity with id {id}"))]
    NoEntityWithId { id: Id },
    #[snafu(display("Cannot see from {:?} to {:?}", pos, disp))]
    NotVisible { pos: Pos, disp: Displace },
}

impl State {
    pub fn new(
        codes: [Option<Code>; NUM_CODES],
        entities: HashMap<Id, FullEntity>,
        blue_templates: [Option<FullEntity>; NUM_TEMPLATES],
        gray_templates: [Option<FullEntity>; NUM_TEMPLATES],
        red_templates: [Option<FullEntity>; NUM_TEMPLATES],
        tiles: Vec<Tile>,
    ) -> Self {
        assert!(tiles.len() == WIDTH * HEIGHT);
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
    pub fn get_creature(
        &self,
        team: Team,
        template: usize,
    ) -> Result<FullEntity, StateError> {
        ensure!(
            template < NUM_TEMPLATES,
            TemplateOutOfBoundsSnafu { template }
        );
        match team {
            Team::Blue => self.blue_templates[template].clone(),
            Team::Gray => self.gray_templates[template].clone(),
            Team::Red => self.red_templates[template].clone(),
        }
        .ok_or(StateError::NoTemplate { team, template })
    }
    pub fn build_entity_from_template(
        &mut self,
        team: Team,
        template: usize,
        pos: Pos,
    ) -> Result<(), StateError> {
        ensure!(!self.has_entity(pos), OccupiedTileSnafu { pos });
        let mut entity = self.get_creature(team, template)?;
        entity.pos = pos;
        self.entities.insert(self.next_unique_id, entity);
        self.tiles[pos.to_index()].entity_id = Some(self.next_unique_id);
        self.next_unique_id += 1;
        Ok(())
    }
    // pub fn construct_creature(
    //     &mut self,
    //     from: Pos,
    //     template: usize,
    //     dir: Direction,
    // ) -> Result<(), StateError> {
    //     //IMPLEMENT_MATERIAL_SUBTRACTION!!!
    //     Ok(())
    // }
    pub fn remove_entity(&mut self, pos: Pos) -> Result<(), StateError> {
        let id = self.tiles[pos.to_index()]
            .entity_id
            .ok_or(StateError::EmptyTile { pos })?;
        self.entities.remove(&id);
        self.tiles[pos.to_index()].entity_id = None;
        Ok(())
    }
    pub fn get_entity(&self, pos: Pos) -> Result<&FullEntity, StateError> {
        let id = self
            .get_tile(pos)
            .entity_id
            .ok_or(StateError::EmptyTile { pos })?;
        Ok(self.entities.get(&id).unwrap())
    }
    pub fn get_entity_by_id(&self, id: Id) -> Result<&FullEntity, StateError> {
        self.entities
            .get(&id)
            .ok_or(StateError::NoEntityWithId { id })
    }
    pub fn get_mut_entity_by_id(
        &mut self,
        id: Id,
    ) -> Result<&mut FullEntity, StateError> {
        self.entities
            .get_mut(&id)
            .ok_or(StateError::NoEntityWithId { id })
    }
    pub fn get_entity_option(&self, pos: Pos) -> Option<&FullEntity> {
        let id = self.get_tile(pos).entity_id;
        match id {
            None => None,
            Some(i) => Some(self.entities.get(&i).unwrap()),
        }
    }
    pub fn get_mut_entity(
        &mut self,
        pos: Pos,
    ) -> Result<&mut FullEntity, StateError> {
        let id = self
            .get_tile(pos)
            .entity_id
            .ok_or(StateError::EmptyTile { pos })?;
        Ok(self.entities.get_mut(&id).unwrap())
    }
    pub fn get_entities_ids(&self) -> Vec<Id> {
        self.entities.keys().map(|x| *x).collect()
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
    pub fn get_visible(&self, from: Pos, disp: &Displace) -> Option<Pos> {
        let point_from = (from.x as i64, from.y as i64);
        let point_to = (from.x as i64 + disp.x, from.y as i64 + disp.y);
        for (x, y) in Bresenham::new(point_from, point_to).skip(1) {
            if !is_within_bounds_signed(x, y) {
                return None;
            }
            if self.has_entity(Pos::new(x as usize, y as usize)) {
                return Some(Pos::new(x as usize, y as usize));
            }
        }
        Some(Pos::new(point_to.0 as usize, point_to.1 as usize))
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
        let entity = self.get_mut_entity(from)?;
        ensure!(
            entity.materials >= *load,
            NoMaterialEntitySnafu {
                pos: from,
                load: load.clone()
            }
        );
        entity.materials -= load.clone();
        self.tiles[to.to_index()].materials += load.clone();
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
        message: Option<Message>,
    ) -> Result<(), StateError> {
        let entity = self.get_mut_entity(pos)?;
        let abilities = entity
            .abilities
            .as_mut()
            .ok_or(StateError::NoAbilities { pos })?;
        abilities.brain.message = message.clone();
        Ok(())
    }
    pub fn add_displace(pos: Pos, disp: &Displace) -> Result<Pos, StateError> {
        add_displace(pos, disp).context(DisplaceOutOfBoundsSnafu {
            pos: pos,
            disp: disp.clone(),
        })
    }

    pub fn execute_command(
        &mut self,
        command: Command,
    ) -> Result<Option<Effect>, StateError> {
        let entity = self.get_entity_by_id(command.entity_id)?;
        match command.verb {
            Verb::Wait => return Ok(None),
            Verb::AttemptMove(dir) => {
                let from = entity.pos.clone();
                let to = State::add_displace(entity.pos, &Displace::from(dir))?;
                ensure!(entity.can_move(), NoWalkSnafu { pos: entity.pos },);
                self.move_entity(from, to)?;
                return Ok(Some(Effect::EntityMove(from, to)));
            }
            Verb::GetMaterials(neigh, load) => {
                let to = entity.pos.clone();
                let from = State::add_displace(entity.pos, &neigh.into())?;
                ensure!(entity.has_ability(), NoAbilitiesSnafu { pos: to });
                self.move_material_to_entity(from, to, &load)?;
                return Ok(Some(Effect::AssetsFloorToEntity {
                    mat: load,
                    from,
                    to,
                }));
            }
            Verb::DropMaterials(neigh, load) => {
                let from = entity.pos.clone();
                let to = State::add_displace(entity.pos, &neigh.into())?;
                ensure!(entity.has_ability(), NoAbilitiesSnafu { pos: from });
                self.move_material_to_floor(from, to, &load)?;
                return Ok(Some(Effect::AssetsEntityToFloor {
                    mat: load,
                    from,
                    to,
                }));
            }
            Verb::Shoot(disp) => {
                let from = entity.pos.clone();
                ensure!(entity.can_shoot(), NoShootSnafu { pos: entity.pos });
                ensure!(entity.has_copper(), NoCopperSnafu { pos: entity.pos });
                let damage = entity.get_gun_damage().unwrap();
                ensure!(
                    disp.square_norm() <= 25,
                    DisplaceTooFarSnafu { disp: disp }
                );
                let to = self.get_visible(from, &disp).ok_or(
                    StateError::NotVisible {
                        pos: from,
                        disp: disp.clone(),
                    },
                )?;
                self.attack(to, damage)?;
                return Ok(Some(Effect::Shoot { from, to, damage }));
            }
            Verb::Drill(dir) => {
                ensure!(
                    entity.has_ability(),
                    NoAbilitiesSnafu { pos: entity.pos }
                );
                let damage = entity.get_drill_damage().unwrap();
                let from = entity.pos.clone();
                let to = add_displace(entity.pos, &dir.into()).context(
                    DisplaceOutOfBoundsSnafu {
                        pos: entity.pos,
                        disp: dir.clone(),
                    },
                )?;
                self.attack(to, damage)?;
                return Ok(Some(Effect::Drill { from, to, damage }));
            }
            Verb::Construct(index, dir) => {
                let from = entity.pos.clone();
                let subtract_the_material_from_entity = 0;
                let creature = self.get_creature(entity.team, index)?;
                ensure!(
                    entity.materials >= cost(&creature),
                    NoMaterialEntitySnafu {
                        pos: from,
                        load: cost(&creature)
                    }
                );
                let to = State::add_displace(from, &Displace::from(dir))?;
                let team = entity.team;
                self.build_entity_from_template(team, index, to)?;
                return Ok(Some(Effect::Construct(Construct {
                    team,
                    template_index: index,
                    builder: from,
                    buildee: to,
                })));
            }
            _ => {
                return Ok(None);
            }
        };
    }
}
