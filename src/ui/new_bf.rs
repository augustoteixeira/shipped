extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use futures::executor::block_on;
use macroquad::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use super::canvas::{draw_floor, draw_mat_map, draw_template_at};
use super::entity_edit::{EntityEdit, EntityEditCommand};
use super::ui::{
  build_incrementer, one_or_ten, split, trim_margins, Button, ButtonPanel, Input, Rect, Ui,
};
use crate::state::bf::{join_tiles, BFState, EntityState, MatName, ValidationError};
use crate::state::constants::{HEIGHT, NUM_TEMPLATES, WIDTH};
use crate::state::entity::Team;
use crate::state::geometry::{half_board_iterator, Pos};
use crate::state::materials::Materials;
use crate::state::state::Tile;
use crate::state::utils::get_next_file_number;

const XDISPL: f32 = 800.0;
const YDISPL: f32 = 30.0;

const SMOKE: macroquad::color::Color = Color::new(0.0, 0.0, 0.0, 0.3);

#[derive(Clone, Debug)]
pub enum Brush {
  Eraser,
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
  MapClk(Pos),
  BotNumber(usize, Sign),
  BotBrush(usize),
  EraserBrush,
  BotEdit(usize),
  BotAddSubs(usize),
  BotDelete(usize),
  Save,
  BackToEdit,
  ExitWithoutSaving,
  Exit,
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
  BrandNew,
  Derived(BFState, usize),
}

