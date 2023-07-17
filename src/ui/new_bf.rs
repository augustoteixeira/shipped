extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use futures::executor::block_on;
use macroquad::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::borrow::BorrowMut;
use std::iter::zip;

use super::canvas::draw_floor;
use super::ui::{
    draw_centered_text, split, trim_margins, ButtonPanel, Input, Rect, Ui,
};
use crate::state::constants::{HEIGHT, WIDTH};
use crate::state::materials::Materials;
use crate::state::state::Tile;

#[derive(Clone, Debug)]
pub enum MatName {
    Carbon,
    Silicon,
    Plutonium,
    Copper,
}

#[derive(Clone, Debug)]
pub enum Brush {
    Carbon,
    Silicon,
    Plutonium,
    Copper,
    Bot(usize),
}

#[derive(Clone, Debug)]
pub enum Sign {
    Plus,
    Minus,
}

#[derive(Clone, Debug)]
pub enum Command {
    MatPM(MatName, Sign),
    MatBrush(MatName),
}

#[derive(Debug)]
pub struct NewBF {
    rect: Rect,
    materials: Materials,
    brush: Brush,
    tiles: Vec<Tile>,
    floor: [usize; WIDTH * HEIGHT],
    tileset: Texture2D,
    buttons: ButtonPanel<Command>,
}

const SMOKE: macroquad::color::Color = Color::new(0.0, 0.0, 0.0, 0.5);

pub fn bf_panel(rect: &Rect) -> ButtonPanel<Command> {
    // Material +- buttons
    let mut rects: Vec<Rect> = split(
        &rect,
        (0..9).map(|p| (p as f32) * 0.05).collect(),
        vec![0.1, 0.175],
    );
    let mut labels: Vec<String> = vec![
        "^".to_string(),
        "v".to_string(),
        "^".to_string(),
        "v".to_string(),
        "^".to_string(),
        "v".to_string(),
        "^".to_string(),
        "v".to_string(),
    ];
    let mut commands = vec![
        Command::MatPM(MatName::Carbon, Sign::Plus),
        Command::MatPM(MatName::Carbon, Sign::Minus),
        Command::MatPM(MatName::Silicon, Sign::Plus),
        Command::MatPM(MatName::Silicon, Sign::Minus),
        Command::MatPM(MatName::Plutonium, Sign::Plus),
        Command::MatPM(MatName::Plutonium, Sign::Minus),
        Command::MatPM(MatName::Copper, Sign::Plus),
        Command::MatPM(MatName::Copper, Sign::Minus),
    ];
    // Material brush buttons
    rects.append(&mut split(
        &rect,
        (0..5).map(|p| (p as f32) * 0.1).collect(),
        vec![0.175, 0.25],
    ));
    labels.append(&mut vec!["Use".to_string(); 4]);
    commands.append(&mut vec![
        Command::MatBrush(MatName::Carbon),
        Command::MatBrush(MatName::Silicon),
        Command::MatBrush(MatName::Plutonium),
        Command::MatBrush(MatName::Copper),
    ]);
    let builder = zip(zip(rects, labels), commands)
        .into_iter()
        .map(|((r, l), c)| (trim_margins(r, 0.1, 0.1, 0.1, 0.1), l, c))
        .collect();
    let buttons = ButtonPanel::<Command>::new(
        Rect::new(0.0, 0.0, 1000.0, 1000.0),
        builder,
    );
    return buttons;
}

#[async_trait]
impl Ui for NewBF {
    type Command = ();
    type Builder = ();

