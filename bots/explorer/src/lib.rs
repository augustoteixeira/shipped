use tools::encoder::encode_verb;
use tools::explorer::{Explorer, ExplorerState};
use tools::game::Pos;

static mut EXPLORER: ExplorerState = Pos { x: 32, y: 20 };

#[no_mangle]
pub fn execute() -> i64 {
  let explorer = unsafe { Explorer::new(&mut EXPLORER as *mut ExplorerState) };
  encode_verb(explorer.next())
}
