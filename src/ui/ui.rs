use async_trait::async_trait;
use macroquad::prelude::*;

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        return Rect { x, y, w, h };
    }
}

pub fn in_rectangle(x: f32, y: f32, rect: &Rect) -> bool {
    x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h
}

pub fn within_rectangle(x: f32, y: f32, rect: &Rect) -> Option<(f32, f32)> {
    if in_rectangle(x, y, rect) {
        Some((x - rect.x, y - rect.y))
    } else {
        None
    }
}

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
    type Command: Clone;

    async fn draw(&self);
    fn get_command(&self, input: Input) -> Self::Command;
}

pub struct Button<T: Clone> {
    pub rect: Rect,
    pub label: String,
    pub command: T,
}

#[async_trait]
impl<T: std::marker::Sync + std::clone::Clone> Ui for Button<T> {
    type Command = T;

    async fn draw(&self) {
        draw_rectangle(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            GREEN,
        );
        draw_text(self.label.as_str(), self.rect.x, self.rect.y, 20.0, BLACK);
    }

    fn get_command(&self, _: Input) -> T {
        self.command.clone()
    }
}