#[derive(Debug)]
pub struct NewBF {
  state: BFState,
  old_state: BFState,
  message: String,
  brush: Brush,
  screen: Screen,
  rect: Rect,
  floor: [usize; WIDTH * HEIGHT],
  tileset: Texture2D,
  joined_tiles: Vec<Tile>,
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
      self.state.get_materials().carbon,
      Command::MatPM(MatName::Carbon, Sign::Plus),
      Command::MatPM(MatName::Carbon, Sign::Minus),
    );
    panel.append(&mut build_incrementer::<Command>(
      &rects[1],
      "Silicon".to_string(),
      self.state.get_materials().silicon,
      Command::MatPM(MatName::Silicon, Sign::Plus),
      Command::MatPM(MatName::Silicon, Sign::Minus),
    ));
    panel.append(&mut build_incrementer::<Command>(
      &rects[2],
      "Pluton.".to_string(),
      self.state.get_materials().plutonium,
      Command::MatPM(MatName::Plutonium, Sign::Plus),
      Command::MatPM(MatName::Plutonium, Sign::Minus),
    ));
    panel.append(&mut build_incrementer::<Command>(
      &rects[3],
      "Copper".to_string(),
      self.state.get_materials().copper,
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
      Brush::Eraser => {}
    }
    let mut material_brush_buttons = ButtonPanel::<Command>::new(
      Rect::new(0.0, 0.0, 1000.0, 1000.0),
      (rects, labels, commands, activities, alerts),
    );
    panel.append(&mut material_brush_buttons);
    panel
  }

  fn build_token_panel(&self, rect: &Rect) -> ButtonPanel<Command> {
    let rects = split(&rect.clone(), vec![0.25, 0.5, 0.75, 1.0], vec![0.0, 0.75]);
    let mut panel: ButtonPanel<Command> = build_incrementer::<Command>(
      &rects[0],
      "Tokens".to_string(),
      self.state.get_tokens(),
      Command::Token(TknButton::Tokens, Sign::Plus),
      Command::Token(TknButton::Tokens, Sign::Minus),
    );
    panel.append(&mut build_incrementer(
      &rects[1],
      "Min Tks".to_string(),
      self.state.get_min_tokens(),
      Command::Token(TknButton::MinTkns, Sign::Plus),
      Command::Token(TknButton::MinTkns, Sign::Minus),
    ));
    panel.push(Button::<Command>::new(
      rects[2].clone(),
      (
        "Erase".to_string(),
        Command::EraserBrush,
        true,
        matches!(self.brush, Brush::Eraser),
      ),
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
    match &self.state.get_entities()[index] {
      // return with New button if entity is empty
      EntityState::Empty => {
        panel.push(Button::<Command>::new(
          rect.clone(),
          ("New".to_string(), Command::BotEdit(index), true, false),
        ));
        return panel;
      }
      EntityState::Entity(_, number) => {
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
        // add brush
        //if let Mix::Full(_) = e.brain {
        panel.push(Button::<Command>::new(
          trim_margins(rects[3].clone(), 0.2, 0.2, 0.1, 0.1),
          (
            "Use".to_string(),
            Command::BotBrush(index),
            true, // TODO make it inactive if there are no bots to use
            (matches!(&self.brush, Brush::Bot(i) if index == *i)),
          ),
        ));
        //}
      }
    }
    panel
  }

  fn revert_from(&mut self, nf: &BFState) {
    self.state = nf.clone();
  }

  fn validate_state(&mut self) {
    match self.is_valid() {
      Ok(true) => self.old_state = self.state.clone(),
      Ok(false) => self.revert_from(&self.old_state.clone()),
      Err(e) => {
        self.message = format!("{}", e);
        self.revert_from(&self.old_state.clone());
      }
    };
  }

  fn is_valid(&self) -> Result<bool, ValidationError> {
    self.state.check_validity()?;
    match self.new_type.clone() {
      NewBFType::BrandNew => Ok(true),
      NewBFType::Derived(reference, _) => self.state.is_compatible(&reference),
    }
  }

  fn update_main_panel(&mut self) {
    self.validate_state();
    self.joined_tiles = join_tiles(&self.state, &self.state);
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
    for pos in half_board_iterator().into_iter() {
      button_panel.push(Button::<Command>::new(
        Rect::new(
          XDISPL + 16.0 * (pos.x as f32),
          YDISPL + 16.0 * (HEIGHT.saturating_sub(pos.y + 1) as f32),
          16.0,
          16.0,
        ),
        ("".to_string(), Command::MapClk(pos), true, false),
      ))
    }
    self.panel = button_panel;
  }

  fn save_nf(&self) -> usize {
    match self.new_type {
      NewBFType::BrandNew => {
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
      NewBFType::Derived(_, level) => {
        let path = &Path::new("./squads/").join(format!("{:05}", level));
        if !path.exists() {
          fs::create_dir_all(path.clone()).unwrap();
        }
        let next_file_number = get_next_file_number(&path, "sqd".to_string());
        let dest_filename = format!("{:05}", next_file_number);
        let mut dest = path.join(dest_filename);
        dest.set_extension("sqd");
        let mut file = File::create(dest).unwrap();
        let serialized = serde_json::to_string(&self.state).unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
        next_file_number
      }
    }
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
  type Builder = Option<(BFState, usize)>;

  fn new(rect: Rect, builder: Option<(BFState, usize)>) -> Self {
    let tileset = block_on(load_texture("assets/tileset.png")).unwrap();
    let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(25).try_into().unwrap();
    let mut floor = [0; WIDTH * HEIGHT];
    for i in 0..(WIDTH * HEIGHT) {
      floor[i] = rng.gen_range(0..7);
    }
    let new_bf_state = match &builder {
      None => BFState::new(),
      Some((state, _)) => state.clone(),
    };
    let mut new_bf = NewBF {
      screen: Screen::Map,
      message: "Editing field".to_string(),
      brush: Brush::Carbon,
      rect: rect.clone(),
      state: new_bf_state.clone(),
      old_state: new_bf_state.clone(),
      floor,
      tileset,
      joined_tiles: vec![],
      panel: ButtonPanel::new(rect, (vec![], vec![], vec![], vec![], vec![])),
      new_type: match builder {
        None => NewBFType::BrandNew,
        Some((_, level)) => NewBFType::Derived(new_bf_state, level),
      },
    };
    new_bf.update_main_panel();
    new_bf
  }

  async fn draw(&self) {
    self.panel.draw().await;
    draw_text(&self.message, 20.0, 40.0, 40.0, DARKBLUE);
    draw_floor(XDISPL, YDISPL, &self.tileset, &self.floor).await;
    draw_mat_map(&self.joined_tiles, XDISPL, YDISPL, &self.tileset).await;
    draw_rectangle(XDISPL, YDISPL, 16.0 * 60.0, 16.0 * 30.0, SMOKE);
    for pos in half_board_iterator() {
      if let Some(id) = &self.state.get_tiles()[pos.to_index()].entity_id {
        if let EntityState::Entity(e, _) = &self.state.get_entities()[*id] {
          draw_template_at(&e.clone(), XDISPL, YDISPL, pos, Team::Blue, &self.tileset).await;
          let f = e.clone();
          draw_template_at(&f, XDISPL, YDISPL, pos.invert(), Team::Red, &self.tileset).await;
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
    if let Input::Click(_, _) = input.clone() {
      self.message = "Editing field".to_string();
    }
    match &mut self.screen {
      Screen::Map => {
        let command = self.panel.process_input(input.clone());
        match command {
          None => {}
          Some(Command::MatPM(mat_name, sign)) => match sign {
            Sign::Plus => {
              self.state.add_material(
                mat_name,
                match input {
                  Input::Click(MouseButton::Left, _) => 1,
                  Input::Click(MouseButton::Right, _) => 10,
                  _ => 0,
                },
              );
            }
            Sign::Minus => {
              if let Err(e) = self.state.try_sub_material(mat_name, 1) {
                self.message = format!("{}", e);
              };
            }
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
                self.state.add_tokens(1);
              }
              Sign::Minus => {
                if let Err(e) = self.state.try_sub_tokens(1) {
                  self.message = format!("{}", e);
                };
              }
            },
            TknButton::MinTkns => match sign {
              Sign::Plus => {
                if let Err(e) = self.state.try_add_min_tokens(1) {
                  self.message = format!("{}", e);
                }
              }
              Sign::Minus => {
                if let Err(e) = self.state.try_sub_min_tokens(1) {
                  self.message = format!("{}", e);
                };
              }
            },
          },
          Some(Command::MapClk(pos)) => match self.brush {
            Brush::Carbon => {
              if let Err(e) =
                self
                  .state
                  .insert_material_tile(MatName::Carbon, pos, one_or_ten(&input))
              {
                self.message = format!("{}", e);
              };
            }
            Brush::Silicon => {
              if let Err(e) =
                self
                  .state
                  .insert_material_tile(MatName::Silicon, pos, one_or_ten(&input))
              {
                self.message = format!("{}", e);
              };
            }
            Brush::Plutonium => {
              if let Err(e) =
                self
                  .state
                  .insert_material_tile(MatName::Plutonium, pos, one_or_ten(&input))
              {
                self.message = format!("{}", e);
              };
            }
            Brush::Copper => {
              if let Err(e) =
                self
                  .state
                  .insert_material_tile(MatName::Copper, pos, one_or_ten(&input))
              {
                self.message = format!("{}", e);
              };
            }
            Brush::Bot(i) => {
              if let Err(e) = self.state.add_bot_board(i, pos) {
                self.message = format!("{}", e);
              }
            }
            Brush::Eraser => match &self.new_type {
              NewBFType::BrandNew => {
                if let Err(e) = self.state.erase_bot_from_board(pos) {
                  self.message = format!("{}", e);
                }
                let remainder = Materials {
                  carbon: 0,
                  silicon: 0,
                  plutonium: 0,
                  copper: 0,
                };
                if let Err(e) = self.state.erase_material_tile(pos, remainder) {
                  self.message = format!("{}", e);
                };
              }
              NewBFType::Derived(reference, _) => {
                let ref_tile = &reference.get_tiles()[pos.to_index()];
                if ref_tile.entity_id.is_some() {
                  self.message = format!("Cannot remove level bot {:?}", pos);
                } else {
                  if let Err(e) = self.state.erase_bot_from_board(pos) {
                    self.message = format!("{}", e);
                  }
                }
                let remainder = ref_tile.materials.clone();
                if let Err(e) = self.state.erase_material_tile(pos, remainder) {
                  self.message = format!("{}", e);
                };
              }
            },
          },
          Some(Command::BotNumber(i, sign)) => match &self.state.get_entities()[i] {
            EntityState::Empty => {}
            EntityState::Entity(_, _) => match sign {
              Sign::Minus => {
                if let Err(e) = self.state.sell_bot(i) {
                  self.message = format!("{}", e);
                }
              }
              Sign::Plus => {
                if let Err(e) = self.state.buy_bot(i) {
                  self.message = format!("{}", e);
                }
              }
            },
          },
          Some(Command::BotBrush(i)) => self.brush = Brush::Bot(i),
          Some(Command::BotEdit(i)) => match &self.state.get_entities()[i] {
            EntityState::Empty => {
              self.state.initialize_bot(i).unwrap();
            }
            _ => {
              self.screen = Screen::Entity(
                EntityEdit::new(
                  trim_margins(self.rect.clone(), 0.15, 0.15, 0.15, 0.15),
                  self.state.get_entities()[i].clone(),
                ),
                i,
              );
            }
          },
          Some(Command::BotAddSubs(_)) => {}
          Some(Command::BotDelete(i)) => self.state.delete_bot(i),
          Some(Command::Finish) => {
            self.screen = Screen::SaveDialogue(self.build_finish_dialogue());
            return None;
          }
          Some(Command::EraserBrush) => {
            self.brush = Brush::Eraser;
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
          if let Err(e) = self.state.update_bot(*index, e.clone()) {
            self.message = format!("{}", e);
          }
          self.screen = Screen::Map;
          self.update_main_panel();
          None
        }
        Some(EntityEditCommand::RequestChange(e)) => {
          let i = *index;
          if let EntityState::Entity(_, _) = self.state.get_entities()[i] {
            if let Err(e) = self.state.update_bot(*index, e.clone()) {
              self.message = format!("{}", e);
            }
            let new_entity_cost = self.state.entity_cost(i);
            let old_entity_cost = self.old_state.entity_cost(i);
            // check material cost
            if new_entity_cost.0 <= self.state.get_materials().clone() + old_entity_cost.0.clone() {
              self.state.add_materials(old_entity_cost.0.clone());
              if let Err(e) = self.state.try_sub_materials(new_entity_cost.0) {
                self.message = format!("{}", e);
              };
            } else {
              self.revert_from(&self.old_state.clone());
            }
            // check token cost
            if new_entity_cost.1 <= self.state.get_tokens() + old_entity_cost.1 {
              self.state.add_tokens(old_entity_cost.1);
              if let Err(e) = self.state.try_sub_tokens(new_entity_cost.1) {
                self.message = format!("{}", e);
              };
            } else {
              self.revert_from(&self.old_state.clone());
            }
          }
          self.update_main_panel();
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
