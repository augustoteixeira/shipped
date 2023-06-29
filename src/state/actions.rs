use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use super::entity::{Id, Materials, Message};
use super::geometry::{
    add_displace, are_neighbors, is_within_bounds, Displace, GeometryError, Pos,
};
use super::replay::Effect;
use super::state::{State, StateError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub entity_id: usize,
    pub verb: Verb,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Verb {
    Wait,
    AttemptMove(Displace),
    GetMaterials(Displace, Materials),
    DropMaterials(Displace, Materials),
    Shoot(Pos),
    Drill(Pos),
    Construct(usize),
    SetMessage(Option<Message>),
}

#[derive(Debug, Snafu)]
pub enum ValidationError {
    #[snafu(display("No entity with id {}", id))]
    NoEntityWithId { source: StateError, id: Id },
    #[snafu(display("Position out of bounds {pos}"))]
    OutOfBounds { pos: Pos },
    #[snafu(display("Interact with non-neighbor from {from} to {to}"))]
    InteractFar { from: Pos, to: Pos },
    #[snafu(display("Move to non-empty {to}"))]
    MoveOccupied { to: Pos },
    #[snafu(display("Displace to negative: from {}, by {:?}", pos, disp))]
    DisplaceNeg {
        source: GeometryError,
        pos: Pos,
        disp: Displace,
    },
}

pub fn validate_command(
    state: &State,
    command: Command,
) -> Result<Option<Effect>, ValidationError> {
    let entity = state.get_entity_by_id(command.entity_id).context(
        NoEntityWithIdSnafu {
            id: command.entity_id,
        },
    )?;
    match command.verb {
        Verb::Wait => return Ok(None),
        Verb::AttemptMove(disp) => {
            let new_pos =
                add_displace(entity.pos, &disp).context(DisplaceNegSnafu {
                    pos: entity.pos,
                    disp: disp.clone(),
                })?;
            ensure!(
                is_within_bounds(new_pos),
                OutOfBoundsSnafu { pos: new_pos }
            );
            ensure!(
                are_neighbors(entity.pos, new_pos),
                InteractFarSnafu {
                    from: entity.pos,
                    to: new_pos
                }
            );
            ensure!(
                !state.has_entity(new_pos),
                MoveOccupiedSnafu { to: new_pos }
            );
            return Ok(Some(Effect::EntityMove(entity.pos, new_pos)));
        }
        Verb::GetMaterials(disp, mat) => {
            let floor_pos =
                add_displace(entity.pos, &disp).context(DisplaceNegSnafu {
                    pos: entity.pos,
                    disp: disp.clone(),
                })?;
            ensure!(
                is_within_bounds(floor_pos),
                OutOfBoundsSnafu { pos: floor_pos }
            );
            ensure!(
                are_neighbors(entity.pos, floor_pos)
                    | (entity.pos == floor_pos),
                InteractFarSnafu {
                    from: entity.pos,
                    to: floor_pos
                }
            );
            return Ok(Some(Effect::AssetsFloorToEntity {
                mat,
                from: floor_pos,
                to: entity.pos,
            }));
        }
        Verb::DropMaterials(disp, mat) => {
            let floor_pos =
                add_displace(entity.pos, &disp).context(DisplaceNegSnafu {
                    pos: entity.pos,
                    disp: disp.clone(),
                })?;
            ensure!(
                is_within_bounds(floor_pos),
                OutOfBoundsSnafu { pos: floor_pos }
            );
            ensure!(
                are_neighbors(entity.pos, floor_pos)
                    | (entity.pos == floor_pos),
                InteractFarSnafu {
                    from: entity.pos,
                    to: floor_pos
                }
            );
            return Ok(Some(Effect::AssetsEntityToFloor {
                mat,
                from: entity.pos,
                to: floor_pos,
            }));
        }
        _ => return Ok(None),
    };
}
