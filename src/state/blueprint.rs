use serde::{Deserialize, Serialize};
use snafu::prelude::*;

//pub struct Tiles<T, const N: usize>(pub [T; WIDTH * HEIGHT]);
//pub struct Tiles<const N: usize>(pub [Tile; WIDTH * HEIGHT]);
//#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blueprint {
    codes: [Option<Code>; NUM_CODES],
    // array of options of body templates
    // array of options of half_entity templates
    // array of options of full_entity templates
    // pub entities: HashMap<Id, FullEntity>,
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
