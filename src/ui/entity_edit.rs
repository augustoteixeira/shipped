extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use macroquad::prelude::*;

use super::new_bf::{EntityStates, Sign};
use super::ui::{
    build_incrementer, split, trim_margins, Button, ButtonPanel, Input, Rect,
    Ui,
};
use crate::state::constants::NUM_TEMPLATES;
use crate::state::entity::{
    Abilities, BareEntity, FullEntity, HalfEntity, MovementType, Team,
};
use crate::state::geometry::Pos;
use crate::state::materials::Materials;

#[derive(Clone, Debug)]
pub enum EntityEditCommand {
    Exit,
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
}

#[derive(Clone, Debug)]
pub enum Command {
    Exit,
    PM(Attribute, Sign),
}

#[derive(Debug)]
pub struct EntityEdit {
    rect: Rect,
    pub entity: EntityStates,
    panel: ButtonPanel<Command>,
}

impl EntityEdit {
    fn update_main_panel(&mut self) {
        let rects: Vec<Rect> =
            split(&self.rect, vec![0.0, 1.0], vec![0.0, 0.25, 0.5, 0.85, 1.0]);
        let mut panel = ButtonPanel::new(
            self.rect.clone(),
            (vec![], vec![], vec![], vec![], vec![]),
        );
        panel.push(Button::<Command>::new(
            trim_margins(rects[3].clone(), 0.1, 0.1, 0.1, 0.1),
            ("Exit".to_string(), Command::Exit, true, false),
        ));
        let tokens;
        let hp;
        let inv_size;
        let carbon;
        let silicon;
        let plutonium;
        let copper;
        match &self.entity {
            EntityStates::Empty => unreachable!(),
            EntityStates::Bare(bare, _) => {
                tokens = bare.tokens;
                hp = bare.hp;
                inv_size = bare.inventory_size;
                carbon = bare.materials.carbon;
                silicon = bare.materials.silicon;
                plutonium = bare.materials.plutonium;
                copper = bare.materials.copper;
            }
            EntityStates::Half(bare, _) => {
                tokens = bare.tokens;
                hp = bare.hp;
                inv_size = bare.inventory_size;
                carbon = bare.materials.carbon;
                silicon = bare.materials.silicon;
                plutonium = bare.materials.plutonium;
                copper = bare.materials.copper;
            }
            EntityStates::Full(bare, _) => {
                tokens = bare.tokens;
                hp = bare.hp;
                inv_size = bare.inventory_size;
                carbon = bare.materials.carbon;
                silicon = bare.materials.silicon;
                plutonium = bare.materials.plutonium;
                copper = bare.materials.copper;
            }
        }
        let first_row_rects: Vec<Rect> =
            split(&rects[0], vec![0.0, 0.25, 0.5, 0.75, 1.0], vec![0.0, 1.0]);
        panel.append(&mut build_incrementer::<Command>(
            &first_row_rects[0],
            "Tokens".to_string(),
            tokens,
            Command::PM(Attribute::Token, Sign::Plus),
            Command::PM(Attribute::Token, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
            &first_row_rects[1],
            "HP".to_string(),
            hp,
            Command::PM(Attribute::HP, Sign::Plus),
            Command::PM(Attribute::HP, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
            &first_row_rects[2],
            "Inv. Size".to_string(),
            inv_size,
            Command::PM(Attribute::InvSize, Sign::Plus),
            Command::PM(Attribute::InvSize, Sign::Minus),
        ));
        let second_row_rects: Vec<Rect> =
            split(&rects[1], vec![0.0, 0.25, 0.5, 0.75, 1.0], vec![0.0, 1.0]);
        panel.append(&mut build_incrementer::<Command>(
            &second_row_rects[0],
            "Carbon".to_string(),
            carbon,
            Command::PM(Attribute::Carbon, Sign::Plus),
            Command::PM(Attribute::Carbon, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
            &second_row_rects[1],
            "Silicon".to_string(),
            silicon,
            Command::PM(Attribute::Silicon, Sign::Plus),
            Command::PM(Attribute::Silicon, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
            &second_row_rects[2],
            "Plutonium".to_string(),
            plutonium,
            Command::PM(Attribute::Plutonium, Sign::Plus),
            Command::PM(Attribute::Plutonium, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
            &second_row_rects[3],
            "Copper".to_string(),
            copper,
            Command::PM(Attribute::Copper, Sign::Plus),
            Command::PM(Attribute::Copper, Sign::Minus),
        ));

        match &self.entity {
            EntityStates::Empty => unreachable!(),
            EntityStates::Bare(bare, _) => {}
            EntityStates::Half(half, _) => {}
            EntityStates::Full(full, _) => {}
        }
        self.panel = panel;
    }
}

#[async_trait]
impl Ui for EntityEdit {
    type Command = EntityEditCommand;
    type Builder = EntityStates;

    fn new(rect: Rect, e: EntityStates) -> Self {
        let panel = ButtonPanel::new(
            rect.clone(),
            (vec![], vec![], vec![], vec![], vec![]),
        );
        let mut ee = EntityEdit {
            rect,
            entity: e,
            panel,
        };
        ee.update_main_panel();
        ee
    }

    async fn draw(&self) {
        draw_rectangle(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            BLACK,
        );
        self.panel.draw().await;
    }

    fn process_input(&mut self, input: Input) -> Option<EntityEditCommand> {
        let command = self.panel.process_input(input.clone());
        if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
            return Some(EntityEditCommand::Exit);
        }
        match command {
            Some(Command::Exit) => return Some(EntityEditCommand::Exit),
            Some(Command::PM(attribute, sign)) => {
                match attribute {
                    Attribute::Token => {}
                    Attribute::HP => {}
                    Attribute::InvSize => {}
                    Attribute::Carbon => {}
                    Attribute::Silicon => {}
                    Attribute::Plutonium => {}
                    Attribute::Copper => {}
                    Attribute::Sub1 => {}
                    Attribute::Sub2 => {}
                    Attribute::CodeID => {}
                    Attribute::Gas => {}
                };
                *j += 1;
            }
            None => {}
        }
        return None;
    }
}
