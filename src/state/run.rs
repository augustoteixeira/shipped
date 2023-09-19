extern crate rand;
extern crate rand_chacha;

use crate::state::bf::{build_state, BFState};
use crate::state::brain::Brains;
use crate::state::state::{Frame, Script};

pub fn run_match(
  level: &BFState,
  blue_squad: &BFState,
  red_squad: &BFState,
  turns: usize,
) -> Script {
  // run match
  let initial_state = build_state(&level, &blue_squad, &red_squad);
  let initial_id_vec = initial_state.get_entities_ids();
  let mut brains: Brains = Brains::new(initial_id_vec).unwrap();
  let mut frames: Vec<Frame> = vec![];
  //let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(17).try_into().unwrap();
  let mut state = initial_state.clone();
  for _ in 1..turns {
    let mut frame = vec![];
    let id_vec = state.get_entities_ids();
    for id in id_vec {
      match brains.get_command(id) {
        Ok(command) => {
          if let Ok(_) = state.execute_command(command.clone()) {
            frame.push(command.clone());
          }
        }
        Err(e) => {
          println!("{:}", e);
        }
      }
    }
    frames.push(frame);
  }
  Script {
    genesis: initial_state,
    frames,
  }
}
