extern crate rand;
extern crate rand_chacha;
use snafu::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;

use init_array::init_array;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use wasmer::{
  imports, CompileError, ExportError, Function, FunctionEnv, FunctionEnvMut, Instance,
  InstantiationError, Module, RuntimeError, Store, Value,
};

use crate::state::constants::NUM_TEMPLATES;
use crate::state::encoder::{decode, decode_displace, encode_coord};
use crate::state::entity::Team;
use crate::state::geometry::{add_displace, Direction, Displace, Neighbor, Pos};
use crate::state::materials::Materials;
use crate::state::state::{Command, Id, State, Verb};

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
  env: Env,
  blue_modules: [Module; NUM_TEMPLATES],
  red_modules: [Module; NUM_TEMPLATES],
  blue_brains: HashMap<Id, Instance>,
  red_brains: HashMap<Id, Instance>,
}

#[derive(Clone)]
struct Env {
  state: Arc<Mutex<State>>,
  current: Arc<Mutex<Id>>,
}

fn get_unencoded_coord(env: FunctionEnvMut<Env>) -> Pos {
  let state = env.data().state.lock().unwrap();
  let current = env.data().current.lock().unwrap();
  let entity = state.get_entity_by_id(*current).unwrap();
  match entity.team {
    Team::Blue => entity.pos,
    Team::Red => entity.pos.invert(),
  }
}

// the function that the bot uses to get its coordinate from the enviroment
fn get_coord(env: FunctionEnvMut<Env>) -> u32 {
  let pos = get_unencoded_coord(env);
  encode_coord(pos.x, pos.y)
}

struct ViewingTile {}

fn encode_tile(tile: Option<ViewingTile>) -> i64 {
  match tile {
    None => 0x0000000000000000,
    Some(_) => 0x0000000000000001,
  }
}

// the function that the bot uses to get a tile around it
fn get_tile(env: FunctionEnvMut<Env>, encoded_displace: u16) -> i64 {
  let displ = decode_displace(encoded_displace);
  let pos = get_unencoded_coord(env);
  let tile: Option<ViewingTile> = match add_displace(pos, &displ) {
    Err(_) => None,
    Ok(_pos) => Some(ViewingTile {}),
  };
  encode_tile(tile)
}

impl Brains {
  pub fn new(state: Arc<Mutex<State>>) -> Result<Self, BrainError> {
    let id_vec = state.lock().unwrap().get_entities_ids();
    let mut store = Store::default();

    let current = Arc::new(Mutex::new(0));
    let env = FunctionEnv::new(
      &mut store,
      Env {
        state: state.clone(),
        current: current.clone(),
      },
    );

    let import_object = imports! {
              "env" => {
                  "get_coord" => Function::new_typed_with_env
                  (&mut store, &env, get_coord),
                  "get_tile" => Function::new_typed_with_env
                  (&mut store, &env, get_tile)
              },
    };

    let wasm_bytes = std::fs::read("./target/wasm32-unknown-unknown/release/bounce.wasm")
      .context(LoadWasmSnafu { index: 0 as usize })?;
    let module =
      Module::new(&store, wasm_bytes).context(CreateModuleSnafu { index: 0 as usize })?;

    let blue_modules: [Module; NUM_TEMPLATES] = init_array(|_| module.clone());
    let red_modules: [Module; NUM_TEMPLATES] = init_array(|_| module.clone());
    //let import_object = imports! {};
    let mut blue_brains: HashMap<Id, Instance> = HashMap::new();
    for id in id_vec {
      let instance = Instance::new(&mut store, &module, &import_object)
        .context(CreateInstanceSnafu { index: 0 as usize })?;
      blue_brains.insert(id, instance.clone());
    }
    let red_brains: HashMap<Id, Instance> = HashMap::new();
    Ok(Brains {
      store,
      env: Env {
        state: state.clone(),
        current: current.clone(),
      },
      blue_modules,
      red_modules,
      blue_brains,
      red_brains,
    })
  }

  pub fn get_command(&mut self, id: usize) -> Result<Command, ExecutionError> {
    // in our enviroment, we first update the current bot
    let mut current = self.env.current.lock().unwrap();
    *current = id;
    let state = self.env.state.lock().unwrap();
    let entity = state.get_entity_by_id(*current).unwrap();
    let team = entity.team.clone();
    drop(current);
    drop(state);
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
      verb: match team {
        Team::Blue => decode(value),
        Team::Red => decode(value).invert(),
        _ => unreachable!(),
      },
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
