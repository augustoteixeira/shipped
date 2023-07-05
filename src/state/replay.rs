use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use super::entity::{Materials, Message, Team};
use super::geometry::Pos;
use super::state::{Command, State, StateError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Effect {
    EntityMove(Pos, Pos),
    AssetsFloorToEntity { mat: Materials, from: Pos, to: Pos },
    AssetsEntityToFloor { mat: Materials, from: Pos, to: Pos },
    Shoot { from: Pos, to: Pos, damage: usize },
    Drill { from: Pos, to: Pos, damage: usize },
    Construct(Construct),
    SendMessage(Pos, Option<Message>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Construct {
    pub team: Team,
    pub template_index: usize,
    pub builder: Pos,
    pub buildee: Pos,
}

pub type Frame = Vec<Command>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Script {
    pub genesis: State,
    pub frames: Vec<Frame>,
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
    #[snafu(display("Attacking: {from:?}, {to:?}, {damage}"))]
    AttackUnit {
        source: StateError,
        from: Pos,
        to: Pos,
        damage: usize,
    },
    #[snafu(display("Construct: {construct:?}"))]
    ConstructError {
        source: StateError,
        construct: Construct,
    },
    #[snafu(display("Set message error in {:?}", message))]
    SetMessageError {
        source: StateError,
        pos: Pos,
        message: Option<Message>,
    },
}

// replay does not try to check logic (like fov). Only the basic necesary
// for its continued good behavior. the other logic was tested during the
// generation of the logs.
pub fn implement_effect(
    state: &mut State,
    effect: Effect,
) -> Result<(), UpdateError> {
    match effect {
        Effect::EntityMove(from, to) => {
            state
                .move_entity(from, to)
                .context(EntityMoveSnafu { from, to })?;
        }
        Effect::AssetsFloorToEntity {
            mat: load,
            from,
            to,
        } => {
            state.move_material_to_entity(from, to, &load).context(
                MaterialMoveToEntitySnafu {
                    from,
                    to,
                    load: load.clone(),
                },
            )?;
        }
        Effect::AssetsEntityToFloor {
            mat: load,
            from,
            to,
        } => {
            state.move_material_to_floor(from, to, &load).context(
                MaterialMoveToFloorSnafu {
                    from,
                    to,
                    load: load.clone(),
                },
            )?;
        }
        Effect::Shoot { from, to, damage } => {
            state.attack(to, damage).context(AttackUnitSnafu {
                from,
                to,
                damage,
            })?;
        }
        Effect::Drill { from, to, damage } => {
            state.attack(to, damage).context(AttackUnitSnafu {
                from,
                to,
                damage,
            })?;
        }
        Effect::Construct(c) => {
            state
                .build_entity_from_template(c.team, c.template_index, c.buildee)
                .context(ConstructSnafu {
                    construct: c.clone(),
                })?;
        }
        Effect::SendMessage(pos, message) => {
            state.set_message(pos, message).context(SetMessageSnafu {
                pos,
                message: message,
            })?
        }
    }
    Ok(())
}
