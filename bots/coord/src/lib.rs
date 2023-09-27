extern "C" {
  fn get_coord() -> u32;
}

#[no_mangle]
pub fn execute() -> i64 {
  let coord = unsafe { get_coord() };
  if coord > 31 {
    return 0x0002010000000000;
  } else {
    return 0x0002020000000000;
  }
}
