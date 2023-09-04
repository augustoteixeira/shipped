extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use macroquad::prelude::*;

use futures::executor::block_on;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use super::ui::{split, trim_margins, Button, ButtonPanel, Input, Rect, Ui};
use crate::state::bf::{build_state, load_level_file, load_squad_file, BFState};
use crate::state::constants::{HEIGHT, NUM_TEMPLATES, WIDTH};
use crate::state::geometry::{Direction, Displace, Neighbor};
use crate::state::materials::Materials;
use crate::state::state::{Command as StateCommand, Frame, Script, State, StateError, Verb};
use crate::ui::canvas::{draw_ent_map, draw_floor, draw_mat_map};

const XDISPL: f32 = 800.0;
const YDISPL: f32 = 30.0;
const FRAME_TIME: f64 = 0.05;

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

fn random_verb(rng: &mut ChaCha8Rng) -> Verb {
  match rng.gen_range(0..7) {
    0 => Verb::AttemptMove(random_direction(rng)),
    1 => Verb::GetMaterials(random_neighbor(rng), random_material(rng)),
    2 => Verb::DropMaterials(random_neighbor(rng), random_material(rng)),
    3 => Verb::Shoot(random_vicinity(rng)),
    4 => Verb::Construct(rng.gen_range(0..NUM_TEMPLATES), random_direction(rng)),
    _ => Verb::Drill(random_direction(rng)),
  }
}

async fn draw_command(state: &State, command: &StateCommand) -> Result<(), StateError> {
  let entity = state.get_entity_by_id(command.entity_id)?;
  match command.verb.clone() {
    Verb::Shoot(disp) => {
      let from = entity.pos;
      let to = State::add_displace(from, &disp)?;
      let damage = entity.get_gun_damage();
      draw_line(
        XDISPL + (16 * from.x) as f32 + 8.0,
        YDISPL + (16 * from.y) as f32 + 8.0,
        XDISPL + (16 * to.x) as f32 + 8.0,
        YDISPL + (16 * to.y) as f32 + 8.0,
        6.0 - (5.0 / (damage as f32)),
        RED,
      );
      draw_circle(
        XDISPL + (16 * to.x) as f32 + 8.0,
        YDISPL + (16 * to.y) as f32 + 8.0,
        12.0 - (11.0 / (damage as f32)),
        RED,
      );
      Ok(())
    }
    Verb::Drill(dir) => {
      let from = entity.pos;
      let to = State::add_displace(from, &dir.into())?;
      let damage = entity.get_gun_damage();
      draw_line(
        XDISPL + (16 * from.x) as f32 + 8.0,
        YDISPL + (16 * from.y) as f32 + 8.0,
        XDISPL + (16 * to.x) as f32 + 8.0,
        YDISPL + (16 * to.y) as f32 + 8.0,
        6.0 - (3.0 / (damage as f32)),
        BLUE,
      );
      draw_circle(
        XDISPL + (16 * to.x) as f32 + 8.0,
        YDISPL + (16 * to.y) as f32 + 8.0,
        12.0 - (6.0 / (damage as f32)),
        BLUE,
      );

      Ok(())
    }
    _ => Ok(()),
  }
}

#[derive(Debug, Clone)]
pub enum PlayState {
  Paused,
}

#[derive(Clone, Debug)]
pub struct ViewState {
  pub level: usize,
  pub blue_squad_number: usize,
  pub red_squad_number: usize,
  pub seconds: f64,
  pub current_frame: usize,
  pub finished: bool,
  pub play_state: PlayState,
}

#[derive(Debug)]
pub struct View {
  rect: Rect,
  view_state: ViewState,
  state: State,
  panel: ButtonPanel<Command>,
  floor: [usize; WIDTH * HEIGHT],
  tileset: Texture2D,
  frames: Vec<Frame>,
}

#[derive(Clone, Debug)]
pub enum Command {
  Exit,
}

impl View {
  fn build_panel(&self, rect: &Rect) -> ButtonPanel<Command> {
    let mut panel: ButtonPanel<Command> =
      ButtonPanel::new(rect.clone(), (vec![], vec![], vec![], vec![], vec![]));
    match &self.view_state.play_state {
      PlayState::Paused => {
        let rects: Vec<Rect> = split(
          &trim_margins(rect.clone(), 0.2, 0.2, 0.2, 0.2),
          vec![0.0, 1.0],
          vec![0.0, 0.6, 0.8, 1.0],
        )
        .into_iter()
        .map(|r| trim_margins(r, 0.1, 0.1, 0.1, 0.1))
        .collect();
        panel.push(Button::<Command>::new(
          rects[1].clone(),
          ("Start".to_string(), Command::Exit, false, false),
        ));
        panel.push(Button::<Command>::new(
          rects[2].clone(),
          ("Quit".to_string(), Command::Exit, false, false),
        ));
      }
    }
    panel
  }
  fn update_main_panel(&mut self) {
    let left_rect = trim_margins(
      split(&self.rect, vec![0.0, 0.45, 1.0], vec![0.0, 1.0])[0].clone(),
      0.05,
      0.05,
      0.05,
      0.05,
    );
    self.panel = self.build_panel(&left_rect);
  }
}

