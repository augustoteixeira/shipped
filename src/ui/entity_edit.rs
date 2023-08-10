extern crate rand;
extern crate rand_chacha;
use std::cmp::{max, min};

use async_trait::async_trait;
use macroquad::prelude::*;

use super::new_bf::EntityStates;
use super::ui::{
  build_incrementer, plus_minus, split, trim_margins, Button, ButtonPanel, Input, Rect, Sign, Ui,
};
use crate::state::constants::{NUM_SUB_ENTITIES, NUM_TEMPLATES};
use crate::state::entity::{
  self, Abilities, BareEntity, Full, FullEntity, HalfEntity, Mix, MixEntity, MovementType, Team,
};
use crate::state::geometry::Pos;
use crate::state::materials::Materials;

#[derive(Clone, Debug)]
pub enum EntityEditCommand {
  Exit(MixEntity),
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

#[derive(Debug)]
pub struct EntityEdit {
  rect: Rect,
  pub entity: EntityStates,
  panel: ButtonPanel<Command>,
}

impl EntityEdit {
  fn update_main_panel(&mut self) {
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
      EntityStates::Empty => unreachable!(),
      EntityStates::Entity(e, _) => {
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

        match &e.abilities {
          None => {
            panel.push(Button::<Command>::new(
              trim_margins(rects[2].clone(), 0.1, 0.1, 0.1, 0.1),
              (
                "Add Abilities".to_string(),
                Command::AddAttribute,
                true,
                false,
              ),
            ));
          }
          Some(a) => {
            let third_row_rects: Vec<Rect> =
              split(&rects[2], vec![0.0, 0.25, 0.5, 0.75, 1.0], vec![0.0, 1.0])
                .into_iter()
                .map(|r| trim_margins(r, 0.1, 0.1, 0.1, 0.1))
                .collect();
            panel.append(&mut build_incrementer::<Command>(
              &third_row_rects[0],
              "Speed".to_string(),
              match a.movement_type {
                MovementType::Still => 0,
                MovementType::Walk => 1,
              },
              Command::PM(Attribute::Speed, Sign::Plus),
              Command::PM(Attribute::Speed, Sign::Minus),
            ));
            panel.append(&mut build_incrementer::<Command>(
              &third_row_rects[1],
              "Gun damage".to_string(),
              a.gun_damage,
              Command::PM(Attribute::GunDamage, Sign::Plus),
              Command::PM(Attribute::GunDamage, Sign::Minus),
            ));
            panel.append(&mut build_incrementer::<Command>(
              &third_row_rects[2],
              "Drill damage".to_string(),
              a.drill_damage,
              Command::PM(Attribute::DrillDamage, Sign::Plus),
              Command::PM(Attribute::DrillDamage, Sign::Minus),
            ));
            let fourth_row_rects: Vec<Rect> =
              split(&rects[3], vec![0.0, 0.25, 0.5, 0.75, 1.0], vec![0.0, 1.0])
                .into_iter()
                .map(|r| trim_margins(r, 0.1, 0.1, 0.1, 0.1))
                .collect();
            match a.brain {
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
      }
    }
    self.panel = panel;
  }
}

#[async_trait]
impl Ui for EntityEdit {
  type Command = EntityEditCommand;
  type Builder = EntityStates;

  fn new(rect: Rect, e: EntityStates) -> Self {
    let panel = ButtonPanel::new(rect.clone(), (vec![], vec![], vec![], vec![], vec![]));
    let mut ee = EntityEdit {
      rect,
      entity: e,
      panel,
    };
    ee.update_main_panel();
    ee
  }

  async fn draw(&self) {
    draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, BLACK);
    self.panel.draw().await;
  }

  fn process_input(&mut self, input: Input) -> Option<EntityEditCommand> {
    let command = self.panel.process_input(input.clone());
    if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
      if let EntityStates::Entity(e, _) = &self.entity {
        return Some(EntityEditCommand::Exit(e.clone()));
      }
    }
    match command {
      Some(Command::Exit) => {
        if let EntityStates::Entity(e, _) = &self.entity {
          return Some(EntityEditCommand::Exit(e.clone()));
        }
      }
      Some(Command::PM(attribute, sign)) => {
        if let EntityStates::Entity(mix, _) = &mut self.entity {
          match attribute {
            Attribute::Token => {
              mix.tokens = plus_minus(mix.tokens, sign);
            }
            Attribute::HP => {
              mix.hp = max(plus_minus(mix.hp, sign), 1);
            }
            Attribute::InvSize => {
              mix.inventory_size = plus_minus(mix.inventory_size, sign);
            }
            Attribute::Carbon => {
              mix.materials.carbon = plus_minus(mix.materials.carbon, sign);
            }
            Attribute::Silicon => {
              mix.materials.silicon = plus_minus(mix.materials.silicon, sign);
            }
            Attribute::Plutonium => {
              mix.materials.plutonium = plus_minus(mix.materials.plutonium, sign);
            }
            Attribute::Copper => {
              mix.materials.copper = plus_minus(mix.materials.copper, sign);
            }
            Attribute::Sub1 => {
              if let Some(Abilities {
                brain: Mix::Half(h) | Mix::Full(Full { half: h, .. }),
                ..
              }) = &mut mix.abilities
              {
                h[0] = min(plus_minus(h[0] as usize, sign), NUM_TEMPLATES - 1) as u8;
              }
            }
            Attribute::Sub2 => {
              if let Some(Abilities {
                brain: Mix::Half(h) | Mix::Full(Full { half: h, .. }),
                ..
              }) = &mut mix.abilities
              {
                h[1] = min(plus_minus(h[1] as usize, sign), NUM_TEMPLATES - 1) as u8;
              }
            }
            Attribute::CodeID => {
              if let Some(Abilities {
                brain: Mix::Full(Full { code_index: c, .. }),
                ..
              }) = &mut mix.abilities
              {
                *c = plus_minus(*c, sign);
              }
            }
            Attribute::Gas => {
              if let Some(Abilities {
                brain: Mix::Full(Full { gas: g, .. }),
                ..
              }) = &mut mix.abilities
              {
                *g = plus_minus(*g, sign);
              }
            }
            Attribute::Speed => {
              if let Some(a) = &mut mix.abilities {
                match sign {
                  Sign::Minus => a.movement_type = MovementType::Still,
                  Sign::Plus => a.movement_type = MovementType::Walk,
                }
              }
            }
            Attribute::GunDamage => {
              if let Some(a) = &mut mix.abilities {
                a.gun_damage = plus_minus(a.gun_damage, sign);
              }
            }
            Attribute::DrillDamage => {
              if let Some(a) = &mut mix.abilities {
                a.drill_damage = plus_minus(a.drill_damage, sign);
              }
            }
          };
        }
      }
      None => {}
      Some(Command::AddAttribute) => {
        if let EntityStates::Entity(mix, _) = &mut self.entity {
          mix.abilities = Some(Abilities {
            movement_type: MovementType::Still,
            gun_damage: 0,
            drill_damage: 0,
            message: None,
            brain: Mix::Bare,
          })
        }
      }
      Some(Command::AddConstructs) => {
        if let EntityStates::Entity(mix, _) = &mut self.entity {
          if let Some(a) = &mut mix.abilities {
            a.brain = Mix::Half([0, 0]);
          }
        }
      }
      Some(Command::AddCode) => {
        if let EntityStates::Entity(mix, _) = &mut self.entity {
          if let Some(a) = &mut mix.abilities {
            if let Abilities {
              brain: Mix::Half(h),
              ..
            } = a
            {
              a.brain = Mix::Full(Full {
                half: h.clone(),
                code_index: 0,
                gas: 0,
              });
            }
          }
        }
      }
    }
    self.update_main_panel();
    return None;
  }
}
