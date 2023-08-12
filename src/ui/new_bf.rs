extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use futures::executor::block_on;
use macroquad::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use super::canvas::{draw_entity, draw_floor, draw_mat_map};
use super::entity_edit::{EntityEdit, EntityEditCommand};
use super::ui::{build_incrementer, split, trim_margins, Button, ButtonPanel, Input, Rect, Ui};
use crate::state::constants::{HEIGHT, NUM_TEMPLATES, WIDTH};
use crate::state::entity::{Mix, MixEntity, MovementType, Team};
use crate::state::geometry::{board_iterator, Pos};
use crate::state::materials::Materials;
use crate::state::state::Tile;
use crate::state::utils::get_next_file_number;

const XDISPL: f32 = 800.0;
const YDISPL: f32 = 30.0;

const SMOKE: macroquad::color::Color = Color::new(0.0, 0.0, 0.0, 0.3);

fn construct_entities() -> [EntityStates; NUM_TEMPLATES] {
  [
    EntityStates::Empty,
    EntityStates::Empty,
    EntityStates::Empty,
    EntityStates::Empty,
  ]
}

#[derive(Clone, Debug)]
pub enum MatName {
  Carbon,
  Silicon,
  Plutonium,
  Copper,
}

#[derive(Clone, Debug)]
pub enum Brush {
  Carbon,
  Silicon,
  Plutonium,
  Copper,
  Bot(usize),
}

#[derive(Clone, Debug)]
pub enum Sign {
  Plus,
  Minus,
}

#[derive(Clone, Debug)]
pub enum TknButton {
  Tokens,
  MinTkns,
}

#[derive(Clone, Debug)]
pub enum Command {
  Finish,
  MatPM(MatName, Sign),
  MatBrush(MatName),
  Token(TknButton, Sign),
  MapLeftClk(Pos),
  BotNumber(usize, Sign),
  BotBrush(usize),
  BotEdit(usize),
  BotAddSubs(usize),
  BotDelete(usize),
  Save,
  BackToEdit,
  ExitWithoutSaving,
  Exit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityStates {
  Empty,
  Entity(MixEntity, usize),
}

#[derive(Clone, Debug)]
enum Screen {
  Map,
  Entity(EntityEdit, usize),
  SaveDialogue(ButtonPanel<Command>),
  DisplayFileNumber(ButtonPanel<Command>),
}

#[derive(Clone, Debug)]
pub enum NewBFType {
  BrandNew(NewBFState),
  Derived(NewBFState, NewBFState),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewBFState {
  pub materials: Materials,
  pub tokens: usize,
  pub min_tokens: usize,
  pub tiles: Vec<Tile>,
  pub entities: [EntityStates; NUM_TEMPLATES],
}

#[derive(Debug)]
pub struct NewBF {
  state: NewBFState,
  brush: Brush,
  screen: Screen,
  rect: Rect,
  floor: [usize; WIDTH * HEIGHT],
  tileset: Texture2D,
  panel: ButtonPanel<Command>,
  new_type: NewBFType,
}

impl NewBF {
  fn build_material_panel(&self, rect: &Rect) -> ButtonPanel<Command> {
    let mut rects: Vec<Rect> = split(
      &rect.clone(),
      vec![0.0, 0.25, 0.5, 0.75, 1.0],
      vec![0.0, 0.75],
    );
    let mut panel: ButtonPanel<Command> = build_incrementer::<Command>(
      &rects[0],
      "Carbon".to_string(),
      self.state.materials.carbon,
      Command::MatPM(MatName::Carbon, Sign::Plus),
      Command::MatPM(MatName::Carbon, Sign::Minus),
    );
    panel.append(&mut build_incrementer::<Command>(
      &rects[1],
      "Silicon".to_string(),
      self.state.materials.silicon,
      Command::MatPM(MatName::Silicon, Sign::Plus),
      Command::MatPM(MatName::Silicon, Sign::Minus),
    ));
    panel.append(&mut build_incrementer::<Command>(
      &rects[2],
      "Pluton.".to_string(),
      self.state.materials.plutonium,
      Command::MatPM(MatName::Plutonium, Sign::Plus),
      Command::MatPM(MatName::Plutonium, Sign::Minus),
    ));
    panel.append(&mut build_incrementer::<Command>(
      &rects[3],
      "Copper".to_string(),
      self.state.materials.copper,
      Command::MatPM(MatName::Copper, Sign::Plus),
      Command::MatPM(MatName::Copper, Sign::Minus),
    ));
    // Material brush buttons
    rects = split(
      &rect.clone(),
      vec![0.0, 0.25, 0.5, 0.75, 1.0],
      vec![0.75, 1.0],
    )
    .into_iter()
    .collect();
    let labels = vec!["Use".to_string(); 4];
    let commands = vec![
      Command::MatBrush(MatName::Carbon),
      Command::MatBrush(MatName::Silicon),
      Command::MatBrush(MatName::Plutonium),
      Command::MatBrush(MatName::Copper),
    ];
    let activities: Vec<bool> = [true; 4].into();
    let mut alerts: Vec<bool> = [false; 4].into();
    match self.brush {
      Brush::Carbon => alerts[0] = true,
      Brush::Silicon => alerts[1] = true,
      Brush::Plutonium => alerts[2] = true,
      Brush::Copper => alerts[3] = true,
      Brush::Bot(_) => {}
    }
    let mut material_brush_buttons = ButtonPanel::<Command>::new(
      Rect::new(0.0, 0.0, 1000.0, 1000.0),
      (rects, labels, commands, activities, alerts),
    );
    panel.append(&mut material_brush_buttons);
    panel
  }