    fn new(rect: Rect, _: ()) -> Self {
        let tileset = block_on(load_texture("assets/tileset.png")).unwrap();
        let mut rng: ChaCha8Rng =
            ChaCha8Rng::seed_from_u64(25).try_into().unwrap();
        let mut floor = [0; WIDTH * HEIGHT];
        for i in 0..(WIDTH * HEIGHT) {
            floor[i] = rng.gen_range(0..7);
        }
        let buttons = bf_panel(&rect);
        NewBF {
            rect,
            materials: Materials {
                carbon: 0,
                silicon: 0,
                plutonium: 0,
                copper: 0,
            },
            brush: Brush::Carbon,
            tiles: (0..(WIDTH * HEIGHT))
                .map(|_| Tile {
                    entity_id: None,
                    materials: Materials {
                        carbon: 0,
                        silicon: 0,
                        plutonium: 0,
                        copper: 0,
                    },
                })
                .collect(),
            floor,
            tileset,
            buttons,
        }
    }
    async fn draw(&self) {
        let x_disp = 800.0;
        let y_disp = 30.0;
        self.buttons.draw().await;
        draw_floor(x_disp, y_disp, &self.tileset, &self.floor).await;
        let mat_rect = split(
            &self.rect,
            (0..5).map(|p| (p as f32) * 0.1).collect(),
            vec![0.0, 0.05, 0.1],
        );
        draw_centered_text(&mat_rect[0], "Carbon").await;
        draw_centered_text(&mat_rect[1], "Silicon").await;
        draw_centered_text(&mat_rect[2], "Plutonion").await;
        draw_centered_text(&mat_rect[3], "Copper").await;
        draw_centered_text(
            &mat_rect[4],
            format!("{:05}", self.materials.carbon,).as_str(),
        )
        .await;
        draw_centered_text(
            &mat_rect[5],
            format!("{:05}", self.materials.silicon,).as_str(),
        )
        .await;
        draw_centered_text(
            &mat_rect[6],
            format!("{:05}", self.materials.plutonium,).as_str(),
        )
        .await;
        draw_centered_text(
            &mat_rect[7],
            format!("{:05}", self.materials.copper,).as_str(),
        )
        .await;
        let sel = self.buttons.buttons[match self.brush {
            Brush::Carbon => 8,
            Brush::Silicon => 9,
            Brush::Plutonium => 10,
            Brush::Copper => 11,
            _ => 11,
        }]
        .rect
        .clone();
        draw_rectangle_lines(sel.x, sel.y, sel.w, sel.h, 6.0, RED);
        draw_rectangle(x_disp, y_disp, 16.0 * 60.0, 16.0 * 30.0, SMOKE);
        for i in 0..=WIDTH {
            draw_line(
                x_disp + (i as f32) * 16.0,
                y_disp,
                x_disp + (i as f32) * 16.0,
                y_disp + (60.0 * 16.0),
                1.0,
                SMOKE,
            );
            draw_line(
                x_disp,
                y_disp + (i as f32) * 16.0,
                x_disp + (60.0 * 16.0),
                y_disp + (i as f32) * 16.0,
                1.0,
                SMOKE,
            );
        }
    }
    fn process_input(&mut self, input: Input) -> Option<()> {
        match self.buttons.process_input(input.clone()) {
            None => {}
            Some(Command::MatPM(mat_name, sign)) => match mat_name {
                MatName::Carbon => match sign {
                    Sign::Plus => {
                        self.materials.carbon += 1;
                    }
                    Sign::Minus => {
                        self.materials.carbon =
                            self.materials.carbon.saturating_sub(1);
                    }
                },
                MatName::Silicon => match sign {
                    Sign::Plus => {
                        self.materials.silicon += 1;
                    }
                    Sign::Minus => {
                        self.materials.silicon =
                            self.materials.silicon.saturating_sub(1);
                    }
                },
                MatName::Plutonium => match sign {
                    Sign::Plus => {
                        self.materials.plutonium += 1;
                    }
                    Sign::Minus => {
                        self.materials.plutonium =
                            self.materials.plutonium.saturating_sub(1);
                    }
                },
                MatName::Copper => match sign {
                    Sign::Plus => {
                        self.materials.copper += 1;
                    }
                    Sign::Minus => {
                        self.materials.copper =
                            self.materials.copper.saturating_sub(1);
                    }
                },
            },
            Some(Command::MatBrush(mat)) => {
                self.brush = match mat {
                    MatName::Carbon => Brush::Carbon,
                    MatName::Silicon => Brush::Silicon,
                    MatName::Plutonium => Brush::Plutonium,
                    MatName::Copper => Brush::Copper,
                }
            }
        };
        if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
            Some(())
        } else {
            None
        }
    }
}
