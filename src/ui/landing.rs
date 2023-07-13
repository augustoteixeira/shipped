use async_trait::async_trait;
use macroquad::prelude::*;

use super::ui::{Button, Grid, Input, Rect, Ui};

#[derive(Clone, Debug)]
enum Selection {
    LoadBF,
    CreateBF,
    UploadCode,
    Credits,
    Quit,
}

pub struct LandingSelection {
    buttons: Grid<1, 5, Button<Selection>>,
}

pub enum Landing {
    Selection(LandingSelection),
}

#[derive(Clone)]
pub enum LandingCommand {
    Exit,
}

#[async_trait]
impl Ui for Landing {
    type Command = LandingCommand;
    type Builder = ();

    fn new(rect: Rect, _: ()) -> Self {
        Landing::Selection(LandingSelection {
            buttons: Grid::new(
                rect,
                [
                    [("Load Battlefield".to_string(), Selection::LoadBF)],
                    [("Create Battlefield".to_string(), Selection::CreateBF)],
                    [("Upload Code".to_string(), Selection::UploadCode)],
                    [("Credits".to_string(), Selection::Credits)],
                    [("Quit".to_string(), Selection::Quit)],
                ],
            ),
        })
    }
    async fn draw(&self) {
        match &self {
            Landing::Selection(s) => s.buttons.draw().await,
        }
    }
    fn get_command(&self, input: Input) -> Option<LandingCommand> {
        match &self {
            Landing::Selection(s) => {
                if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) =
                    input
                {
                    return Some(LandingCommand::Exit);
                }
                match s.buttons.get_command(input) {
                    Some(Selection::Quit) => Some(LandingCommand::Exit),
                    _ => None,
                }
            }
        }
    }
}
