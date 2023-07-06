use serde::{Deserialize, Serialize};

use super::entity::{Materials, Message, Team};
use super::geometry::Pos;
use super::state::{Command, State};

pub type Frame = Vec<Command>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Script {
    pub genesis: State,
    pub frames: Vec<Frame>,
}
