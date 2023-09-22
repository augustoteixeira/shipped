const WASM_MEMORY_BUFFER_SIZE: usize = 1;
static mut WASM_MEMORY_BUFFER: [u8; WASM_MEMORY_BUFFER_SIZE] = [0; WASM_MEMORY_BUFFER_SIZE];

// Function to store the passed value in our buffer
fn store_value_in_wasm_memory(index: usize, value: u8) -> bool {
  if index < WASM_MEMORY_BUFFER_SIZE {
    unsafe {
      WASM_MEMORY_BUFFER[index] = value;
    }
    true
  } else {
    false
  }
}

// Function to get value in our buffer
fn read_value_in_wasm_memory(index: usize) -> Option<u8> {
  if index < WASM_MEMORY_BUFFER_SIZE {
    unsafe {
      return Some(WASM_MEMORY_BUFFER[index]);
    }
  } else {
    None
  }
}

extern "C" {
  fn rand() -> u32;
}

#[no_mangle]
pub fn execute() -> i64 {
  let value = read_value_in_wasm_memory(0);
  if let Some(_) = value {
    let random = unsafe { rand() };
    if random < (1 << 30) {
      //store_value_in_wasm_memory(0, 0);
      return 0x0002010000000000;
    } else {
      //store_value_in_wasm_memory(0, new_value);
      return 0x0002020000000000;
    }
  }
  return 0x0001000000000000;
}
