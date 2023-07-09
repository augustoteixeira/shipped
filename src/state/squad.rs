use serde::{Deserialize, Serialize};
use snafu::prelude::*;
use std::collections::HashMap;

use super::constants::{HEIGHT, NUM_CODES, NUM_TEMPLATES, WIDTH};
use super::entity::{Code, Entity, Team, TemplateEntity};
use super::geometry::Pos;
use super::materials::Materials;
use super::state::{State, StateError, Tile};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Placement {
    pub template: usize,
    pub pos: Pos,
    pub grayed: bool,
}

// what is returned from the level editor:
// - array of options of body templates
// - array of options of half_entity templates
// - array of options of full_entity templates
// - ...

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Squad {
    pub codes: [Option<Code>; NUM_CODES],
    pub templates: [Option<TemplateEntity>; NUM_TEMPLATES],
    pub placements: Vec<Placement>,
}

#[derive(Debug, Snafu)]
pub enum SquadError {
    #[snafu(display("Wrong court side, team: {:?}, pos: {:?}", team, pos))]
    WrongCourtSide { team: Team, pos: Pos },
    #[snafu(display("Squad from {:?} contained outsider", team))]
    WrongTeam { team: Team },
    #[snafu(display(
        "Placing template: {} from team: {:?} in pos: {:?}",
        template,
        team,
        pos
    ))]
    BuildEntityError {
        source: StateError,
        team: Team,
        template: usize,
        pos: Pos,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub min_tokens: usize,
    pub tiles: Vec<Tile>,
}
// implement comparison between Blueprints. It is used to
// we require:
// - if a.body_template[i] = Some(e), then b.body_template[i] should be the same
// - same with half and full
// - entities that exist in a should also exist in b. In the same position.
// - entities will give rise to certain amounts of: full, half, bodies and materials.
//   all these values should be bigger or equal in b than in a.
// - perhaps key should be an asset as well, distributed by the level designer and the players
// - think abou how to create NPC entities.

// implement conversion from two blueprints to one state
pub fn build_state(
    blue_squad: Squad,
    red_squad: Squad,
    settings: Settings,
) -> Result<State, SquadError> {
    let mut state = State::new(
        settings.min_tokens,
        blue_squad.codes,
        red_squad.codes,
        HashMap::new(),
        blue_squad.templates,
        red_squad.templates,
        (0..(WIDTH * HEIGHT))
            .map(|_| Tile {
                entity_id: None,
                materials: Materials {
                    carbon: 0,
                    silicon: 0,
                    plutonium: 0,
                    copper: 0,
                },
            })
            .collect(),
    );
    state.tiles = settings.tiles;
    for placement in blue_squad.placements {
        ensure!(
            placement.pos.y < HEIGHT / 2,
            WrongCourtSideSnafu {
                team: Team::Blue,
                pos: placement.pos
            }
        );
        state
            .build_entity_from_template(
                if placement.grayed {
                    Team::BlueGray
                } else {
                    Team::Blue
                },
                if placement.grayed { 0 } else { 1 },
                placement.template,
                placement.pos,
            )
            .context(BuildEntitySnafu {
                team: Team::Blue,
                template: placement.template,
                pos: placement.pos,
            })?;
    }
    for placement in red_squad.placements {
        ensure!(
            placement.pos.y >= HEIGHT / 2,
            WrongCourtSideSnafu {
                team: Team::Red,
                pos: placement.pos
            }
        );
        state
            .build_entity_from_template(
                if placement.grayed {
                    Team::RedGray
                } else {
                    Team::Red
                },
                if placement.grayed { 0 } else { 1 },
                placement.template,
                placement.pos,
            )
            .context(BuildEntitySnafu {
                team: Team::Red,
                template: placement.template,
                pos: placement.pos,
            })?;
    }
    Ok(state)
}
