extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use futures::executor::block_on;
use macroquad::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use super::canvas::{draw_floor, draw_mat_map, draw_materials, draw_template_at};
use super::new_bf::NewBF;
use super::ui::{
  build_incrementer, plus_minus, split, trim_margins, Button, ButtonPanel, Input, Rect, Sign, Ui,
};
use super::view::{PlayState, View, ViewState};
use crate::state::bf::{join_tiles, load_level_file, load_squad_file, BFState, EntityState};
use crate::state::constants::{HEIGHT, WIDTH};
use crate::state::entity::Team;
use crate::state::geometry::{board_iterator, half_board_iterator};
use crate::state::state::Tile;

const SMOKE: macroquad::color::Color = Color::new(0.0, 0.0, 0.0, 0.3);

const XDISPL: f32 = 800.0;
const YDISPL: f32 = 30.0;

#[derive(Clone, Debug)]
pub enum Command {
  NewSquadForBF(usize),
  BuildBattle(usize),
  ChangeBF(Sign),
  ChangeSquad(Team, Sign),
  Start,
  Exit,
}

#[derive(Clone, Debug)]
pub struct BattleParams {
  level: usize,
  blue_index: usize,
  red_index: usize,
  blue_squad: BFState,
  red_squad: BFState,
  joined_tiles: Vec<Tile>,
}

#[derive(Debug)]
pub struct ShowingDetails {
  level: usize,
  level_state: BFState,
  has_squads: bool,
  joined_tiles: Vec<Tile>,
}

