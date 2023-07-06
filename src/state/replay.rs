use serde::{Deserialize, Serialize};

use super::entity::{Materials, Message, Team};
use super::geometry::Pos;
use super::state::{Command, State};

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
