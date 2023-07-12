use async_trait::async_trait;

use super::ui::{Button, Grid, Input, Rect, Ui};

#[derive(Clone)]
enum Selection {
    LoadBF,
    _CreateBF,
    _UploadCode,
}

pub struct LandingSelection {
    buttons: Grid<1, 1, Button<Selection>>,
}

pub enum Landing {
    Selection(LandingSelection),
}

#[derive(Clone)]
pub enum LandingCommand {
    Nothing,
    Exit,
}

impl Landing {
    pub fn new() -> Self {
        Landing::Selection(LandingSelection {
            buttons: Grid {
                rect: Rect::new(400.0, 300.0, 150.0, 60.0),
                components: [[Button {
                    rect: Rect::new(400.0, 300.0, 150.0, 60.0),
                    label: "Hello".to_string(),
                    command: Selection::LoadBF,
                }]],
            },
        })
    }
}

#[async_trait]
impl Ui for Landing {
    type Command = LandingCommand;
    async fn draw(&self) {
        match &self {
            Landing::Selection(s) => s.buttons.draw().await,
        }
    }
    fn get_command(&self, input: Input) -> LandingCommand {
        match &self {
            Landing::Selection(s) => match input {
                _ => LandingCommand::Exit,
            },
        }
    }
}
