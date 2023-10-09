extern "C" {
  fn get_rand() -> u32;
}

#[no_mangle]
pub fn execute() -> i64 {
  let random = unsafe { get_rand() };
  if random < (1 << 30) {
    return 0x0002010000000000;
  } else {
    return 0x0002020000000000;
  }
}
