extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use futures::executor::block_on;
use macroquad::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::path::Path;
//use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
//use std::path::Path;

use super::canvas::{draw_entity, draw_floor, draw_mat_map};
//use super::entity_edit::{EntityEdit, EntityEditCommand};
use super::new_bf::NewBF;
use super::ui::{
  build_incrementer, plus_minus, split, trim_margins, Button, ButtonPanel, Input, Rect, Sign, Ui,
};
use crate::state::bf::{BFState, EntityState};
use crate::state::constants::{HEIGHT, WIDTH};
use crate::state::entity::Team;
//use crate::state::entity::{Mix, MixEntity, MovementType, Team};
use crate::state::geometry::{board_iterator, Pos};
//use crate::state::materials::Materials;

const SMOKE: macroquad::color::Color = Color::new(0.0, 0.0, 0.0, 0.3);

const XDISPL: f32 = 800.0;
const YDISPL: f32 = 30.0;

#[derive(Clone, Debug)]
pub enum Command {
  SelectBF(usize),
  ChangeBF(Sign),
  Exit,
}

#[derive(Debug)]
pub enum LoadBFState {
  NoFiles,
  Showing(usize, BFState),
  NewSquad(NewBF),
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
    let mut panel: ButtonPanel<Command> =
      ButtonPanel::new(self.rect.clone(), (vec![], vec![], vec![], vec![], vec![]));
    match &self.state {
      LoadBFState::NoFiles => {
        let rects: Vec<Rect> = split(
          &trim_margins(self.rect.clone(), 0.4, 0.4, 0.4, 0.4),
          vec![0.0, 1.0],
          vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0],
        );
        panel.push(Button::<Command>::new(
          rects[0].clone(),
          ("No levels".to_string(), Command::Exit, false, false),
        ));
      }
      LoadBFState::Showing(s, _) => {
        let rects: Vec<Rect> = split(rect, vec![0.0, 1.0], vec![0.0, 0.3, 0.6]);
        panel.append(&mut build_incrementer::<Command>(
          &rects[0],
          "Level".to_string(),
          *s,
          Command::ChangeBF(Sign::Plus),
          Command::ChangeBF(Sign::Minus),
        ));
        panel.push(Button::<Command>::new(
          rects[1].clone(),
          ("Load".to_string(), Command::SelectBF(*s), true, false),
        ));
      }
      LoadBFState::NewSquad(_) => {}
    }
    panel.push(Button::<Command>::new(
      trim_margins(self.rect.clone(), 0.7, 0.2, 0.1, 0.7),
      ("Main Menu".to_string(), Command::Exit, true, false),
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
    let rects: Vec<Rect> = split(&left_rect, vec![0.0, 0.3, 1.0], vec![0.0, 0.8, 1.0]);
    self.panel = self.build_panel(&rects[0]);
  }

  fn load_file(n: usize) -> Option<BFState> {
    let path = Path::new("./levels");
    let dest_filename = format!("{:05}", n);
    let mut dest = path.join(dest_filename);
    dest.set_extension("lvl");
    if dest.exists() {
      let mut file = File::open(dest).unwrap();
      let mut contents = String::new();
      file.read_to_string(&mut contents).unwrap();
      Some(serde_json::from_str(&contents).unwrap())
    } else {
      None
    }
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
    // find out if there exists file zero
    let mut load_bf = LoadBF {
      rect: rect.clone(),
      state: match Self::load_file(0) {
        Some(state) => LoadBFState::Showing(0, state),
        None => LoadBFState::NoFiles,
      },
      floor,
      tileset,
      panel: ButtonPanel::new(rect, (vec![], vec![], vec![], vec![], vec![])),
    };
    load_bf.update_main_panel();
    load_bf
  }

  async fn draw(&self) {
    match &self.state {
      LoadBFState::Showing(_, bf_state) => {
        draw_floor(XDISPL, YDISPL, &self.tileset, &self.floor).await;
        draw_mat_map(&bf_state.get_tiles(), XDISPL, YDISPL, &self.tileset).await;
        for pos in board_iterator() {
          if pos.y >= HEIGHT / 2 {
            if let Some(id) = &bf_state.get_tiles()[pos.to_index()].entity_id {
              if let EntityState::Entity(e, _) = &bf_state.get_entities()[*id] {
                draw_entity(
                  Some(&e.clone().try_into().unwrap()),
                  XDISPL,
                  YDISPL,
                  pos,
                  &self.tileset,
                )
                .await;
                let mut f = e.clone();
                f.team = Team::Red;
                draw_entity(
                  Some(&f.try_into().unwrap()),
                  XDISPL,
                  YDISPL,
                  Pos::new(WIDTH - pos.x - 1, HEIGHT - pos.y - 1),
                  &self.tileset,
                )
                .await;
              }
            }
          }
        }
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
        self.panel.draw().await;
      }
      LoadBFState::NoFiles => {
        self.panel.draw().await;
      }
      LoadBFState::NewSquad(n) => {
        n.draw().await;
      }
    }
  }

  fn process_input(&mut self, input: Input) -> Option<()> {
    let command = &self.panel.process_input(input.clone());
    match &mut self.state {
      LoadBFState::Showing(s, bf_state) => match command {
        Some(Command::SelectBF(level)) => {
          self.state =
            LoadBFState::NewSquad(NewBF::new(self.rect.clone(), Self::load_file(*level)));
        }
        Some(Command::ChangeBF(sign)) => {
          let s_prime = plus_minus(*s, *sign);
          match Self::load_file(s_prime) {
            Some(state) => {
              *bf_state = state;
              *s = s_prime;
            }
            _ => {}
          }
        }
        Some(Command::Exit) => {
          return Some(());
        }
        _ => {}
      },
      LoadBFState::NoFiles => {
        if let Some(Command::Exit) = command {
          return Some(());
        }
      }
      LoadBFState::NewSquad(n) => match n.process_input(input.clone()) {
        Some(()) => {
          if let Some(state) = Self::load_file(0) {
            self.state = LoadBFState::Showing(0, state);
          }
        }
        _ => {}
      },
    };
    self.update_main_panel();
    if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
      Some(())
    } else {
      None
    }
  }
}
