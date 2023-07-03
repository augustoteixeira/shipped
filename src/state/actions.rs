extern crate line_drawing;

use line_drawing::Bresenham;
use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use super::entity::{Id, Materials, Message};
use super::geometry::{
    add_displace, are_neighbors, is_within_bounds, is_within_bounds_signed,
    Direction, Displace, GeometryError, Pos,
};
use super::replay::Effect;
use super::state::{State, StateError};
use crate::state::entity::MovementType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub entity_id: usize,
    pub verb: Verb,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Verb {
    Wait,
    AttemptMove(Direction),
    GetMaterials(Displace, Materials),
    DropMaterials(Displace, Materials),
    Shoot(Displace),
    Drill(Pos),
    Construct(usize),
    SetMessage(Option<Message>),
}

#[derive(Debug, Snafu)]
pub enum ValidationError {
    #[snafu(display("No entity with id {}", id))]
    NoEntityWithId { source: StateError, id: Id },
    #[snafu(display("Position out of bounds (signed) {x} {y}"))]
    OutOfBoundsSigned { x: i64, y: i64 },
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
    #[snafu(display("Entity with no abilities in {}", pos))]
    NoAbility { pos: Pos },
    #[snafu(display("Entity cannot walk {}", pos))]
    NoWalk { pos: Pos },
    #[snafu(display("Entity cannot shoot {}", pos))]
    NoShoot { pos: Pos },
    #[snafu(display("Too far to interact {:?}", disp))]
    TooFar { disp: Displace },
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
        Verb::AttemptMove(dir) => {
            let new_pos = add_displace(entity.pos, &Displace::from(dir))
                .context(DisplaceNegSnafu {
                    pos: entity.pos,
                    disp: Displace::from(dir.clone()),
                })?;
            ensure!(
                is_within_bounds(new_pos),
                OutOfBoundsSnafu { pos: new_pos } // TODO: unify this check, it is done in other ways below
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
            match &entity.abilities {
                None => {
                    return Err(ValidationError::NoAbility { pos: entity.pos })
                }
                Some(a) => {
                    ensure!(
                        a.movement_type == MovementType::Walk,
                        NoWalkSnafu { pos: entity.pos },
                    );
                }
            };
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
            ensure!(
                entity.abilities.is_some(),
                NoAbilitySnafu { pos: entity.pos }
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
            ensure!(
                entity.abilities.is_some(),
                NoAbilitySnafu { pos: entity.pos }
            );
            return Ok(Some(Effect::AssetsEntityToFloor {
                mat,
                from: entity.pos,
                to: floor_pos,
            }));
        }
        Verb::Shoot(d) => {
            match &entity.abilities {
                None => {
                    return Err(ValidationError::NoAbility { pos: entity.pos })
                }
                Some(a) => {
                    ensure!(a.gun_damage > 0, NoShootSnafu { pos: entity.pos });
                    ensure!(d.square_norm() <= 25, TooFarSnafu { disp: d });
                    for (x, y) in Bresenham::new(
                        (entity.pos.x as i64, entity.pos.y as i64),
                        (entity.pos.x as i64 + d.x, entity.pos.y as i64 + d.y),
                    )
                    .skip(1)
                    {
                        ensure!(
                            is_within_bounds_signed(x, y),
                            OutOfBoundsSignedSnafu { x, y }
                        );
                        if state.has_entity(Pos::new(x as usize, y as usize)) {
                            return Ok(Some(Effect::Shoot {
                                from: entity.pos,
                                to: Pos::new(x as usize, y as usize),
                                damage: a.gun_damage,
                            }));
                        }
                    }
                }
            }
            return Ok(None);
        }
        _ => return Ok(None),
    };
}
