extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use macroquad::prelude::*;

use futures::executor::block_on;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use super::ui::{build_incrementer, split, trim_margins, Button, ButtonPanel, Input, Rect, Ui};
use crate::state::bf::{load_level_file, load_squad_file, BFState};
use crate::state::constants::{HEIGHT, WIDTH};
use crate::state::run::run_match;
use crate::state::state::{Frame, GameStatus, State};
use crate::ui::canvas::{draw_entity_map, draw_floor, draw_mat_map};

const XDISPL: f32 = 800.0;
const YDISPL: f32 = 30.0;

#[derive(Clone, Debug)]
pub struct ViewState {
  pub level: usize,
  pub blue_squad_number: usize,
  pub red_squad_number: usize,
  pub current_frame: usize,
  pub finished: bool,
  pub seconds: f64,
  pub speed: usize,
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
  Faster,
  Slower,
}

impl View {
  fn build_panel(&self, rect: &Rect) -> ButtonPanel<Command> {
    let mut panel: ButtonPanel<Command> =
      ButtonPanel::new(rect.clone(), (vec![], vec![], vec![], vec![], vec![]));
    let rects: Vec<Rect> = split(
      &trim_margins(rect.clone(), 0.2, 0.2, 0.2, 0.2),
      vec![0.0, 1.0],
      vec![0.0, 0.6, 0.8, 1.0],
    )
    .into_iter()
    .map(|r| trim_margins(r, 0.1, 0.1, 0.1, 0.1))
    .collect();
    panel.append(&mut build_incrementer::<Command>(
      &rects[0],
      "Speed".to_string(),
      self.view_state.speed,
      Command::Faster,
      Command::Slower,
    ));
    panel.push(Button::<Command>::new(
      rects[2].clone(),
      ("Quit".to_string(), Command::Exit, false, false),
    ));
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

    let script = run_match(&level, &blue_squad, &red_squad, 10);

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
    //match &self.view_state.play_state {
    //  PlayState::Paused => {
    self.panel.draw().await;
    //  }
    //}

    draw_floor(XDISPL, YDISPL, &self.tileset, &self.floor).await;
    draw_mat_map(&self.state.tiles, XDISPL, YDISPL, &self.tileset).await;
    draw_entity_map(&self.state, XDISPL, YDISPL, &self.tileset).await;
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
  }
  fn process_input(&mut self, input: Input) -> Option<()> {
    if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
      return Some(());
    }
    match self.panel.process_input(input) {
      Some(Command::Exit) => return Some(()),
      Some(Command::Faster) => {
        if self.view_state.speed == 0 {
          self.view_state.seconds = get_time();
        }
        self.view_state.speed += 1;
      }
      Some(Command::Slower) => {
        self.view_state.speed = self.view_state.speed.saturating_sub(1);
      }
      None => {}
    }
    loop {
      if self.view_state.speed > 0 {
        if (get_time()
          > self.view_state.seconds
            + 1.0 / ((self.view_state.speed * self.view_state.speed) as f64))
          & (self.state.game_status == GameStatus::Running)
        {
          self.view_state.seconds += 1.0 / ((self.view_state.speed * self.view_state.speed) as f64);
          if let Some(f) = self.frames.get(self.view_state.current_frame) {
            self.view_state.current_frame += 1;
            for command in f.iter() {
              if self.state.execute_command(command.clone()).is_ok() {
                //let _ = draw_command(&self.state, command).await;
              }
            }
          } else {
            self.view_state.finished = true;

            return None;
          }
        } else {
          break;
        }
      } else {
        break;
      }
    }
    self.update_main_panel();
    None
  }
}
