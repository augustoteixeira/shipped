use tools::driller;
use tools::encoder::encode_verb;
use tools::explorer::{Explorer, ExplorerState};
use tools::game::{Pos, Verb};

static mut EXPLORER: ExplorerState = Pos { x: 32, y: 20 };

extern "C" {
  fn get_entity(_: u16) -> i64;
}

#[no_mangle]
pub fn execute() -> i64 {
  let driller_verb = driller::next();
  if let Verb::Wait = driller_verb {
    let explorer = unsafe { Explorer::new(&mut EXPLORER as *mut ExplorerState) };
    encode_verb(explorer.next())
  } else {
    encode_verb(driller_verb)
  }
}
