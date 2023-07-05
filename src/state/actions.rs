use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use super::entity::{cost, Id, Materials, Message};
use super::geometry::{
    add_displace, are_neighbors, is_within_bounds, Direction, Displace,
    GeometryError, Neighbor, Pos,
};
use super::replay::{implement_effect, Construct, Effect, UpdateError};
use super::state::{State, StateError};

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
    #[snafu(display("Displace out of bounds: from {}, by {:?}", pos, disp))]
    DisplaceOut {
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
    #[snafu(display("Entity has no copper {}", pos))]
    NoCopper { pos: Pos },
    #[snafu(display("Not visible {}, {:?}", pos, disp))]
    NotVisible { pos: Pos, disp: Displace },
    #[snafu(display("Too far to interact {:?}", disp))]
    TooFar { disp: Displace },
    #[snafu(display("Error constructing from {}, index {}", pos, index))]
    Construct {
        source: StateError,
        pos: Pos,
        index: usize,
        dir: Direction,
    },
    #[snafu(display("Entity at {} does not have enough {:?}", pos, mat))]
    NotEnoughMaterial { pos: Pos, mat: Materials },
    #[snafu(display("Error implementing effect {:?}", effect))]
    ImplementingEffect { source: UpdateError, effect: Effect },
    #[snafu(display("No template with index {}", index))]
    NoTemplate { source: StateError, index: usize },
    #[snafu(display("Cannot create with template {} in {}", index, pos))]
    CannotCreate {
        source: StateError,
        index: usize,
        pos: Pos,
    },
}

pub fn validate_command(
    state: &mut State,
    command: Command,
) -> Result<Option<Effect>, ValidationError> {
    let entity = state.get_entity_by_id(command.entity_id).context(
        NoEntityWithIdSnafu {
            id: command.entity_id,
        },
    )?;
    let effect: Option<Effect> = match command.verb {
        Verb::Wait => return Ok(None),
        Verb::AttemptMove(dir) => {
            let new_pos = add_displace(entity.pos, &Displace::from(dir))
                .context(DisplaceOutSnafu {
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
            ensure!(entity.can_move(), NoWalkSnafu { pos: entity.pos },);
            Some(Effect::EntityMove(entity.pos, new_pos))
        }
        Verb::GetMaterials(neighbor, mat) => {
            let floor_pos = add_displace(entity.pos, &neighbor.into())
                .context(DisplaceOutSnafu {
                    pos: entity.pos,
                    disp: neighbor.clone(),
                })?;
            ensure!(
                is_within_bounds(floor_pos),
                OutOfBoundsSnafu { pos: floor_pos }
            );
            // ensure!(
            //     are_neighbors(entity.pos, floor_pos)
            //         | (entity.pos == floor_pos),
            //     InteractFarSnafu {
            //         from: entity.pos,
            //         to: floor_pos
            //     }
            // );
            ensure!(entity.has_ability(), NoAbilitySnafu { pos: entity.pos });
            Some(Effect::AssetsFloorToEntity {
                mat,
                from: floor_pos,
                to: entity.pos,
            })
        }
        Verb::DropMaterials(neighbor, mat) => {
            let floor_pos = add_displace(entity.pos, &neighbor.into())
                .context(DisplaceOutSnafu {
                    pos: entity.pos,
                    disp: neighbor.clone(),
                })?;
            ensure!(
                is_within_bounds(floor_pos),
                OutOfBoundsSnafu { pos: floor_pos }
            );
            // ensure!(
            //     are_neighbors(entity.pos, floor_pos)
            //         | (entity.pos == floor_pos),
            //     InteractFarSnafu {
            //         from: entity.pos,
            //         to: floor_pos
            //     }
            // );
            ensure!(entity.has_ability(), NoAbilitySnafu { pos: entity.pos });
            Some(Effect::AssetsEntityToFloor {
                mat,
                from: entity.pos,
                to: floor_pos,
            })
        }
        Verb::Shoot(disp) => {
            ensure!(entity.can_shoot(), NoShootSnafu { pos: entity.pos });
            ensure!(entity.has_copper(), NoCopperSnafu { pos: entity.pos });
            let damage = entity.get_gun_damage().unwrap();
            ensure!(disp.square_norm() <= 25, TooFarSnafu { disp: disp });
            let target = state.get_visible(entity.pos, &disp).ok_or(
                ValidationError::NotVisible {
                    pos: entity.pos,
                    disp: disp.clone(),
                },
            )?;
            Some(Effect::Shoot {
                from: entity.pos,
                to: target,
                damage,
            })
        }
        Verb::Drill(dir) => {
            ensure!(entity.has_ability(), NoAbilitySnafu { pos: entity.pos });
            let damage = entity.get_drill_damage().unwrap();
            let to = add_displace(entity.pos, &dir.into()).context(
                DisplaceOutSnafu {
                    pos: entity.pos,
                    disp: dir.clone(),
                },
            )?;
            ensure!(is_within_bounds(to), OutOfBoundsSnafu { pos: to });
            Some(Effect::Drill {
                from: entity.pos,
                to,
                damage,
            })
        }
        Verb::Construct(index, dir) => {
            let creature = state
                .get_creature(entity.team, index)
                .context(NoTemplateSnafu { index })?;
            // ensure!(
            //     entity.materials >= cost(&creature),
            //     NotEnoughMaterialSnafu {
            //         pos: entity.pos,
            //         mat: cost(&creature)
            //     }
            // );
            let creature_pos = add_displace(entity.pos, &Displace::from(dir))
                .context(DisplaceOutSnafu {
                pos: entity.pos,
                disp: Displace::from(dir),
            })?;
            println!("Pos {creature_pos:?}");
            let entity_pos = entity.pos;
            let team = entity.team;
            state
                .build_entity_from_template(team, index, creature_pos)
                .context({
                    CannotCreateSnafu {
                        index,
                        pos: creature_pos,
                    }
                })?;
            Some(Effect::Construct(Construct {
                team,
                template_index: index,
                builder: entity_pos,
                buildee: creature_pos,
            }))
        }
        _ => None,
    };
    if let Some(e) = &effect {
        if implement_effect(state, e.clone()).is_ok() {
            return Ok(effect);
        }
    }
    return Ok(None);
}
