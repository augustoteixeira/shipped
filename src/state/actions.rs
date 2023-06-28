use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use super::entity::{Id, Materials, Message};
use super::geometry::{
    add_displace, are_neighbors, is_within_bounds, Displace, GeometryError, Pos,
};
use super::replay::Event;
use super::state::{State, StateError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    pub entity_id: usize,
    pub verb: Verb,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Verb {
    Wait,
    AttemptMove(Displace),
    GetMaterials(Pos, Materials),
    DropMaterials(Pos, Materials),
    Shoot(Pos),
    Drill(Pos),
    Construct(usize),
    SetMessage(Option<Message>),
}

#[derive(Debug, Snafu)]
pub enum ActionError {
    #[snafu(display("No entity with id {}", id))]
    NoEntityWithActionId { source: StateError, id: Id },
    #[snafu(display("Position out of bounds {pos}"))]
    OutOfBounds { pos: Pos },
    #[snafu(display("Move to non-neighbor from {from} to {to}"))]
    MoveFar { from: Pos, to: Pos },
    #[snafu(display("Move to non-empty {to}"))]
    MoveOccupied { to: Pos },
    #[snafu(display("Displace to negative: from {}, by {:?}", pos, disp))]
    DisplaceNeg {
        source: GeometryError,
        pos: Pos,
        disp: Displace,
    },
}

pub fn implement_action(
    state: &State,
    action: Action,
) -> Result<Option<Event>, ActionError> {
    let entity = state.get_entity_by_id(action.entity_id).context(
        NoEntityWithActionIdSnafu {
            id: action.entity_id,
        },
    )?;
    match action.verb {
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
                MoveFarSnafu {
                    from: entity.pos,
                    to: new_pos
                }
            );
            ensure!(
                !state.has_entity(new_pos),
                MoveOccupiedSnafu { to: new_pos }
            );
            return Ok(Some(Event::EntityMove(entity.pos, new_pos)));
        }
        _ => return Ok(None),
    };
}
