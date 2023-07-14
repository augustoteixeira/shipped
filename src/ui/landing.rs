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
    Credits(Credits),
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
            Landing::Credits(c) => c.draw().await,
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
            Landing::Credits(s) => {
                self = &Self::new(Rect::new(0.0, 0.0, 1000.0, 1000.0), ());
                None
            }
        }
    }
}

struct Credits {
    text: String,
}

#[async_trait]
impl Ui for Credits {
    type Command = ();
    type Builder = String;

    fn new(_: Rect, string: String) -> Self {
        Credits { text: string }
    }
    async fn draw(&self) {
        draw_text(self.text.as_str(), 200.0, 200.0, 40.0, DARKGREEN);
    }
    fn get_command(&self, input: Input) -> Option<()> {
        Some(())
    }
}
