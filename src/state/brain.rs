extern crate rand;
extern crate rand_chacha;
use snafu::prelude::*;

use init_array::init_array;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use wasmer::{
  imports, CompileError, ExportError, Instance, InstantiationError, Module, RuntimeError, Store,
  Value,
};

use crate::state::constants::NUM_TEMPLATES;
use crate::state::geometry::{Direction, Displace, Neighbor};
use crate::state::materials::Materials;
use crate::state::state::{Command, Id, Verb};

fn decode_direction(code: u8) -> Option<Direction> {
  match code {
    0 => Some(Direction::North),
    1 => Some(Direction::West),
    2 => Some(Direction::East),
    3 => Some(Direction::South),
    _ => None,
  }
}

fn decode_neighbor(code: u8) -> Option<Neighbor> {
  match code {
    0 => Some(Neighbor::Here),
    1 => Some(Neighbor::North),
    2 => Some(Neighbor::West),
    3 => Some(Neighbor::East),
    4 => Some(Neighbor::South),
    _ => None,
  }
}

fn decode_materials(code: u32) -> Materials {
  let carbon: usize = (code & 0x000000FF).try_into().unwrap();
  let silicon: usize = ((code & 0x0000FF00) >> 8).try_into().unwrap();
  let plutonium: usize = ((code & 0x00FF0000) >> 16).try_into().unwrap();
  let copper: usize = ((code & 0xFF000000) >> 24).try_into().unwrap();
  Materials {
    carbon,
    silicon,
    plutonium,
    copper,
  }
}

fn decode_displace(code: u16) -> Displace {
  let x: u8 = ((code & 0xFF00) >> 8).try_into().unwrap();
  let y: u8 = ((code & 0x00FF) >> 8).try_into().unwrap();
  let signed_x = x as i8;
  let signed_y = y as i8;
  Displace {
    x: x.into(),
    y: y.into(),
  }
}

fn decode(opcode: i64) -> Verb {
  match (opcode & 0x00FF000000000000) >> 48 {
    1 => Verb::Wait,
    2 => {
      // AttemptMove
      if let Ok(code_direction) = ((opcode & 0x0000FF0000000000) >> 40).try_into() {
        if let Some(direction) = decode_direction(code_direction) {
          return Verb::AttemptMove(direction);
        }
      }
      Verb::Wait
    }
    3 => {
      // GetMaterials
      if let Ok(code_neighbor) = ((opcode & 0x0000FF0000000000) >> 40).try_into() {
        if let Some(neighbor) = decode_neighbor(code_neighbor) {
          if let Ok(code_mat) = ((opcode & 0x00FFFFFFFF0000) >> 16).try_into() {
            return Verb::GetMaterials(neighbor, decode_materials(code_mat));
          }
        }
      }
      Verb::Wait
    }
    4 => {
      // DropMaterials
      if let Ok(code_neighbor) = ((opcode & 0x0000FF0000000000) >> 40).try_into() {
        if let Some(neighbor) = decode_neighbor(code_neighbor) {
          if let Ok(code_mat) = ((opcode & 0x000000FFFFFFFF0000) >> 16).try_into() {
            return Verb::DropMaterials(neighbor, decode_materials(code_mat));
          }
        }
      }
      Verb::Wait
    }
    5 => {
      // Shoot
      let code_displace: u16 = ((opcode & 0x0000FFFF00000000) >> 32).try_into().unwrap();
      Verb::Shoot(decode_displace(code_displace))
    }
    6 => {
      // Drill
      if let Ok(code_direction) = ((opcode & 0x0000FF0000000000) >> 40).try_into() {
        if let Some(direction) = decode_direction(code_direction) {
          return Verb::Drill(direction);
        }
      }
      Verb::Wait
    }
    7 => {
      // Construct
      if let Ok(template) = ((opcode & 0x0000FF0000000000) >> 40).try_into() {
        if let Ok(code_direction) = ((opcode & 0x000000FF00000000) >> 32).try_into() {
          if let Some(direction) = decode_direction(code_direction) {
            return Verb::Construct(template, direction);
          }
        }
      }
      Verb::Wait
    }
    // TODO set message
    _ => Verb::Wait,
  }
}