  fn build_token_panel(&self, rect: &Rect) -> ButtonPanel<Command> {
    let rects = split(&rect.clone(), vec![0.25, 0.5, 0.75], vec![0.0, 0.75]);
    let mut panel: ButtonPanel<Command> = build_incrementer::<Command>(
      &rects[0],
      "Tokens".to_string(),
      self.state.tokens,
      Command::Token(TknButton::Tokens, Sign::Plus),
      Command::Token(TknButton::Tokens, Sign::Minus),
    );
    panel.append(&mut build_incrementer(
      &rects[1],
      "Min Tks".to_string(),
      self.state.min_tokens,
      Command::Token(TknButton::MinTkns, Sign::Plus),
      Command::Token(TknButton::MinTkns, Sign::Minus),
    ));
    panel
  }

  fn build_bot_panels(&self, rect: &Rect) -> ButtonPanel<Command> {
    let mut panel = ButtonPanel::new(rect.clone(), (vec![], vec![], vec![], vec![], vec![]));
    let bot_rect = split(rect, vec![0.0, 0.25, 0.5, 0.75, 1.0], vec![0.0, 1.0]);
    for i in 0..NUM_TEMPLATES {
      panel.append(&mut self.build_single_bot_panel(&bot_rect[i], i));
    }
    panel
  }

  fn build_single_bot_panel(&self, rect: &Rect, index: usize) -> ButtonPanel<Command> {
    let rects: Vec<Rect> = split(rect, vec![0.0, 1.0], vec![0.0, 0.4, 0.6, 0.8, 1.0]);
    let mut panel = ButtonPanel::new(rect.clone(), (vec![], vec![], vec![], vec![], vec![]));
    match &self.state.entities[index] {
      // return with New button if entity is empty
      EntityStates::Empty => {
        panel.push(Button::<Command>::new(
          rect.clone(),
          ("New".to_string(), Command::BotEdit(index), true, false),
        ));
        return panel;
      }
      EntityStates::Entity(e, number) => {
        panel.append(&mut build_incrementer::<Command>(
          &rects[0],
          format!("Bot {}", index).to_string(),
          *number,
          Command::BotNumber(index, Sign::Plus),
          Command::BotNumber(index, Sign::Minus),
        ));
        // edit button
        panel.push(Button::<Command>::new(
          trim_margins(rects[1].clone(), 0.2, 0.2, 0.1, 0.1),
          ("Edit".to_string(), Command::BotEdit(index), true, false),
        ));
        // del button
        panel.push(Button::<Command>::new(
          trim_margins(rects[2].clone(), 0.2, 0.2, 0.1, 0.1),
          ("Delete".to_string(), Command::BotDelete(index), true, false),
        ));
        // if full, add brush
        if let Mix::Full(_) = e.brain {
          panel.push(Button::<Command>::new(
            trim_margins(rects[3].clone(), 0.2, 0.2, 0.1, 0.1),
            (
              "Use".to_string(),
              Command::BotBrush(index),
              true,
              (matches!(&self.brush, Brush::Bot(i) if index == *i)),
            ),
          ));
        }
      }
    }
    panel
  }

