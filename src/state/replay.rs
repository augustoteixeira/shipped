use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use super::entity::{Materials, Message};
use super::geometry::Pos;
use super::state::{State, StateError, Team};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    EntityMove(Pos, Pos),
    AssetsFloorToEntity(Materials, Pos, Pos),
    AssetsEntityToFloor(Materials, Pos, Pos),
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
pub fn replay_event(
    state: &mut State,
    event: Event,
) -> Result<(), UpdateError> {
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
            state.set_message(pos, message).context(SetMessageSnafu {
                pos,
                message: message,
            })?
        }
    }
    Ok(())
}

pub type Frame = Vec<Event>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Script {
    pub genesis: State,
    pub frames: Vec<Frame>,
}
