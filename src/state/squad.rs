use serde::{Deserialize, Serialize};
use snafu::prelude::*;
use std::collections::HashMap;

use super::constants::{HEIGHT, NUM_CODES, NUM_TEMPLATES, WIDTH};
use super::entity::{Code, FullEntity, Team};
use super::geometry::Pos;
use super::materials::Materials;
use super::state::{State, Tile};

//pub struct Tiles<T, const N: usize>(pub [T; WIDTH * HEIGHT]);
//pub struct Tiles<const N: usize>(pub [Tile; WIDTH * HEIGHT]);
//#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Squad {
    codes: [Option<Code>; NUM_CODES],
    templates: [Option<FullEntity>; NUM_TEMPLATES],
    // array of options of body templates
    // array of options of half_entity templates
    // array of options of full_entity templates
    // pub entities: HashMap<Id, FullEntity>,
}

#[derive(Debug, Snafu)]
pub enum SquadError {
    #[snafu(display("Wrong court side, team: {:?}, pos: {:?}", team, pos))]
    WrongCourtSide { team: Team, pos: Pos },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    min_tokens: usize,
    gray_squad: Squad,
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
) -> Result<(), SquadError> {
    let mut state = State::new(
        settings.min_tokens,
        blue_squad.codes,
        red_squad.codes,
        HashMap::new(),
        blue_squad.templates,
        settings.gray_squad.templates,
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
    // - fill entities (making sanity checks)
    // - fill tiles
    Ok(())
}
