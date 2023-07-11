use async_trait::async_trait;

use super::ui::{Button, Input, Rect, Ui};

#[derive(Clone)]
enum Selection {
    LoadBF,
    _CreateBF,
    _UploadCode,
}

pub struct Landing {
    load_button: Button<Selection>,
    //create_button: Button<Selection>,
    //upload_button: Button<Selection>,
}

#[derive(Clone)]
pub enum LandingCommand {
    Nothing,
    Exit,
}

impl Landing {
    pub fn new() -> Self {
        Landing {
            load_button: Button {
                rect: Rect::new(400.0, 300.0, 150.0, 60.0),
                label: "Hello".to_string(),
                command: Selection::LoadBF,
            },
        }
    }
}

#[async_trait]
impl Ui for Landing {
    type Command = LandingCommand;

    async fn draw(&self) {
        self.load_button.draw().await;
    }
    fn get_command(&self, input: Input) -> LandingCommand {
        match input {
            _ => LandingCommand::Exit,
        }
    }
}
