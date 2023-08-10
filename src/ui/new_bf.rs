extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use futures::executor::block_on;
use macroquad::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use super::canvas::{draw_entity, draw_floor, draw_mat_map};
use super::entity_edit::{EntityEdit, EntityEditCommand};
use super::ui::{build_incrementer, split, trim_margins, Button, ButtonPanel, Input, Rect, Ui};
use crate::state::constants::{HEIGHT, NUM_TEMPLATES, WIDTH};
use crate::state::entity::{Mix, MixEntity, MovementType, Team};
use crate::state::geometry::{board_iterator, Pos};
use crate::state::materials::Materials;
use crate::state::state::Tile;

const XDISPL: f32 = 800.0;
const YDISPL: f32 = 30.0;

const SMOKE: macroquad::color::Color = Color::new(0.0, 0.0, 0.0, 0.3);
//const DARKSMOKE: macroquad::color::Color = Color::new(0.0, 0.0, 0.0, 0.5);

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
  SaveAndExit,
  MatPM(MatName, Sign),
  MatBrush(MatName),
  Token(TknButton, Sign),
  MapLeftClk(Pos),
  BotNumber(usize, Sign),
  BotBrush(usize),
  BotEdit(usize),
  BotAddSubs(usize),
  BotDelete(usize),
}

#[derive(Debug, Clone)]
pub enum EntityStates {
  Empty,
  Entity(MixEntity, usize),
}

#[derive(Clone, Debug)]
enum Screen {
  Map,
  Entity(EntityEdit, usize),
}

#[derive(Clone, Debug)]
pub enum NewBFType {
  BrandNew(Option<Box<NewBF>>),
  Derived(Option<Box<NewBF>>, Option<Box<NewBF>>),
}

