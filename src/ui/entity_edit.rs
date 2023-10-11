extern crate rand;
extern crate rand_chacha;
use std::cmp::{max, min};

use async_trait::async_trait;
use macroquad::prelude::*;

use super::ui::{
  build_incrementer, plus_minus, split, trim_margins, Button, ButtonPanel, Input, Rect, Sign, Ui,
};
use crate::state::bf::EntityState;
use crate::state::brain::get_code_vec;
use crate::state::constants::NUM_TEMPLATES;
use crate::state::entity::{Full, Mix, MixTemplate, MovementType};

#[derive(Clone, Debug)]
pub enum EntityEditCommand {
  RequestChange(MixTemplate),
  Exit(MixTemplate),
}

#[derive(Clone, Debug)]
pub enum Attribute {
  Token,
  HP,
  InvSize,
  Carbon,
  Silicon,
  Plutonium,
  Copper,
  Sub1,
  Sub2,
  CodeID,
  Gas,
  GunDamage,
  DrillDamage,
  Speed,
}

#[derive(Clone, Debug)]
pub enum Command {
  Exit,
  PM(Attribute, Sign),
  AddAttribute,
  AddConstructs,
  AddCode,
}

#[derive(Clone, Debug)]
pub struct EntityEdit {
  pub entity: EntityState,
  old_entity: EntityState,
  rect: Rect,
  message: String,
  panel: ButtonPanel<Command>,
}

impl EntityEdit {
  fn validate_state(&mut self) {
    if !self.is_valid() {
      self.entity = self.old_entity.clone();
    }
  }
  fn is_valid(&self) -> bool {
    match &self.entity {
      EntityState::Empty => {}
      EntityState::Entity(e, _) => {
        let load =
          e.materials.carbon + e.materials.silicon + e.materials.plutonium + e.materials.copper;
        if load > e.inventory_size {
          return false;
        }
      }
    }
    true
  }