#[derive(Debug, Snafu)]
pub enum BrainError {
  #[snafu(display("Could not create module {:}", index))]
  CreateModule { source: CompileError, index: usize },
  #[snafu(display("Could not create instance {:}", index))]
  CreateInstance {
    source: InstantiationError,
    index: usize,
  },
  #[snafu(display("Could not load wasm code {:}", index))]
  LoadWasm {
    source: std::io::Error,
    index: usize,
  },
}

#[derive(Debug, Snafu)]
pub enum ExecutionError {
  #[snafu(display("No execute function in template {:}: {:}", index, source))]
  NoExecute { source: ExportError, index: usize },
  #[snafu(display("Error executing code for bot {:}", index))]
  Runtime { source: RuntimeError, index: usize },
}

pub struct Brains {
  store: Store,
  blue_modules: [Module; NUM_TEMPLATES],
  red_modules: [Module; NUM_TEMPLATES],
  blue_brains: HashMap<Id, Instance>,
  red_brains: HashMap<Id, Instance>,
}

impl Brains {
  pub fn new(id_vec: Vec<usize>) -> Result<Self, BrainError> {
    let mut store = Store::default();

    let wasm_bytes = std::fs::read("./target/wasm32-unknown-unknown/release/zigzag.wasm")
      .context(LoadWasmSnafu { index: 0 as usize })?;
    let module =
      Module::new(&store, wasm_bytes).context(CreateModuleSnafu { index: 0 as usize })?;

    // The module doesn't import anything, so we create an empty import object.
    let blue_modules: [Module; NUM_TEMPLATES] = init_array(|_| module.clone());
    let red_modules: [Module; NUM_TEMPLATES] = init_array(|_| module.clone());
    let import_object = imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)
      .context(CreateInstanceSnafu { index: 0 as usize })?;
    let mut blue_brains: HashMap<Id, Instance> = HashMap::new();
    for id in id_vec {
      blue_brains.insert(id, instance.clone());
    }
    let red_brains: HashMap<Id, Instance> = HashMap::new();
    Ok(Brains {
      store,
      blue_modules,
      red_modules,
      blue_brains,
      red_brains,
    })
  }

  pub fn get_command(&mut self, id: usize) -> Result<Command, ExecutionError> {
    let execute = self
      .blue_brains
      .get(&id)
      .unwrap()
      .exports
      .get_function("execute")
      .context(NoExecuteSnafu { index: id })?;
    let result = execute
      .call(&mut self.store, &[])
      .context(RuntimeSnafu { index: id })?;
    let value = match result[0] {
      Value::I64(r) => r,
      _ => 0x0001000000000000,
    };
    Ok(Command {
      entity_id: id,
      verb: decode(value),
    })
  }
}

fn random_material(rng: &mut ChaCha8Rng) -> Materials {
  let material_type = rng.gen_range(0..4);
  Materials {
    carbon: if material_type == 0 { 1 } else { 0 },
    silicon: if material_type == 1 { 1 } else { 0 },
    plutonium: if material_type == 2 { 1 } else { 0 },
    copper: if material_type == 3 { 1 } else { 0 },
  }
}

fn random_direction(rng: &mut ChaCha8Rng) -> Direction {
  match rng.gen_range(0..4) {
    0 => Direction::North,
    1 => Direction::East,
    2 => Direction::South,
    _ => Direction::West,
  }
}

fn random_neighbor(rng: &mut ChaCha8Rng) -> Neighbor {
  match rng.gen_range(0..5) {
    0 => Neighbor::North,
    1 => Neighbor::East,
    2 => Neighbor::South,
    3 => Neighbor::West,
    _ => Neighbor::Here,
  }
}

fn random_vicinity(rng: &mut ChaCha8Rng) -> Displace {
  Displace::new(
    rng.gen_range(0..11) as i64 - 5,
    rng.gen_range(0..11) as i64 - 5,
  )
}

pub fn random_verb(rng: &mut ChaCha8Rng) -> Verb {
  match rng.gen_range(0..7) {
    0 => Verb::AttemptMove(random_direction(rng)),
    1 => Verb::GetMaterials(random_neighbor(rng), random_material(rng)),
    2 => Verb::DropMaterials(random_neighbor(rng), random_material(rng)),
    3 => Verb::Shoot(random_vicinity(rng)),
    4 => Verb::Construct(rng.gen_range(0..NUM_TEMPLATES), random_direction(rng)),
    _ => Verb::Drill(random_direction(rng)),
  }
}