#[derive(Clone, Debug)]
pub struct NewBF {
  screen: Screen,
  rect: Rect,
  materials: Materials,
  tokens: usize,
  min_tokens: usize,
  brush: Brush,
  tiles: Vec<Tile>,
  entities: [EntityStates; NUM_TEMPLATES],
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
      self.materials.carbon,
      Command::MatPM(MatName::Carbon, Sign::Plus),
      Command::MatPM(MatName::Carbon, Sign::Minus),
    );
    panel.append(&mut build_incrementer::<Command>(
      &rects[1],
      "Silicon".to_string(),
      self.materials.silicon,
      Command::MatPM(MatName::Silicon, Sign::Plus),
      Command::MatPM(MatName::Silicon, Sign::Minus),
    ));
    panel.append(&mut build_incrementer::<Command>(
      &rects[2],
      "Pluton.".to_string(),
      self.materials.plutonium,
      Command::MatPM(MatName::Plutonium, Sign::Plus),
      Command::MatPM(MatName::Plutonium, Sign::Minus),
    ));
    panel.append(&mut build_incrementer::<Command>(
      &rects[3],
      "Copper".to_string(),
      self.materials.copper,
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
      self.tokens,
      Command::Token(TknButton::Tokens, Sign::Plus),
      Command::Token(TknButton::Tokens, Sign::Minus),
    );
    panel.append(&mut build_incrementer(
      &rects[1],
      "Min Tks".to_string(),
      self.min_tokens,
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
    match &self.entities[index] {
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

  fn revert_from(&mut self, nf: &NewBF) {
    self.screen = nf.screen.clone();
    self.rect = nf.rect.clone();
    self.materials = nf.materials.clone();
    self.tokens = nf.tokens.clone();
    self.min_tokens = nf.min_tokens.clone();
    self.brush = nf.brush.clone();
    self.tiles = nf.tiles.clone();
    self.entities = nf.entities.clone();
    self.floor = nf.floor.clone();
    self.tileset = nf.tileset.clone();
  }

  fn validate_state(&mut self) {
    match self.new_type.clone() {
      NewBFType::BrandNew(nf_old) => {
        if !self.is_valid() {
          self.revert_from(nf_old.as_ref().unwrap());
        }
      }
      NewBFType::Derived(nf_reference, nf_old) => {}
    }
  }

  fn is_valid(&self) -> bool {
    true
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
      ("Save & Exit".to_string(), Command::SaveAndExit, true, false),
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
    let mut new_bf = NewBF {
      screen: Screen::Map,
      rect: rect.clone(),
      materials: Materials {
        carbon: 0,
        silicon: 0,
        plutonium: 0,
        copper: 0,
      },
      tokens: 0,
      min_tokens: 0,
      brush: Brush::Carbon,
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
      floor,
      tileset,
      panel: ButtonPanel::new(rect, (vec![], vec![], vec![], vec![], vec![])),
      new_type: NewBFType::BrandNew(None),
    };
    new_bf.new_type = NewBFType::BrandNew(Some(Box::new(new_bf.clone())));
    new_bf.update_main_panel();
    new_bf
  }

  async fn draw(&self) {
    self.panel.draw().await;
    draw_floor(XDISPL, YDISPL, &self.tileset, &self.floor).await;
    draw_mat_map(&self.tiles, XDISPL, YDISPL, &self.tileset).await;
    draw_rectangle(XDISPL, YDISPL, 16.0 * 60.0, 16.0 * 30.0, SMOKE);
    for pos in board_iterator() {
      if pos.y >= HEIGHT / 2 {
        if let Some(id) = &self.tiles[pos.to_index()].entity_id {
          if let EntityStates::Entity(e, _) = &self.entities[*id] {
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
                self.materials.carbon += 1;
              }
              Sign::Minus => {
                self.materials.carbon = self.materials.carbon.saturating_sub(1);
              }
            },
            MatName::Silicon => match sign {
              Sign::Plus => {
                self.materials.silicon += 1;
              }
              Sign::Minus => {
                self.materials.silicon = self.materials.silicon.saturating_sub(1);
              }
            },
            MatName::Plutonium => match sign {
              Sign::Plus => {
                self.materials.plutonium += 1;
              }
              Sign::Minus => {
                self.materials.plutonium = self.materials.plutonium.saturating_sub(1);
              }
            },
            MatName::Copper => match sign {
              Sign::Plus => {
                self.materials.copper += 1;
              }
              Sign::Minus => {
                self.materials.copper = self.materials.copper.saturating_sub(1);
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
                self.tokens += 1;
              }
              Sign::Minus => {
                self.tokens = self.tokens.saturating_sub(1);
              }
            },
            TknButton::MinTkns => match sign {
              Sign::Plus => {
                self.min_tokens += 1;
              }
              Sign::Minus => {
                self.min_tokens = self.min_tokens.saturating_sub(1);
              }
            },
          },
          Some(Command::MapLeftClk(pos)) => match self.brush {
            Brush::Carbon => {
              self.tiles[pos.to_index()].materials.carbon += 1;
              self.tiles[pos.invert().to_index()].materials.carbon += 1;
            }
            Brush::Silicon => {
              self.tiles[pos.to_index()].materials.silicon += 1;
              self.tiles[pos.invert().to_index()].materials.silicon += 1;
            }
            Brush::Plutonium => {
              self.tiles[pos.to_index()].materials.plutonium += 1;
              self.tiles[pos.invert().to_index()].materials.plutonium += 1;
            }
            Brush::Copper => {
              self.tiles[pos.to_index()].materials.copper += 1;
              self.tiles[pos.invert().to_index()].materials.copper += 1;
            }
            Brush::Bot(i) => {
              self.tiles[pos.to_index()].entity_id = Some(i);
            }
          },
          Some(Command::BotNumber(i, sign)) => match &mut self.entities[i] {
            EntityStates::Empty => {}
            EntityStates::Entity(_, j) => match sign {
              Sign::Minus => *j = j.saturating_sub(1),
              Sign::Plus => *j += 1,
            },
          },
          Some(Command::BotBrush(i)) => self.brush = Brush::Bot(i),
          Some(Command::BotEdit(i)) => match &self.entities[i] {
            EntityStates::Empty => {
              self.entities[i] = EntityStates::Entity(
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
                  self.entities[i].clone(),
                ),
                i,
              );
            }
          },
          Some(Command::BotAddSubs(_)) => {}
          Some(Command::BotDelete(i)) => self.entities[i] = EntityStates::Empty,
          Some(Command::SaveAndExit) => return Some(()),
        };
        self.update_main_panel();
        if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
          Some(())
        } else {
          None
        }
      }
      Screen::Entity(ee, index) => match &mut ee.process_input(input) {
        Some(EntityEditCommand::Exit(e)) => {
          if let EntityStates::Entity(_, n) = self.entities[*index] {
            self.entities[*index] = EntityStates::Entity(e.clone(), n);
          }
          self.screen = Screen::Map;
          self.update_main_panel();
          None
        }
        _ => None,
      },
    }
  }
}
