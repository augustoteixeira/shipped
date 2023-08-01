extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use macroquad::prelude::*;

use super::new_bf::EntityStates;
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
pub enum Command {
    Exit,
}

#[derive(Debug)]
pub struct EntityEdit {
    pub entity: EntityStates,
    panel: ButtonPanel<Command>,
}

#[async_trait]
impl Ui for EntityEdit {
    type Command = EntityEditCommand;
    type Builder = ();

    fn new(rect: Rect, _: ()) -> Self {
        let mut panel = ButtonPanel::new(
            rect.clone(),
            (vec![], vec![], vec![], vec![], vec![]),
        );
        panel.push(Button::<Command>::new(
            trim_margins(rect.clone(), 0.3, 0.3, 0.3, 0.3),
            ("Exit".to_string(), Command::Exit, true, false),
        ));
        EntityEdit {
            entity: EntityStates::Empty,
            panel,
        }
    }

    async fn draw(&self) {
        self.panel.draw().await;
    }

    fn process_input(&mut self, input: Input) -> Option<EntityEditCommand> {
        let command = self.panel.process_input(input.clone());
        if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
            return Some(EntityEditCommand::Exit);
        }
        match command {
            Some(Command::Exit) => Some(EntityEditCommand::Exit),
            None => None,
        }
    }
}