  fn revert_from(&mut self, nf: &NewBFState) {
    self.state = nf.clone();
  }

  fn validate_state(&mut self) {
    match self.new_type.clone() {
      NewBFType::BrandNew(nf_old) => {
        if !self.is_valid() {
          self.revert_from(&nf_old);
        } else {
          self.new_type = NewBFType::BrandNew(self.state.clone());
        }
      }
      NewBFType::Derived(nf_reference, nf_old) => {
        if !self.is_valid() {
          self.revert_from(&nf_old);
        } else {
          self.new_type = NewBFType::Derived(nf_reference, self.state.clone());
        }
      }
    }
  }

  fn is_valid(&self) -> bool {
    match self.new_type.clone() {
      NewBFType::BrandNew(_) => true,
      NewBFType::Derived(_, _) => true,
    }
  }

  fn update_main_panel(&mut self) {
    self.validate_state();
    let left_rect = trim_margins(
      split(&self.rect, vec![0.0, 0.45, 1.0], vec![0.0, 1.0])[0].clone(),
      0.05,
      0.05,
      0.05,
      0.05,
    );
    let rects: Vec<Rect> = split(&left_rect, vec![0.0, 1.0], vec![0.0, 0.25, 0.5, 0.9, 1.0]);
    let mut button_panel = self.build_material_panel(&rects[0]);
    button_panel.append(&mut self.build_token_panel(&rects[1]));
    button_panel.append(&mut self.build_bot_panels(&rects[2]));
    button_panel.push(Button::<Command>::new(
      trim_margins(rects[3].clone(), 0.3, 0.3, 0.3, 0.3),
      ("Save & Exit".to_string(), Command::Finish, true, false),
    ));
    for pos in board_iterator().into_iter() {
      button_panel.push(Button::<Command>::new(
        Rect::new(
          XDISPL + 16.0 * (pos.x as f32),
          YDISPL + 16.0 * (pos.y as f32),
          16.0,
          16.0,
        ),
        ("".to_string(), Command::MapLeftClk(pos), true, false),
      ))
    }
    self.panel = button_panel;
  }

  fn save_nf(&self) -> usize {
    let path = Path::new("./levels");
    let next_file_number = get_next_file_number(path, "lvl".to_string());
    let dest_filename = format!("{:05}", next_file_number);
    let mut dest = path.join(dest_filename);
    dest.set_extension("lvl");
    let mut file = File::create(dest).unwrap();
    let serialized = serde_json::to_string(&self.state).unwrap();
    file.write_all(serialized.as_bytes()).unwrap();
    next_file_number
  }

  fn build_finish_dialogue(&self) -> ButtonPanel<Command> {
    let rects: Vec<Rect> = split(
      &trim_margins(self.rect.clone(), 0.4, 0.4, 0.4, 0.4),
      vec![0.0, 1.0],
      vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0],
    );
    let mut panel = ButtonPanel::new(self.rect.clone(), (vec![], vec![], vec![], vec![], vec![]));
    panel.push(Button::<Command>::new(
      rects[0].clone(),
      ("Save?".to_string(), Command::BackToEdit, false, false),
    ));
    panel.push(Button::<Command>::new(
      rects[1].clone(),
      (
        "Continue editing".to_string(),
        Command::BackToEdit,
        true,
        false,
      ),
    ));
    panel.push(Button::<Command>::new(
      rects[2].clone(),
      ("Save".to_string(), Command::Save, true, false),
    ));
    panel.push(Button::<Command>::new(
      rects[3].clone(),
      (
        "Discard".to_string(),
        Command::ExitWithoutSaving,
        true,
        false,
      ),
    ));
    panel
  }
}

#[async_trait]
impl Ui for NewBF {
  type Command = ();
  type Builder = ();

