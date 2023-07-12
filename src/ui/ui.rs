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
    fn get_command(&self, input: Input) -> Self::Command;
}

#[derive(Debug)]
pub struct Button<T: Clone + core::fmt::Debug> {
    rect: Rect,
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
            2.0,
            GREEN,
        );
        draw_text(self.label.as_str(), self.rect.x, self.rect.y, 20.0, BLACK);
    }

    fn get_command(&self, _: Input) -> T {
        self.command.clone()
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
        // let mut components: [[C; N]; M] =
        //     unsafe { MaybeUninit::uninit().assume_init() };
        // for (i, column) in &mut components[..].into_iter().enumerate() {
        //     for (j, element) in &mut column[..].into_iter().enumerate() {
        //         std::ptr::write(
        //             element,
        //             C::new(Rect::new(x, y, x_delta, y_delta), builder[i][j]),
        //         );
        //         x += x_delta;
        //     }
        //     y += y_delta;
        // }
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
            components, // : builder
                        // .into_iter()
                        // .map(|row| {
                        //     row.into_iter()
                        //         .map(|c| {
                        //             let c = C::new(
                        //                 Rect::new(
                        //                     x + (i as f32) * x_delta,
                        //                     y,
                        //                     x_delta,
                        //                     y_delta,
                        //                 ),
                        //                 builder[i][j].clone(),
                        //             );
                        //             i += 1;
                        //             c
                        //         })
                        //         .collect::<Vec<C>>()
                        //         .try_into()
                        //         .unwrap()
                        // })
                        // .collect::<Vec<[C; N]>>()
                        // .try_into()
                        // .unwrap(),
        }
    }
    async fn draw(&self) {
        for i in 0..N {
            for j in 0..M {
                self.components[j][i].draw().await;
            }
        }
    }

    fn get_command(&self, input: Input) -> <C>::Command {
        self.components[0][0].get_command(input).clone()
    }
}
