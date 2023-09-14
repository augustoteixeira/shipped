extern crate rand;
extern crate rand_chacha;
use snafu::prelude::*;

use init_array::init_array;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use wasmer::{imports, CompileError, Instance, InstantiationError, Module, Store, Value};

use crate::state::constants::NUM_TEMPLATES;
use crate::state::geometry::{Direction, Displace, Neighbor};
use crate::state::materials::Materials;
use crate::state::state::{Command, Id, Verb};

#[derive(Debug, Snafu)]
pub enum BrainError {
  #[snafu(display("Could not create module {:}", index))]
  CreateModule { source: CompileError, index: usize },
  #[snafu(display("Could not create instance {:}", index))]
  CreateInstance {
    source: InstantiationError,
    index: usize,
  },
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
    let module_wat = r#"
    (module
    (type $t0 (func (param i32) (result i32)))
    (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
        get_local $p0
        i32.const 1
        i32.add))
    "#;

    let mut store = Store::default();
    let module =
      Module::new(&store, &module_wat).context(CreateModuleSnafu { index: 0 as usize })?;
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

  pub fn get_command(&self, id: usize) -> Command {
    Command {
      entity_id: id,
      verb: Verb::Wait,
    }
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
