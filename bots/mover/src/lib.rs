use tools::game::Pos;
use tools::mover::{Mover, MoverState};

static mut MOVER: MoverState = Pos { x: 32, y: 20 };

#[no_mangle]
pub fn execute() -> i64 {
  let mover = unsafe { Mover::new(&mut MOVER as *mut MoverState) };
  mover.next()
}