#[derive(Debug)]
pub enum LoadBFState {
  NoFiles,
  Showing(ShowingDetails),
  SelectingSquads(BattleParams),
  NewSquad(NewBF),
  View(View),
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
      LoadBFState::Showing(ShowingDetails {
        level: s,
        level_state: _,
        has_squads,
        ..
      }) => {
        let rects: Vec<Rect> = split(rect, vec![0.0, 0.5], vec![0.0, 0.3, 0.45, 0.6]);
        panel.append(&mut build_incrementer::<Command>(
          &rects[0],
          "Level".to_string(),
          *s,
          Command::ChangeBF(Sign::Plus),
          Command::ChangeBF(Sign::Minus),
        ));
        match has_squads {
          true => {
            panel.push(Button::<Command>::new(
              trim_margins(rects[1].clone(), 0.1, 0.1, 0.1, 0.1),
              ("Battle".to_string(), Command::BuildBattle(*s), true, false),
            ));
          }
          false => {
            panel.push(Button::<Command>::new(
              trim_margins(rects[1].clone(), 0.1, 0.1, 0.1, 0.1),
              (
                "No squads".to_string(),
                Command::NewSquadForBF(*s),
                false,
                false,
              ),
            ));
          }
        }
        panel.push(Button::<Command>::new(
          trim_margins(rects[2].clone(), 0.1, 0.1, 0.1, 0.1),
          (
            "New squad".to_string(),
            Command::NewSquadForBF(*s),
            true,
            false,
          ),
        ));
      }
      LoadBFState::SelectingSquads(BattleParams {
        blue_index,
        red_index,
        ..
      }) => {
        let rects: Vec<Rect> = split(rect, vec![0.0, 0.5, 1.0], vec![0.0, 0.3]);
        panel.append(&mut build_incrementer::<Command>(
          &rects[0],
          "Blue Squad".to_string(),
          *blue_index,
          Command::ChangeSquad(Team::Blue, Sign::Plus),
          Command::ChangeSquad(Team::Blue, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
          &rects[1],
          "Red Squad".to_string(),
          *red_index,
          Command::ChangeSquad(Team::Red, Sign::Plus),
          Command::ChangeSquad(Team::Red, Sign::Minus),
        ));
        panel.push(Button::<Command>::new(
          trim_margins(self.rect.clone(), 0.6, 0.3, 0.1, 0.7),
          ("Start Battle".to_string(), Command::Start, true, false),
        ));
      }
      LoadBFState::View(_) => {}
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
    let rects: Vec<Rect> = split(&left_rect, vec![0.0, 0.6, 1.0], vec![0.0, 0.8, 1.0]);
    if let LoadBFState::SelectingSquads(BattleParams {
      blue_squad,
      red_squad,
      joined_tiles,
      ..
    }) = &mut self.state
    {
      *joined_tiles = join_tiles(blue_squad, red_squad);
    }
    self.panel = self.build_panel(&rects[0]);
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
      // state: match load_level_file(0) {
      //   Some(state) => LoadBFState::Showing(ShowingDetails {
      //     level: 0,
      //     level_state: state.clone(),
      //     has_squads: load_squad_file(0, 0).is_some(),
      //     joined_tiles: join_tiles(&state, &state),
      //   }),
      //   None => LoadBFState::NoFiles,
      // },
      state: LoadBFState::View(View::new(
        rect.clone(),
        ViewState {
          level: 0,
          blue_squad_number: 0,
          red_squad_number: 0,
          current_frame: 0,
          seconds: get_time(),
          finished: false,
          play_state: PlayState::Paused,
        },
      )),
      floor,
      tileset,
      panel: ButtonPanel::new(rect, (vec![], vec![], vec![], vec![], vec![])),
    };
    load_bf.update_main_panel();
    load_bf
  }

  async fn draw(&self) {
    match &self.state {
      LoadBFState::Showing(ShowingDetails {
        level_state: bf_state,
        joined_tiles: jt,
        ..
      }) => {
        draw_floor(XDISPL, YDISPL, &self.tileset, &self.floor).await;
        draw_mat_map(&jt, XDISPL, YDISPL, &self.tileset).await;
        for pos in half_board_iterator() {
          if let Some(id) = &bf_state.get_tiles()[pos.to_index()].entity_id {
            if let EntityState::Entity(e, _) = &bf_state.get_entities()[*id] {
              draw_template_at(
                &e.clone().try_into().unwrap(),
                XDISPL,
                YDISPL,
                pos,
                Team::Blue,
                &self.tileset,
              )
              .await;
              let f = e.clone();
              draw_template_at(
                &f.try_into().unwrap(),
                XDISPL,
                YDISPL,
                pos.invert(),
                Team::Red,
                &self.tileset,
              )
              .await;
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
      LoadBFState::SelectingSquads(BattleParams {
        blue_squad,
        red_squad,
        joined_tiles,
        ..
      }) => {
        draw_floor(XDISPL, YDISPL, &self.tileset, &self.floor).await;
        for pos in board_iterator() {
          let tile = joined_tiles[pos.to_index()].clone();
          if let Some(id) = tile.entity_id {
            let mut entity = if pos.y < HEIGHT / 2 {
              red_squad.get_entities()[id].clone()
            } else {
              blue_squad.get_entities()[id].clone()
            };
            draw_materials(tile.materials.clone(), XDISPL, YDISPL, pos, &self.tileset).await;
            if let EntityState::Entity(e, _) = &mut entity {
              draw_template_at(
                &e.clone().try_into().unwrap(),
                XDISPL,
                YDISPL,
                pos,
                if pos.is_bottom() {
                  Team::Blue
                } else {
                  Team::Red
                },
                &self.tileset,
              )
              .await;
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
      LoadBFState::View(v) => v.draw().await,
      LoadBFState::NewSquad(n) => {
        n.draw().await;
      }
    }
  }

  fn process_input(&mut self, input: Input) -> Option<()> {
    let command = &self.panel.process_input(input.clone());
    match &mut self.state {
      LoadBFState::Showing(ShowingDetails {
        level: s,
        level_state: bf_state,
        has_squads,
        ..
      }) => match command {
        Some(Command::NewSquadForBF(level)) => {
          self.state = LoadBFState::NewSquad(NewBF::new(
            self.rect.clone(),
            load_level_file(*level).map(|bf| (bf, *level)),
          ));
        }
        Some(Command::ChangeBF(sign)) => {
          let s_prime = plus_minus(*s, *sign);
          match load_level_file(s_prime) {
            Some(state) => {
              *bf_state = state;
              *s = s_prime;
              *has_squads = load_squad_file(s_prime, 0).is_some();
            }
            _ => {}
          }
        }
        Some(Command::ChangeSquad(_, _)) => {}
        Some(Command::Exit) => {
          return Some(());
        }
        Some(Command::BuildBattle(level)) => {
          if let Some(sqd) = load_squad_file(*level, 0) {
            self.state = LoadBFState::SelectingSquads(BattleParams {
              level: *level,
              blue_index: 0,
              red_index: 0,
              blue_squad: sqd.clone(),
              red_squad: sqd.clone(),
              joined_tiles: join_tiles(&sqd, &sqd),
            })
          }
        }
        Some(_) => {}
        None => {}
      },
      LoadBFState::NoFiles => {
        if let Some(Command::Exit) = command {
          return Some(());
        }
      }
      LoadBFState::SelectingSquads(ref mut battle_params) => match command {
        Some(Command::ChangeSquad(team, sign)) => {
          let BattleParams {
            level,
            blue_index,
            red_index,
            blue_squad,
            red_squad,
            ..
          } = battle_params;
          let (relevant_squad, relevant_index) = match team {
            Team::Blue => (blue_squad, blue_index),
            Team::Red => (red_squad, red_index),
            _ => unimplemented!(),
          };
          let s_prime = plus_minus(*relevant_index, *sign);
          match load_squad_file(*level, s_prime) {
            Some(state) => {
              *relevant_squad = state;
              *relevant_index = s_prime;
            }
            _ => {}
          }
          self.state = LoadBFState::SelectingSquads(battle_params.clone());
          if let Some(Command::Exit) = command {
            return Some(());
          }
        }
        Some(Command::Start) => {
          self.state = LoadBFState::View(View::new(
            self.rect.clone(),
            ViewState {
              level: 0,
              blue_squad_number: 0,
              red_squad_number: 0,
              current_frame: 0,
              seconds: 0.0,
              finished: false,
              play_state: PlayState::Paused,
            },
          ));
        }
        Some(Command::Exit) => {
          return Some(());
        }
        _ => {}
      },
      LoadBFState::NewSquad(n) => match n.process_input(input.clone()) {
        Some(()) => {
          if let Some(state) = load_level_file(0) {
            self.state = LoadBFState::Showing(ShowingDetails {
              level: 0,
              level_state: state.clone(),
              has_squads: load_squad_file(0, 0).is_some(),
              joined_tiles: join_tiles(&state, &state),
            });
          }
        }
        _ => {}
      },
      LoadBFState::View(v) => match v.process_input(input.clone()) {
        Some(()) => {
          if let Some(state) = load_level_file(0) {
            self.state = LoadBFState::Showing(ShowingDetails {
              level: 0,
              level_state: state.clone(),
              has_squads: load_squad_file(0, 0).is_some(),
              joined_tiles: join_tiles(&state, &state),
            });
            self.update_main_panel();
            return None;
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
