use super::game::{Pos, Verb};
use super::mover::{Mover, MoverState};

pub type ExplorerState = MoverState;

pub struct Explorer {
  pointer: *mut MoverState,
}

extern "C" {
  fn get_rand() -> u32;
}

impl Explorer {
  pub fn new(pointer: *mut ExplorerState) -> Self {
    Explorer { pointer }
  }
  pub fn next(&self) -> Verb {
    let mover = Mover::new(self.pointer as *mut MoverState);
    let move_verb = mover.next();
    if let Verb::Wait = move_verb {
      unsafe {
        let x = (get_rand() % 60) as usize;
        let y = (get_rand() % 60) as usize;
        let target = self.pointer as *mut Pos;
        *target = Pos { x, y };
      }
    }
    move_verb
  }
}
