pub mod ui;

use crate::ui::landing::{Landing, LandingCommand};
use crate::ui::ui::{get_input, Ui};
use macroquad::prelude::*;

const WIN_WIDTH: f32 = 1500.0;
const WIN_HEIGHT: f32 = 1020.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Gui".to_owned(),
        window_height: WIN_HEIGHT as i32,
        window_width: WIN_WIDTH as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let landing = Landing::new();
    loop {
        clear_background(RED);

        landing.draw().await;

        if let Some(input) = get_input() {
            match landing.get_command(input) {
                LandingCommand::Exit => {
                    break;
                }
                _ => {}
            }
        }

        next_frame().await
    }
}