  fn new(rect: Rect, _: ()) -> Self {
    let tileset = block_on(load_texture("assets/tileset.png")).unwrap();
    let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(25).try_into().unwrap();
    let mut floor = [0; WIDTH * HEIGHT];
    for i in 0..(WIDTH * HEIGHT) {
      floor[i] = rng.gen_range(0..7);
    }
    let new_bf_state = NewBFState {
      materials: Materials {
        carbon: 0,
        silicon: 0,
        plutonium: 0,
        copper: 0,
      },
      tokens: 0,
      min_tokens: 0,
      tiles: (0..(WIDTH * HEIGHT))
        .map(|_| Tile {
          entity_id: None,
          materials: Materials {
            carbon: 0,
            silicon: 0,
            plutonium: 0,
            copper: 0,
          },
        })
        .collect(),
      entities: construct_entities(),
    };
    let mut new_bf = NewBF {
      screen: Screen::Map,
      brush: Brush::Carbon,
      rect: rect.clone(),
      state: new_bf_state.clone(),
      floor,
      tileset,
      panel: ButtonPanel::new(rect, (vec![], vec![], vec![], vec![], vec![])),
      new_type: NewBFType::BrandNew(new_bf_state),
    };
    new_bf.update_main_panel();
    new_bf
  }

  async fn draw(&self) {
    self.panel.draw().await;
    draw_floor(XDISPL, YDISPL, &self.tileset, &self.floor).await;
    draw_mat_map(&self.state.tiles, XDISPL, YDISPL, &self.tileset).await;
    draw_rectangle(XDISPL, YDISPL, 16.0 * 60.0, 16.0 * 30.0, SMOKE);
    for pos in board_iterator() {
      if pos.y >= HEIGHT / 2 {
        if let Some(id) = &self.state.tiles[pos.to_index()].entity_id {
          if let EntityStates::Entity(e, _) = &self.state.entities[*id] {
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
    match &self.screen {
      Screen::Map => {}
      Screen::Entity(ee, _) => {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, SMOKE);
        ee.draw().await;
      }
      Screen::SaveDialogue(panel) => {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, SMOKE);
        panel.draw().await;
      }
      Screen::DisplayFileNumber(panel) => {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, SMOKE);
        panel.draw().await;
      }
    }
  }

  fn process_input(&mut self, input: Input) -> Option<()> {
    match &mut self.screen {
      Screen::Map => {
        let command = self.panel.process_input(input.clone());
        match command {
          None => {}
          Some(Command::MatPM(mat_name, sign)) => match mat_name {
            MatName::Carbon => match sign {
              Sign::Plus => {
                self.state.materials.carbon += 1;
              }
              Sign::Minus => {
                self.state.materials.carbon = self.state.materials.carbon.saturating_sub(1);
              }
            },
            MatName::Silicon => match sign {
              Sign::Plus => {
                self.state.materials.silicon += 1;
              }
              Sign::Minus => {
                self.state.materials.silicon = self.state.materials.silicon.saturating_sub(1);
              }
            },
            MatName::Plutonium => match sign {
              Sign::Plus => {
                self.state.materials.plutonium += 1;
              }
              Sign::Minus => {
                self.state.materials.plutonium = self.state.materials.plutonium.saturating_sub(1);
              }
            },
            MatName::Copper => match sign {
              Sign::Plus => {
                self.state.materials.copper += 1;
              }
              Sign::Minus => {
                self.state.materials.copper = self.state.materials.copper.saturating_sub(1);
              }
            },
          },
          Some(Command::MatBrush(mat)) => {
            self.brush = match mat {
              MatName::Carbon => Brush::Carbon,
              MatName::Silicon => Brush::Silicon,
              MatName::Plutonium => Brush::Plutonium,
              MatName::Copper => Brush::Copper,
            }
          }
          Some(Command::Token(tk_button, sign)) => match tk_button {
            TknButton::Tokens => match sign {
              Sign::Plus => {
                self.state.tokens += 1;
              }
              Sign::Minus => {
                self.state.tokens = self.state.tokens.saturating_sub(1);
              }
            },
            TknButton::MinTkns => match sign {
              Sign::Plus => {
                self.state.min_tokens += 1;
              }
              Sign::Minus => {
                self.state.min_tokens = self.state.min_tokens.saturating_sub(1);
              }
            },
          },
          Some(Command::MapLeftClk(pos)) => match self.brush {
            Brush::Carbon => {
              self.state.tiles[pos.to_index()].materials.carbon += 1;
              self.state.tiles[pos.invert().to_index()].materials.carbon += 1;
            }
            Brush::Silicon => {
              self.state.tiles[pos.to_index()].materials.silicon += 1;
              self.state.tiles[pos.invert().to_index()].materials.silicon += 1;
            }
            Brush::Plutonium => {
              self.state.tiles[pos.to_index()].materials.plutonium += 1;
              self.state.tiles[pos.invert().to_index()]
                .materials
                .plutonium += 1;
            }
            Brush::Copper => {
              self.state.tiles[pos.to_index()].materials.copper += 1;
              self.state.tiles[pos.invert().to_index()].materials.copper += 1;
            }
            Brush::Bot(i) => {
              self.state.tiles[pos.to_index()].entity_id = Some(i);
            }
          },
          Some(Command::BotNumber(i, sign)) => match &mut self.state.entities[i] {
            EntityStates::Empty => {}
            EntityStates::Entity(_, j) => match sign {
              Sign::Minus => *j = j.saturating_sub(1),
              Sign::Plus => *j += 1,
            },
          },
          Some(Command::BotBrush(i)) => self.brush = Brush::Bot(i),
          Some(Command::BotEdit(i)) => match &self.state.entities[i] {
            EntityStates::Empty => {
              self.state.entities[i] = EntityStates::Entity(
                MixEntity {
                  tokens: 0,
                  team: Team::Blue,
                  pos: Pos::new(0, 0),
                  hp: 1,
                  inventory_size: 0,
                  materials: Materials {
                    carbon: 0,
                    silicon: 0,
                    plutonium: 0,
                    copper: 0,
                  },
                  movement_type: MovementType::Still,
                  gun_damage: 0,
                  drill_damage: 0,
                  message: None,
                  brain: Mix::Bare,
                },
                0,
              )
            }
            _ => {
              self.screen = Screen::Entity(
                EntityEdit::new(
                  trim_margins(self.rect.clone(), 0.15, 0.15, 0.15, 0.15),
                  self.state.entities[i].clone(),
                ),
                i,
              );
            }
          },
          Some(Command::BotAddSubs(_)) => {}
          Some(Command::BotDelete(i)) => self.state.entities[i] = EntityStates::Empty,
          Some(Command::Finish) => {
            self.screen = Screen::SaveDialogue(self.build_finish_dialogue());
            return None;
          }
          _ => {}
        };
        self.update_main_panel();
        if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
          self.screen = Screen::SaveDialogue(self.build_finish_dialogue());
          None
        } else {
          None
        }
      }
      Screen::Entity(ee, index) => match &mut ee.process_input(input) {
        Some(EntityEditCommand::Exit(e)) => {
          if let EntityStates::Entity(_, n) = self.state.entities[*index] {
            self.state.entities[*index] = EntityStates::Entity(e.clone(), n);
          }
          self.screen = Screen::Map;
          self.update_main_panel();
          None
        }
        Some(EntityEditCommand::RequestChange(e)) => {
          if let EntityStates::Entity(_, n) = self.state.entities[*index] {
            self.state.entities[*index] = EntityStates::Entity(e.clone(), n);
          }
          let i = *index;
          self.update_main_panel();
          self.screen = Screen::Entity(
            EntityEdit::new(
              trim_margins(self.rect.clone(), 0.15, 0.15, 0.15, 0.15),
              self.state.entities[i].clone(),
            ),
            i,
          );
          None
        }
        _ => None,
      },
      Screen::SaveDialogue(panel) => {
        let command = panel.process_input(input.clone());
        match command {
          Some(Command::Save) => {
            // TODO probably we should just change the state and build the
            // panel in update_main_panel().
            let file_number = self.save_nf();
            let rects: Vec<Rect> = split(
              &trim_margins(self.rect.clone(), 0.4, 0.4, 0.4, 0.4),
              vec![0.0, 1.0],
              vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0],
            );
            let mut panel =
              ButtonPanel::new(rects[0].clone(), (vec![], vec![], vec![], vec![], vec![]));
            panel.push(Button::<Command>::new(
              rects[0].clone(),
              (
                format! {"Saved to: {:05}", file_number},
                Command::BackToEdit,
                false,
                false,
              ),
            ));
            panel.push(Button::<Command>::new(
              rects[1].clone(),
              ("Main Menu".to_string(), Command::Exit, true, false),
            ));
            self.screen = Screen::DisplayFileNumber(panel);
          }
          Some(Command::BackToEdit) => {
            self.screen = Screen::Map;
            self.update_main_panel();
          }
          Some(Command::ExitWithoutSaving) => return Some(()),
          _ => {}
        }
        None
      }
      Screen::DisplayFileNumber(panel) => {
        let command = panel.process_input(input.clone());
        match command {
          Some(Command::Exit) => return Some(()),
          _ => {}
        }

        None
      }
    }
  }
}
