use std::path::Path;

pub fn get_next_file_number(path: &Path, extension: String) -> usize {
  assert!(path.is_dir());
  let mut i: usize = 0;
  loop {
    let dest_filename = format!("{:05}", i);
    let mut dest = path.join(dest_filename);
    dest.set_extension(&extension);
    if dest.exists() {
      i = i + 1;
    } else {
      break;
    }
  }
  i
}
