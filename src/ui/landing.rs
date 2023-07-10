use async_trait::async_trait;

use super::ui::{Input, Ui};
use macroquad::prelude::*;

pub struct Landing {}

pub enum LandingCommand {
    Nothing,
    Exit,
}

#[async_trait]
impl Ui for Landing {
    type Builder = ();
    type State = ();
    type Command = LandingCommand;

    async fn draw(_: f32, _: f32, a_width: f32, a_height: f32, _: ()) {
        draw_rectangle(
            a_width / 2.0 - 60.0,
            a_height / 2.0 - 60.0,
            120.0,
            40.0,
            GREEN,
        );
        draw_text(
            "HELLO",
            a_width / 2.0 - 25.0,
            a_height / 2.0 - 35.0,
            20.0,
            DARKGRAY,
        );
    }
    fn get_command(_: f32, _: f32, _: (), input: Input) -> LandingCommand {
        match input {
            _ => LandingCommand::Exit,
        }
    }
}
