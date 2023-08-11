extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use futures::executor::block_on;
use macroquad::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
//use serde::{Deserialize, Serialize};
//use std::fs::File;
//use std::io::Write;
//use std::path::Path;

use super::canvas::{draw_entity, draw_floor, draw_mat_map};
//use super::entity_edit::{EntityEdit, EntityEditCommand};
use super::ui::{
  build_incrementer, plus_minus, split, trim_margins, Button, ButtonPanel, Input, Rect, Sign, Ui,
};
use crate::state::constants::{HEIGHT, NUM_TEMPLATES, WIDTH};
//use crate::state::entity::{Mix, MixEntity, MovementType, Team};
use crate::state::geometry::{board_iterator, Pos};
//use crate::state::materials::Materials;
//use crate::state::state::Tile;

const SMOKE: macroquad::color::Color = Color::new(0.0, 0.0, 0.0, 0.3);

const XDISPL: f32 = 800.0;
const YDISPL: f32 = 30.0;

#[derive(Clone, Debug)]
pub enum Command {
  SelectBF(usize),
  ChangeBF(Sign),
  Exit,
}

#[derive(Clone, Debug)]
pub enum LoadBFState {
  NoFiles,
  Selected(usize),
}

pub struct LoadBF {
  rect: Rect,
  state: LoadBFState,
  floor: [usize; WIDTH * HEIGHT],
  tileset: Texture2D,
  panel: ButtonPanel<Command>,
}

impl LoadBF {
  fn build_panel(&self, rect: &Rect) -> ButtonPanel<Command> {
    match &self.state {
      LoadBFState::NoFiles => unimplemented!(),
      LoadBFState::Selected(s) => {
        let mut rects: Vec<Rect> = split(rect, vec![0.0, 1.0], vec![0.0, 0.5, 1.0]);
        let mut panel: ButtonPanel<Command> = build_incrementer::<Command>(
          &rects[0],
          "Level".to_string(),
          *s,
          Command::ChangeBF(Sign::Plus),
          Command::ChangeBF(Sign::Minus),
        );
        panel.push(Button::<Command>::new(
          rects[1].clone(),
          ("Load".to_string(), Command::SelectBF(*s), true, false),
        ));
        panel
      }
    }
  }

  fn update_main_panel(&mut self) {
    let left_rect = trim_margins(
      split(&self.rect, vec![0.0, 0.45, 1.0], vec![0.0, 1.0])[0].clone(),
      0.05,
      0.05,
      0.05,
      0.05,
    );
    let rects: Vec<Rect> = split(&left_rect, vec![0.0, 0.3, 1.0], vec![0.0, 0.8, 1.0]);
    let mut panel = self.build_panel(&rects[0]);
    panel.push(Button::<Command>::new(
      trim_margins(rects[3].clone(), 0.3, 0.3, 0.3, 0.3),
      ("Back".to_string(), Command::Exit, true, false),
    ));
    self.panel = panel;
  }
}

#[async_trait]
impl Ui for LoadBF {
  type Command = ();
  type Builder = ();

  fn new(rect: Rect, _: ()) -> Self {
    let tileset = block_on(load_texture("assets/tileset.png")).unwrap();
    let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(25).try_into().unwrap();
    let mut floor = [0; WIDTH * HEIGHT];
    for i in 0..(WIDTH * HEIGHT) {
      floor[i] = rng.gen_range(0..7);
    }
    let mut load_bf = LoadBF {
      rect: rect.clone(),
      state: LoadBFState::Selected(0),
      floor,
      tileset,
      panel: ButtonPanel::new(rect, (vec![], vec![], vec![], vec![], vec![])),
    };
    load_bf.update_main_panel();
    load_bf
  }

  async fn draw(&self) {
    self.panel.draw().await;
    draw_floor(XDISPL, YDISPL, &self.tileset, &self.floor).await;
    //draw_mat_map(&self.state.tiles, XDISPL, YDISPL, &self.tileset).await;
    for i in 0..=WIDTH {
      draw_line(
        XDISPL + (i as f32) * 16.0,
        YDISPL,
        XDISPL + (i as f32) * 16.0,
        YDISPL + (60.0 * 16.0),
        1.0,
        SMOKE,
      );
      draw_line(
        XDISPL,
        YDISPL + (i as f32) * 16.0,
        XDISPL + (60.0 * 16.0),
        YDISPL + (i as f32) * 16.0,
        1.0,
        SMOKE,
      );
    }
  }

  fn process_input(&mut self, input: Input) -> Option<()> {
    let command = &self.panel.process_input(input.clone());
    match command {
      None => {}
      Some(Command::SelectBF(level)) => {}
      Some(Command::ChangeBF(sign)) => match &mut self.state {
        LoadBFState::Selected(s) => {
          *s = plus_minus(*s, *sign);
        }
        LoadBFState::NoFiles => {}
      },
      Some(Command::Exit) => return Some(()),
    };
    self.update_main_panel();
    if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
      Some(())
    } else {
      None
    }
  }
}
