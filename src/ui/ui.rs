use async_trait::async_trait;
use macroquad::prelude::*;

pub enum Input {
    Key(KeyCode),
    Click(MouseButton, (f32, f32)),
}

pub fn get_input() -> Option<Input> {
    let optional_key = get_last_key_pressed();
    match optional_key {
        None => {}
        Some(k) => {
            return Some(Input::Key(k));
        }
    };
    if is_mouse_button_pressed(MouseButton::Left) {
        return Some(Input::Click(MouseButton::Left, mouse_position()));
    }
    if is_mouse_button_pressed(MouseButton::Right) {
        return Some(Input::Click(MouseButton::Right, mouse_position()));
    }
    None
}

#[async_trait]
pub trait Ui {
    type Builder;
    type State;
    type Command;

    async fn draw(
        x: f32,
        y: f32,
        avail_w: f32,
        avail_h: f32,
        state: Self::State,
    );
    fn get_command(
        avail_w: f32,
        avail_h: f32,
        state: Self::State,
        input: Input,
    ) -> Self::Command;
}
