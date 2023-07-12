use async_trait::async_trait;

use super::ui::{Button, Grid, Input, Rect, Ui};

#[derive(Clone, Debug)]
enum Selection {
    LoadBF,
    _CreateBF,
    _UploadCode,
}

pub struct LandingSelection {
    buttons: Grid<1, 2, Button<Selection>>,
}

pub enum Landing {
    Selection(LandingSelection),
}

#[derive(Clone)]
pub enum LandingCommand {
    Nothing,
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
                    [("Hello".to_string(), Selection::LoadBF)],
                    [("World".to_string(), Selection::LoadBF)],
                ],
            ),
        })
    }
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
