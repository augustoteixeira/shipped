use async_trait::async_trait;
use init_array::*;
use macroquad::prelude::*;
use std::iter::zip;

#[derive(Debug, Clone)]
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

pub fn trim_margins(rect: Rect, t: f32, b: f32, l: f32, r: f32) -> Rect {
    assert!(t >= 0.0 && b >= 0.0 && l >= 0.0 && r >= 0.0);
    assert!(t + b <= 1.0 && l + r <= 1.0);
    Rect::new(
        rect.x + l * rect.w,
        rect.y + t * rect.h,
        rect.w * (1.0 - l - r),
        rect.h * (1.0 - t - b),
    )
}

pub fn split(rect: &Rect, x_tics: Vec<f32>, y_tics: Vec<f32>) -> Vec<Rect> {
    let mut result: Vec<Rect> = vec![];
    for y in y_tics.windows(2) {
        for x in x_tics.windows(2) {
            result.push(Rect::new(
                rect.x + rect.w * x[0],
                rect.y + rect.h * y[0],
                rect.x + rect.w * (x[1] - x[0]),
                rect.y + rect.h * (y[1] - y[0]),
            ));
        }
    }
    result
}

pub async fn draw_centered_text(rect: &Rect, text: &str) {
    let center = get_text_center(&text, None, 40, 1.0, 0.0);
    draw_text(
        &text,
        rect.x + rect.w / 2.0 - center.x,
        rect.y + rect.h / 2.0 - center.y,
        40.0,
        DARKGREEN,
    );
}

#[derive(Debug, Clone)]
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
    type Builder;

    fn new(rect: Rect, builder: Self::Builder) -> Self;
    async fn draw(&self);
    fn process_input(&mut self, input: Input) -> Option<Self::Command>;
}

pub struct Text {
    rect: Rect,
    contents: String,
}

#[async_trait]
impl Ui for Text {
    type Command = ();
    type Builder = String;

    fn new(rect: Rect, builder: String) -> Self {
        Text {
            rect,
            contents: builder,
        }
    }
    async fn draw(&self) {
        let TextDimensions {
            width: t_w,
            height: t_h,
            ..
        } = measure_text(self.contents.as_str(), None, 40, 1.0);
        draw_text(
            self.contents.as_str(),
            self.rect.x + self.rect.w / 2.0 - t_w / 2.0,
            self.rect.y + self.rect.h / 2.0 + t_h / 2.0,
            40.0,
            DARKGREEN,
        );
    }
    fn process_input(&mut self, _: Input) -> Option<()> {
        None
    }
}

#[derive(Debug)]
pub struct Button<T: Clone + core::fmt::Debug> {
    pub rect: Rect,
    label: String,
    command: T,
}

#[async_trait]
impl<T: Sync + Clone + core::fmt::Debug> Ui for Button<T> {
    type Command = T;
    type Builder = (String, T);

    fn new(rect: Rect, builder: (String, T)) -> Self {
        Button {
            rect,
            label: builder.0,
            command: builder.1,
        }
    }
    async fn draw(&self) {
        draw_rectangle_lines(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            4.0,
            DARKGREEN,
        );
        let TextDimensions {
            width: t_w,
            height: t_h,
            ..
        } = measure_text(self.label.as_str(), None, 40, 1.0);
        draw_text(
            self.label.as_str(),
            self.rect.x + self.rect.w / 2.0 - t_w / 2.0,
            self.rect.y + self.rect.h / 2.0 + t_h / 2.0,
            40.0,
            DARKGREEN,
        );
    }

    fn process_input(&mut self, input: Input) -> Option<T> {
        if let Input::Click(MouseButton::Left, (x, y)) = input {
            if in_rectangle(x, y, &self.rect) {
                return Some(self.command.clone());
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct ButtonPanel<T: Clone + core::fmt::Debug> {
    pub buttons: Vec<Button<T>>,
}

#[async_trait]
impl<T: Sync + Clone + core::fmt::Debug + Send> Ui for ButtonPanel<T> {
    type Command = T;
    type Builder = Vec<(Rect, String, T)>;

    fn new(_: Rect, builder: Vec<(Rect, String, T)>) -> Self {
        ButtonPanel {
            buttons: builder
                .into_iter()
                .map(|(rect, s, t)| Button {
                    rect,
                    label: s,
                    command: t,
                })
                .collect(),
        }
    }
    async fn draw(&self) {
        for b in &self.buttons {
            b.draw().await;
        }
    }

    fn process_input(&mut self, input: Input) -> Option<T> {
        if let Input::Click(MouseButton::Left, (x, y)) = input {
            for b in &self.buttons {
                if in_rectangle(x, y, &b.rect) {
                    return Some(b.command.clone());
                }
            }
        }
        None
    }
}

pub struct Grid<const N: usize, const M: usize, C: Ui> {
    pub rect: Rect,
    pub components: [[C; N]; M],
}

#[async_trait]
impl<
        const N: usize,
        const M: usize,
        C: Ui + Sync + Send + core::fmt::Debug,
    > Ui for Grid<N, M, C>
{
    type Command = <C>::Command;
    type Builder = [[<C>::Builder; N]; M];

    fn new(rect: Rect, builder: [[<C>::Builder; N]; M]) -> Self {
        let x = rect.x;
        let y = rect.y;
        let x_delta = rect.w / (N as f32);
        let y_delta = rect.h / (M as f32);
        let x_pos: [[f32; N]; M] =
            [init_array(|i| x + (i as f32) * x_delta); M];
        let y_pos: [[f32; N]; M] =
            init_array(|i| [y + y_delta * (i as f32); N]);
        let components = zip(zip(x_pos, y_pos), builder)
            .into_iter()
            .map(|((x_row, y_row), c_row)| {
                let row = zip(zip(x_row, y_row), c_row);
                row.into_iter()
                    .map(|((x_displace, y_displace), c)| {
                        C::new(
                            Rect::new(x_displace, y_displace, x_delta, y_delta),
                            c,
                        )
                    })
                    .collect::<Vec<C>>()
                    .try_into()
                    .unwrap()
            })
            .collect::<Vec<[C; N]>>()
            .try_into()
            .unwrap();
        Grid::<N, M, C> {
            rect: rect.clone(),
            components,
        }
    }
    async fn draw(&self) {
        for i in 0..N {
            for j in 0..M {
                self.components[j][i].draw().await;
            }
        }
    }

    fn process_input(&mut self, input: Input) -> Option<<C>::Command> {
        if let Input::Click(MouseButton::Left, (x, y)) = input {
            for i in 0..N {
                for j in 0..M {
                    let r = self.rect.clone();
                    let y_delta = r.h / (M as f32);
                    let x_delta = r.w / (N as f32);
                    if in_rectangle(
                        x,
                        y,
                        &Rect::new(
                            r.x,
                            r.y + (j as f32) * y_delta,
                            x_delta,
                            y_delta,
                        ),
                    ) {
                        return self.components[j][i]
                            .process_input(input)
                            .clone();
                    }
                }
            }
        }
        None
    }
}

// pub struct ButtonVec {
//     pub rect: Rect,
//     pub components: [[C; N]; M],
// }
