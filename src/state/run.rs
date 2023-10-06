extern crate rand;
extern crate rand_chacha;

use std::sync::Arc;
use std::sync::Mutex;

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
  let state = Arc::new(Mutex::new(initial_state.clone())).clone();

  let mut brains: Brains = Brains::new(state.clone()).unwrap();
  let mut frames: Vec<Frame> = vec![];

  for _ in 1..turns {
    let mut frame = vec![];
    let id_vec = state.lock().unwrap().get_entities_ids();
    for id in id_vec {
      let exists = matches!(state.lock().unwrap().get_entity_by_id(id), Ok(_));
      if exists {
        match brains.get_command(id) {
          Ok(command) => {
            if let Ok(_) = state.lock().unwrap().execute_command(command.clone()) {
              frame.push(command.clone());
            }
          }
          Err(e) => {
            println!("{:}", e);
          }
        };
      }
    }
    frames.push(frame);
  }
  Script {
    genesis: initial_state,
    frames,
  }
}
