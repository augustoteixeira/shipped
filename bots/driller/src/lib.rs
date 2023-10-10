use tools::driller::next;
use tools::encoder::encode_verb;

#[no_mangle]
pub fn execute() -> i64 {
  encode_verb(next())
}