  fn update_main_panel(&mut self) {
    self.validate_state();
    self.old_entity = self.entity.clone();
    let rects: Vec<Rect> = split(
      &self.rect,
      vec![0.0, 1.0],
      vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0],
    );
    let mut panel = ButtonPanel::new(self.rect.clone(), (vec![], vec![], vec![], vec![], vec![]));
    panel.push(Button::<Command>::new(
      trim_margins(rects[4].clone(), 0.1, 0.1, 0.1, 0.1),
      ("Exit".to_string(), Command::Exit, true, false),
    ));
    match &self.entity {
      EntityState::Empty => unreachable!(),
      EntityState::Entity(e, _) => {
        let first_row_rects: Vec<Rect> =
          split(&rects[0], vec![0.0, 0.25, 0.5, 0.75, 1.0], vec![0.0, 1.0])
            .into_iter()
            .map(|r| trim_margins(r, 0.1, 0.1, 0.1, 0.1))
            .collect();
        panel.append(&mut build_incrementer::<Command>(
          &first_row_rects[0],
          "Tokens".to_string(),
          e.tokens,
          Command::PM(Attribute::Token, Sign::Plus),
          Command::PM(Attribute::Token, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
          &first_row_rects[1],
          "HP".to_string(),
          e.hp,
          Command::PM(Attribute::HP, Sign::Plus),
          Command::PM(Attribute::HP, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
          &first_row_rects[2],
          "Inv. Size".to_string(),
          e.inventory_size,
          Command::PM(Attribute::InvSize, Sign::Plus),
          Command::PM(Attribute::InvSize, Sign::Minus),
        ));
        let second_row_rects: Vec<Rect> =
          split(&rects[1], vec![0.0, 0.25, 0.5, 0.75, 1.0], vec![0.0, 1.0])
            .into_iter()
            .map(|r| trim_margins(r, 0.1, 0.1, 0.1, 0.1))
            .collect();
        panel.append(&mut build_incrementer::<Command>(
          &second_row_rects[0],
          "Carbon".to_string(),
          e.materials.carbon,
          Command::PM(Attribute::Carbon, Sign::Plus),
          Command::PM(Attribute::Carbon, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
          &second_row_rects[1],
          "Silicon".to_string(),
          e.materials.silicon,
          Command::PM(Attribute::Silicon, Sign::Plus),
          Command::PM(Attribute::Silicon, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
          &second_row_rects[2],
          "Plutonium".to_string(),
          e.materials.plutonium,
          Command::PM(Attribute::Plutonium, Sign::Plus),
          Command::PM(Attribute::Plutonium, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
          &second_row_rects[3],
          "Copper".to_string(),
          e.materials.copper,
          Command::PM(Attribute::Copper, Sign::Plus),
          Command::PM(Attribute::Copper, Sign::Minus),
        ));

        let third_row_rects: Vec<Rect> =
          split(&rects[2], vec![0.0, 0.25, 0.5, 0.75, 1.0], vec![0.0, 1.0])
            .into_iter()
            .map(|r| trim_margins(r, 0.1, 0.1, 0.1, 0.1))
            .collect();
        panel.append(&mut build_incrementer::<Command>(
          &third_row_rects[0],
          "Speed".to_string(),
          match e.movement_type {
            MovementType::Still => 0,
            MovementType::Walk => 1,
          },
          Command::PM(Attribute::Speed, Sign::Plus),
          Command::PM(Attribute::Speed, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
          &third_row_rects[1],
          "Gun damage".to_string(),
          e.gun_damage,
          Command::PM(Attribute::GunDamage, Sign::Plus),
          Command::PM(Attribute::GunDamage, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
          &third_row_rects[2],
          "Drill damage".to_string(),
          e.drill_damage,
          Command::PM(Attribute::DrillDamage, Sign::Plus),
          Command::PM(Attribute::DrillDamage, Sign::Minus),
        ));
        let fourth_row_rects: Vec<Rect> =
          split(&rects[3], vec![0.0, 0.25, 0.5, 0.75, 1.0], vec![0.0, 1.0])
            .into_iter()
            .map(|r| trim_margins(r, 0.1, 0.1, 0.1, 0.1))
            .collect();
        match e.brain {
          Mix::Bare => panel.push(Button::<Command>::new(
            trim_margins(rects[3].clone(), 0.1, 0.1, 0.1, 0.1),
            (
              "Add Constr.".to_string(),
              Command::AddConstructs,
              true,
              false,
            ),
          )),
          Mix::Half(h) => {
            panel.append(&mut build_incrementer::<Command>(
              &fourth_row_rects[0],
              "Sub 1".to_string(),
              h[0] as usize,
              Command::PM(Attribute::Sub1, Sign::Plus),
              Command::PM(Attribute::Sub1, Sign::Minus),
            ));
            panel.append(&mut build_incrementer::<Command>(
              &fourth_row_rects[1],
              "Sub 2".to_string(),
              h[1] as usize,
              Command::PM(Attribute::Sub2, Sign::Plus),
              Command::PM(Attribute::Sub2, Sign::Minus),
            ));
            panel.push(Button::<Command>::new(
              trim_margins(fourth_row_rects[2].clone(), 0.1, 0.1, 0.1, 0.1),
              ("Add Code".to_string(), Command::AddCode, true, false),
            ));
          }
          Mix::Full(Full {
            half: h,
            code_index: ci,
            gas,
          }) => {
            panel.append(&mut build_incrementer::<Command>(
              &fourth_row_rects[0],
              "Sub 1".to_string(),
              h[0] as usize,
              Command::PM(Attribute::Sub1, Sign::Plus),
              Command::PM(Attribute::Sub1, Sign::Minus),
            ));
            panel.append(&mut build_incrementer::<Command>(
              &fourth_row_rects[1],
              "Sub 2".to_string(),
              h[1] as usize,
              Command::PM(Attribute::Sub2, Sign::Plus),
              Command::PM(Attribute::Sub2, Sign::Minus),
            ));
            panel.append(&mut build_incrementer::<Command>(
              &fourth_row_rects[2],
              "Code".to_string(),
              ci,
              Command::PM(Attribute::CodeID, Sign::Plus),
              Command::PM(Attribute::CodeID, Sign::Minus),
            ));
            panel.append(&mut build_incrementer::<Command>(
              &fourth_row_rects[3],
              "Gas".to_string(),
              gas,
              Command::PM(Attribute::Gas, Sign::Plus),
              Command::PM(Attribute::Gas, Sign::Minus),
            ));
          }
        }
      }
    }
    self.panel = panel;
  }
}

#[async_trait]
impl Ui for EntityEdit {
  type Command = EntityEditCommand;
  type Builder = EntityState;

  fn new(rect: Rect, e: EntityState) -> Self {
    let panel = ButtonPanel::new(rect.clone(), (vec![], vec![], vec![], vec![], vec![]));
    let mut ee = EntityEdit {
      rect,
      entity: e.clone(),
      message: "Editing bot".to_string(),
      panel,
      old_entity: e,
    };
    ee.update_main_panel();
    ee
  }
  async fn draw(&self) {
    draw_rectangle(
      self.rect.x,
      self.rect.y - 40.0,
      self.rect.w,
      self.rect.h + 40.0,
      BLACK,
    );
    draw_text(
      &self.message,
      self.rect.x + 20.0,
      self.rect.y,
      40.0,
      DARKBLUE,
    );
    self.panel.draw().await;
  }

  fn process_input(&mut self, input: Input) -> Option<EntityEditCommand> {
    let command = self.panel.process_input(input.clone());
    if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
      if let EntityState::Entity(e, _) = &self.entity {
        return Some(EntityEditCommand::Exit(e.clone()));
      }
    }
    match command {
      Some(Command::Exit) => {
        if let EntityState::Entity(e, _) = &self.entity {
          return Some(EntityEditCommand::Exit(e.clone()));
        }
      }
      Some(Command::PM(attribute, sign)) => {
        if let EntityState::Entity(mix, _) = &mut self.entity {
          match attribute {
            Attribute::Token => {
              mix.tokens = plus_minus(&input, mix.tokens, sign);
            }
            Attribute::HP => {
              mix.hp = max(plus_minus(&input, mix.hp, sign), 1);
            }
            Attribute::InvSize => {
              mix.inventory_size = plus_minus(&input, mix.inventory_size, sign);
            }
            Attribute::Carbon => {
              mix.materials.carbon = plus_minus(&input, mix.materials.carbon, sign);
            }
            Attribute::Silicon => {
              mix.materials.silicon = plus_minus(&input, mix.materials.silicon, sign);
            }
            Attribute::Plutonium => {
              mix.materials.plutonium = plus_minus(&input, mix.materials.plutonium, sign);
            }
            Attribute::Copper => {
              mix.materials.copper = plus_minus(&input, mix.materials.copper, sign);
            }
            Attribute::Sub1 => {
              if let Mix::Half(h) | Mix::Full(Full { half: h, .. }) = &mut mix.brain {
                h[0] = min(plus_minus(&input, h[0] as usize, sign), NUM_TEMPLATES - 1) as u8;
              }
            }
            Attribute::Sub2 => {
              if let Mix::Half(h) | Mix::Full(Full { half: h, .. }) = &mut mix.brain {
                h[1] = min(plus_minus(&input, h[1] as usize, sign), NUM_TEMPLATES - 1) as u8;
              }
            }
            Attribute::CodeID => {
              if let Mix::Full(Full { code_index: c, .. }) = &mut mix.brain {
                let new_code = plus_minus(&input, *c, sign);
                if let Some((_, path)) = get_code_vec().get(new_code) {
                  self.message = path
                    .clone()
                    .file_name()
                    .unwrap()
                    .to_os_string()
                    .to_str()
                    .unwrap_or("Unknown")
                    .to_string();
                  *c = new_code;
                }
              }
            }
            Attribute::Gas => {
              if let Mix::Full(Full { gas: g, .. }) = &mut mix.brain {
                *g = plus_minus(&input, *g, sign);
              }
            }
            Attribute::Speed => match sign {
              Sign::Minus => mix.movement_type = MovementType::Still,
              Sign::Plus => mix.movement_type = MovementType::Walk,
            },
            Attribute::GunDamage => {
              mix.gun_damage = plus_minus(&input, mix.gun_damage, sign);
            }
            Attribute::DrillDamage => {
              mix.drill_damage = plus_minus(&input, mix.drill_damage, sign);
            }
          };
        }
      }
      None => {}
      Some(Command::AddAttribute) => {
        if let EntityState::Entity(mix, _) = &mut self.entity {
          mix.movement_type = MovementType::Still;
          mix.gun_damage = 0;
          mix.drill_damage = 0;
          mix.brain = Mix::Bare;
        }
      }
      Some(Command::AddConstructs) => {
        if let EntityState::Entity(mix, _) = &mut self.entity {
          mix.brain = Mix::Half([0, 0]);
        }
      }
      Some(Command::AddCode) => {
        if let EntityState::Entity(mix, _) = &mut self.entity {
          if let Mix::Half(h) = mix.brain {
            mix.brain = Mix::Full(Full {
              half: h.clone(),
              code_index: 0,
              gas: 0,
            });
          }
        }
      }
    }
    self.update_main_panel();
    if let EntityState::Entity(e, _) = &self.entity {
      return Some(EntityEditCommand::RequestChange(e.clone()));
    } else {
      return None;
    }
  }
}
