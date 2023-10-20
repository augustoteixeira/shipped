use tools::abbrev::{GO_NORTH, GO_WEST};
use tools::encoder::{decode_tile_materials, encode_displace, encode_verb};
use tools::game::{Displace, Materials, Neighbor, Verb};

extern "C" {
  fn get_materials(_: u16) -> i64;
}

#[no_mangle]
pub fn execute() -> i64 {
  let code = unsafe { get_materials(encode_displace(&Displace { x: 0, y: 1 })) };
  let tile = decode_tile_materials(code);
  match tile {
    Some(materials) => {
      if materials.carbon == 0 {
        encode_verb(GO_NORTH)
      } else {
        encode_verb(Verb::GetMaterials(
          Neighbor::North,
          Materials {
            carbon: 1,
            silicon: 0,
            plutonium: 0,
            copper: 0,
          },
        ))
      }
    }
    None => encode_verb(GO_WEST),
  }
}
