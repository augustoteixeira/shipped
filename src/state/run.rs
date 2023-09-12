extern crate rand;
extern crate rand_chacha;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::state::bf::{build_state, BFState};
use crate::state::brain::random_verb;
use crate::state::state::{Command as StateCommand, Frame, Script};

pub fn run_match(
  level: &BFState,
  blue_squad: &BFState,
  red_squad: &BFState,
  turns: usize,
) -> Script {
  // run match
  let initial_state = build_state(&level, &blue_squad, &red_squad);
  let mut state = initial_state.clone();
  let mut frames: Vec<Frame> = vec![];
  let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(17).try_into().unwrap();
  for _ in 1..turns {
    let mut frame = vec![];
    let id_vec = state.get_entities_ids();
    for id in id_vec {
      let command = StateCommand {
        entity_id: id,
        verb: random_verb(&mut rng),
      };
      if let Ok(_) = state.execute_command(command.clone()) {
        frame.push(command.clone());
      }
    }
    frames.push(frame);
  }
  Script {
    genesis: initial_state,
    frames,
  }
}
