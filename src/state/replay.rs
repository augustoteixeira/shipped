use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use super::entity::{Materials, Message};
use super::geometry::Pos;
use super::state::{State, StateError, Team};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Effect {
    EntityMove(Pos, Pos),
    AssetsFloorToEntity { mat: Materials, from: Pos, to: Pos },
    AssetsEntityToFloor { mat: Materials, from: Pos, to: Pos },
    Shoot(Attack),
    Drill(Attack),
    Construct(Construct),
    SendMessage(Pos, Option<Message>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attack {
    pub origin: Pos,
    pub destination: Pos,
    pub damage: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        Effect::Shoot(a) => {
            state
                .attack(a.destination, a.damage)
                .context(AttackUnitSnafu { attack: a })?;
        }
        Effect::Drill(a) => {
            state
                .attack(a.destination, a.damage)
                .context(AttackUnitSnafu { attack: a })?;
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

pub type Frame = Vec<Effect>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Script {
    pub genesis: State,
    pub frames: Vec<Frame>,
}
