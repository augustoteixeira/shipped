use tools::encoder::decode_coord;

extern "C" {
  fn get_coord() -> u32;
}

#[no_mangle]
pub fn execute() -> i64 {
  let code = unsafe { get_coord() };
  let (x, y) = decode_coord(code);
  match x {
    0..=28 => return 0x0002020000000000,
    31..=64 => return 0x0002010000000000,
    _ => match y {
      0..=28 => return 0x0002000000000000,
      31..=64 => return 0x0002030000000000,
      _ => return 0x0001000000000000,
    },
  }
}
