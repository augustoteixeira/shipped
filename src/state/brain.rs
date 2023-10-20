extern crate rand;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use snafu::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;

use init_array::init_array;
use std::collections::HashMap;
use wasmer::{
  imports, CompileError, ExportError, Function, FunctionEnv, FunctionEnvMut, Instance,
  InstantiationError, Module, RuntimeError, Store, Value,
};

use crate::state::constants::{NUM_TEMPLATES, RANGE};
use crate::state::encoder::{
  decode_displace, decode_verb, encode_coord, encode_materials, encode_view, ViewResult,
};
use crate::state::entity::Team;
use crate::state::geometry::{add_displace, Pos};
use crate::state::state::{Command, Id, State, StateError};

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
  #[snafu(display("No entity in state"))]
  NoEntity { source: StateError },
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
  blue_modules: [Option<Module>; NUM_TEMPLATES],
  red_modules: [Option<Module>; NUM_TEMPLATES],
  blue_brains: HashMap<Id, Option<Instance>>,
  red_brains: HashMap<Id, Option<Instance>>,
}

#[derive(Clone)]
struct Env {
  state: Arc<Mutex<State>>,
  current: Arc<Mutex<Id>>,
  rng: Arc<Mutex<StdRng>>,
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

// the function that the bot uses to get its coordinate from the enviroment
fn get_rand(env: FunctionEnvMut<Env>) -> u32 {
  let mut rng = env.data().rng.lock().unwrap();
  rng.gen_range(0..0xFFFFFFFF)
}

// the function that the bot uses to get the materials in a tile around it
fn get_materials(env: FunctionEnvMut<Env>, encoded_displace: u16) -> i64 {
  let state = env.data().state.lock().unwrap();
  let current = env.data().current.lock().unwrap();
  let entity = state.get_entity_by_id(*current).unwrap();
  let pos = entity.pos;
  let displ = match entity.team {
    Team::Blue => decode_displace(encoded_displace),
    Team::Red => decode_displace(encoded_displace).invert(),
  };
  if (displ.x < -(RANGE as i64))
    && (displ.x > RANGE as i64)
    && (displ.y < -(RANGE as i64))
    && (displ.y > RANGE as i64)
  {
    return 0x0000000000000000;
  }
  match add_displace(pos, &displ) {
    Err(_) => {
      return 0x0000000000000000;
    }
    Ok(target_pos) => {
      let materials = state.get_floor_mat(target_pos);
      return (encode_materials(materials.clone()) as i64) + 0x0001000000000000;
    }
  }
}

// the function that the bot uses to get the bot in a tile around it
fn get_entity(env: FunctionEnvMut<Env>, encoded_displace: u16) -> i64 {
  let state = env.data().state.lock().unwrap();
  let current = env.data().current.lock().unwrap();
  let entity = state.get_entity_by_id(*current).unwrap();
  let pos = entity.pos;
  let displ = match entity.team {
    Team::Blue => decode_displace(encoded_displace),
    Team::Red => decode_displace(encoded_displace).invert(),
  };
  if (displ.x < -(RANGE as i64))
    && (displ.x > RANGE as i64)
    && (displ.y < -(RANGE as i64))
    && (displ.y > RANGE as i64)
  {
    return encode_view(ViewResult::OutOfBounds);
  }
  encode_view(match state.get_visible(pos, &displ) {
    None => ViewResult::OutOfBounds,
    Some(viewed_pos) => match state.get_tile(viewed_pos).entity_id {
      None => ViewResult::Empty,
      Some(viewed_entity_id) => match state.get_entity_by_id(viewed_entity_id) {
        Err(_) => ViewResult::Error,
        Ok(viewed_entity) => ViewResult::Entity(match entity.team {
          Team::Blue => viewed_entity.clone().into(),
          Team::Red => {
            let mut inverted_color = viewed_entity.clone();
            inverted_color.team = inverted_color.team.invert();
            inverted_color.into()
          }
        }),
      },
    },
  })
}

impl Brains {
  pub fn new(state: Arc<Mutex<State>>) -> Result<Self, BrainError> {
    let id_vec = state.lock().unwrap().get_entities_ids();
    let mut store = Store::default();

    let current = Arc::new(Mutex::new(0));
    let rng = Arc::new(Mutex::new(StdRng::from_entropy()));
    let env = FunctionEnv::new(
      &mut store,
      Env {
        state: state.clone(),
        current: current.clone(),
        rng: rng.clone(),
      },
    );

    let import_object = imports! {
              "env" => {
                  "get_coord" => Function::new_typed_with_env
                  (&mut store, &env, get_coord),
                  "get_materials" => Function::new_typed_with_env
                  (&mut store, &env, get_materials),
                  "get_entity" => Function::new_typed_with_env
                  (&mut store, &env, get_entity),
                  "get_rand" => Function::new_typed_with_env
                  (&mut store, &env, get_rand)
              },
    };

    //let code_vec: HashMap<u128, String> = get_code_vec();

    let mut blue_modules: [Option<Module>; NUM_TEMPLATES] = init_array(|_| None);
    let mut blue_brain_index: HashMap<String, usize> = HashMap::new();
    let mut red_modules: [Option<Module>; NUM_TEMPLATES] = init_array(|_| None);
    let mut red_brain_index: HashMap<String, usize> = HashMap::new();
    {
      let blue_templates = state.lock().unwrap().blue_templates.clone();
      println!("{:?}", blue_templates);
    }

    for (index, template) in state
      .lock()
      .unwrap()
      .blue_templates
      .clone()
      .iter()
      .enumerate()
    {
      if let Some(template_entity) = template {
        if let Some(brain) = template_entity.brain.clone() {
          // println!(
          //   "index: {index}, brain.code = {:?}, code_vec[index] = {:?}",
          //   brain.code_index, code_vec[index]
          // );
          let wasm_bytes = std::fs::read(Path::new(&brain.code_name))
            .context(LoadWasmSnafu { index: 0 as usize })?;
          blue_brain_index.insert(brain.code_name, index);
          let module =
            Module::new(&store, wasm_bytes).context(CreateModuleSnafu { index: 0 as usize })?;
          blue_modules[index] = Some(module.clone());
        }
      }
    }

    for (index, template) in state
      .lock()
      .unwrap()
      .red_templates
      .clone()
      .iter()
      .enumerate()
    {
      if let Some(template_entity) = template {
        if let Some(brain) = template_entity.brain.clone() {
          let wasm_bytes = std::fs::read(Path::new(&brain.code_name))
            .context(LoadWasmSnafu { index: 0 as usize })?;
          red_brain_index.insert(brain.code_name, index);
          let module =
            Module::new(&store, wasm_bytes).context(CreateModuleSnafu { index: 0 as usize })?;
          red_modules[index] = Some(module.clone());
        }
      }
    }

    let mut blue_brains: HashMap<Id, Option<Instance>> = HashMap::new();
    let mut red_brains: HashMap<Id, Option<Instance>> = HashMap::new();

    for id in id_vec {
      let state_guard = state.lock().unwrap();
      let entity = state_guard.get_entity_by_id(id).context(NoEntitySnafu {})?;
      let (module_vec, brains, brain_index): (
        &[Option<Module>; NUM_TEMPLATES],
        &mut HashMap<Id, Option<Instance>>,
        &HashMap<String, usize>,
      ) = match entity.team {
        Team::Blue => (&blue_modules, &mut blue_brains, &blue_brain_index),
        Team::Red => (&red_modules, &mut red_brains, &red_brain_index),
      };
      let optional_module = match entity.brain.clone() {
        None => None,
        Some(brain) => {
          // TODO: this is terrible, so one needs to fix it. The problem is that
          // once we instantiate an entity from a template, we erase the information
          // about which template we used.
          module_vec[*brain_index.get(&brain.code_name).unwrap()].clone()
        }
      };
      let instance: Option<Instance> = match optional_module {
        None => None,
        Some(module) => Some(
          Instance::new(&mut store, &module, &import_object)
            .context(CreateInstanceSnafu { index: 0 as usize })?,
        ),
      };
      brains.insert(id, instance.clone());
    }

    Ok(Brains {
      store,
      env: Env {
        state: state.clone(),
        current: current.clone(),
        rng: rng.clone(),
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
    let current_brain = match team {
      Team::Blue => &self.blue_brains,
      Team::Red => &self.red_brains,
    };
    match current_brain.get(&id) {
      None | Some(None) => {
        return Ok(Command {
          entity_id: id,
          verb: super::state::Verb::Wait,
        })
      }
      Some(Some(instance)) => {
        let execute = instance
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
            Team::Blue => decode_verb(value),
            Team::Red => decode_verb(value).invert(),
          },
        })
      }
    }
  }
}
