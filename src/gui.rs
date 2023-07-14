pub mod state;
pub mod ui;

use crate::ui::landing::{Landing, LandingCommand};
use crate::ui::ui::{get_input, Rect, Ui};
use macroquad::prelude::*;

const WIN_WIDTH: f32 = 1800.0;
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
    let mut landing =
        Landing::new(Rect::new(0.0, 0.0, WIN_WIDTH, WIN_HEIGHT), ());
    loop {
        clear_background(BLACK);

        landing.draw().await;

        if let Some(input) = get_input() {
            match landing.process_input(input) {
                Some(LandingCommand::Exit) => {
                    break;
                }
                _ => {}
            }
        }

        next_frame().await
    }
}