#[async_trait]
impl Ui for View {
  type Command = ();
  type Builder = ViewState;

  fn new(rect: Rect, v: ViewState) -> Self {
    let level: BFState = match load_level_file(v.level) {
      Some(level_state) => level_state,
      None => unreachable!(),
    };

    let blue_squad: BFState = match load_squad_file(v.level, v.blue_squad_number) {
      Some(blue_squad_state) => blue_squad_state,
      None => unreachable!(),
    };

    let red_squad: BFState = match load_squad_file(v.level, v.red_squad_number) {
      Some(red_squad_state) => red_squad_state,
      None => unreachable!(),
    };

    let initial_state = build_state(&level, &blue_squad, &red_squad);
    let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(17).try_into().unwrap();
    let mut state = initial_state.clone();
    let mut frames: Vec<Frame> = vec![];
    for _ in 1..1000 {
      let mut frame = vec![];
      let id_vec = state.get_entities_ids();
      for id in id_vec {
        let command = StateCommand {
          entity_id: id,
          verb: random_verb(&mut rng),
        };
        if let Ok(_) = state.execute_command(command.clone()) {
          frame.push(command.clone());
        }
      }
      frames.push(frame);
    }
    let script: Script = Script {
      genesis: initial_state,
      frames,
    };

    let state = script.genesis;
    let frames = script.frames;
    // time constants

    let tileset = block_on(load_texture("assets/tileset.png")).unwrap();
    let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(25).try_into().unwrap();
    let mut floor = [0; WIDTH * HEIGHT];
    for i in 0..(WIDTH * HEIGHT) {
      floor[i] = rng.gen_range(0..7);
    }

    let mut view: View = View {
      rect: rect.clone(),
      view_state: v,
      frames,
      state,
      panel: ButtonPanel::new(rect, (vec![], vec![], vec![], vec![], vec![])),
      tileset,
      floor,
    };
    view.update_main_panel();
    view
  }

  async fn draw(&self) {
    match &self.view_state.play_state {
      PlayState::Paused => {
        self.panel.draw().await;
      }
    }

    draw_floor(XDISPL, YDISPL, &self.tileset, &self.floor).await;
    draw_mat_map(&self.state.tiles, XDISPL, YDISPL, &self.tileset).await;
    draw_ent_map(&self.state, XDISPL, YDISPL, &self.tileset).await;
    draw_text(format!("FPS: {}", get_fps()).as_str(), 0., 16., 32., WHITE);
    draw_text(
      format!("Blue Tokens: {}", &self.state.blue_tokens).as_str(),
      200.,
      36.,
      32.,
      WHITE,
    );
    draw_text(
      format!("Red_Tokens: {}", &self.state.red_tokens).as_str(),
      200.,
      96.,
      32.,
      WHITE,
    );
    draw_text(
      format!(
        "Game status: {:?}, min tokens {}",
        &self.state.game_status, self.state.min_tokens
      )
      .as_str(),
      200.,
      156.,
      32.,
      WHITE,
    );
    // // update
    // if (get_time() > self.view_state.seconds + FRAME_TIME)
    //   & (self.state.game_status == GameStatus::Running)
    // {
    //   self.view_state.seconds += FRAME_TIME;
    //   if let Some(f) = self.frames.get(self.view_state.current_frame) {
    //     for command in f.iter() {
    //       if self.state.execute_command(command.clone()).is_ok() {
    //         let _ = draw_command(&self.state, command).await;
    //       }
    //     }
    //   } else {
    //     self.view_state.finished = true;
    //   }
    // }
  }
  fn process_input(&mut self, input: Input) -> Option<()> {
    match &mut self.view_state.play_state {
      PlayState::Paused => {
        if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
          return Some(());
        }
        match self.panel.process_input(input) {
          Some(Command::Exit) => return Some(()),
          _ => {}
        }
      }
    }
    self.update_main_panel();
    None
  }
}
