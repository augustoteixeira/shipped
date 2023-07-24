extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use futures::executor::block_on;
use init_array::init_array;
use macroquad::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::iter::zip;

use super::canvas::{draw_floor, draw_mat_map};
use super::ui::{
    draw_centered_text, split, trim_margins, ButtonPanel, Input, Rect, Ui,
};
use crate::state::constants::{HEIGHT, NUM_SUB_ENTITIES, WIDTH};
use crate::state::entity::{BareEntity, FullEntity, HalfEntity};
use crate::state::geometry::{board_iterator, Pos};
use crate::state::materials::Materials;
use crate::state::state::Tile;

const XDISPL: f32 = 800.0;
const YDISPL: f32 = 30.0;

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
pub enum TknButton {
    Tokens,
    MinTkns,
}

#[derive(Clone, Debug)]
pub enum Command {
    MatPM(MatName, Sign),
    MatBrush(MatName),
    Token(TknButton, Sign),
    MapLeftClk(Pos),
    BotNumber(usize, Sign),
    BotBrush(usize),
}

#[derive(Debug)]
pub enum EntityStates {
    Empty,
    Bare(BareEntity),
    Half(HalfEntity),
    Full(FullEntity),
}

#[derive(Debug)]
pub struct NewBF {
    rect: Rect,
    materials: Materials,
    tokens: usize,
    min_tokens: usize,
    brush: Brush,
    tiles: Vec<Tile>,
    entities: [(EntityStates, usize); NUM_SUB_ENTITIES],
    floor: [usize; WIDTH * HEIGHT],
    tileset: Texture2D,
    main_panel: ButtonPanel<Command>,
    bot_panels: [ButtonPanel<Command>; NUM_SUB_ENTITIES],
}

const SMOKE: macroquad::color::Color = Color::new(0.0, 0.0, 0.0, 0.3);

pub fn main_panel(rect: &Rect) -> ButtonPanel<Command> {
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
    // Token buttons
    rects.append(&mut split(
        &rect,
        (2..7).map(|p| (p as f32) * 0.05).collect(),
        vec![0.375, 0.45],
    ));
    labels.append(&mut vec![
        "^".to_string(),
        "v".to_string(),
        "^".to_string(),
        "v".to_string(),
    ]);
    commands.append(&mut vec![
        Command::Token(TknButton::Tokens, Sign::Plus),
        Command::Token(TknButton::Tokens, Sign::Minus),
        Command::Token(TknButton::MinTkns, Sign::Plus),
        Command::Token(TknButton::MinTkns, Sign::Minus),
    ]);
    // collect
    let mut builder: Vec<(Rect, String, Command)> =
        zip(zip(rects, labels), commands)
            .into_iter()
            .map(|((r, l), c)| (trim_margins(r, 0.1, 0.1, 0.1, 0.1), l, c))
            .collect();
    // map grid
    builder.append(
        &mut board_iterator()
            .into_iter()
            .map(|pos| -> (Rect, String, Command) {
                (
                    Rect::new(
                        XDISPL + 16.0 * (pos.x as f32),
                        YDISPL + 16.0 * (pos.y as f32),
                        16.0,
                        16.0,
                    ),
                    "".to_string(),
                    Command::MapLeftClk(pos),
                )
            })
            .collect(),
    );
    let buttons = ButtonPanel::<Command>::new(
        Rect::new(0.0, 0.0, 1000.0, 1000.0),
        builder,
    );
    return buttons;
}

fn bot_panel_builder(
    rect: &Rect,
    index: usize,
) -> Vec<(Rect, String, Command)> {
    let rects: Vec<Rect> =
        split(&rect, vec![0.0, 0.5, 1.0], vec![0.0, 0.5, 1.0]);
    let labels: Vec<String> = vec![
        "^".to_string(),
        "v".to_string(),
        "Edt".to_string(),
        "Sel".to_string(),
    ];
    let commands = vec![
        Command::BotNumber(index, Sign::Plus),
        Command::BotNumber(index, Sign::Minus),
        Command::BotBrush(index),
        Command::BotBrush(index),
    ];
    zip(zip(rects, labels), commands)
        .into_iter()
        .map(|((r, l), c)| (trim_margins(r, 0.1, 0.1, 0.1, 0.1), l, c))
        .collect()
}

fn build_bot_panels(rect: &Rect) -> [ButtonPanel<Command>; NUM_SUB_ENTITIES] {
    init_array(|i| {
        // bot panel
        let bot_rect = split(
            &rect,
            (0..5).map(|p| (p as f32) * 0.1).collect(),
            vec![0.575, 0.725],
        )[i]
            .clone();
        let builder: Vec<(Rect, String, Command)> =
            bot_panel_builder(&bot_rect, i);
        ButtonPanel::<Command>::new(
            Rect::new(0.0, 0.0, 1000.0, 1000.0),
            builder,
        )
    })
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
        let main_panel = main_panel(&rect);
        let bot_panels = build_bot_panels(&rect);
        NewBF {
            rect,
            materials: Materials {
                carbon: 0,
                silicon: 0,
                plutonium: 0,
                copper: 0,
            },
            tokens: 0,
            min_tokens: 0,
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
            entities: init_array(|_| (EntityStates::Empty, 0)),
            floor,
            tileset,
            main_panel,
            bot_panels,
        }
    }

    async fn draw(&self) {
        self.main_panel.draw().await;
        for panel in self.bot_panels.iter() {
            panel.draw().await;
        }
        draw_floor(XDISPL, YDISPL, &self.tileset, &self.floor).await;
        draw_mat_map(&self.tiles, XDISPL, YDISPL, &self.tileset).await;
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
        let tk_rect = split(
            &self.rect,
            (1..4).map(|p| (p as f32) * 0.1).collect(),
            vec![0.275, 0.325, 0.375],
        );
        draw_centered_text(&tk_rect[0], "Tokens").await;
        draw_centered_text(&tk_rect[1], "Min Tkns").await;
        draw_centered_text(
            &tk_rect[2],
            format!("{:05}", self.tokens,).as_str(),
        )
        .await;
        draw_centered_text(
            &tk_rect[3],
            format!("{:05}", self.min_tokens,).as_str(),
        )
        .await;
        let sel = match self.brush {
            Brush::Carbon => &self.main_panel.buttons[8],
            Brush::Silicon => &self.main_panel.buttons[9],
            Brush::Plutonium => &self.main_panel.buttons[10],
            Brush::Copper => &self.main_panel.buttons[11],
            Brush::Bot(i) => &self.bot_panels[i].buttons[3],
        }
        .rect
        .clone();
        draw_rectangle_lines(sel.x, sel.y, sel.w, sel.h, 6.0, RED);
        let bot_rect = split(
            &self.rect,
            (0..5).map(|p| (p as f32) * 0.1).collect(),
            vec![0.475, 0.525, 0.575],
        );
        draw_centered_text(&bot_rect[0], "Bot 0").await;
        draw_centered_text(&bot_rect[1], "Bot 1").await;
        draw_centered_text(&bot_rect[2], "Bot 2").await;
        draw_centered_text(&bot_rect[3], "Bot 3").await;
        draw_centered_text(
            &bot_rect[4],
            format!("{:03}", self.entities[0].1).as_str(),
        )
        .await;
        draw_centered_text(
            &bot_rect[5],
            format!("{:03}", self.entities[1].1).as_str(),
        )
        .await;
        draw_centered_text(
            &bot_rect[6],
            format!("{:03}", self.entities[2].1).as_str(),
        )
        .await;
        draw_centered_text(
            &bot_rect[7],
            format!("{:03}", self.entities[3].1).as_str(),
        )
        .await;
        draw_rectangle(XDISPL, YDISPL, 16.0 * 60.0, 16.0 * 30.0, SMOKE);
        for i in 0..=WIDTH {
            draw_line(
                XDISPL + (i as f32) * 16.0,
                YDISPL,
                XDISPL + (i as f32) * 16.0,
                YDISPL + (60.0 * 16.0),
                1.0,
                SMOKE,
            );
            draw_line(
                XDISPL,
                YDISPL + (i as f32) * 16.0,
                XDISPL + (60.0 * 16.0),
                YDISPL + (i as f32) * 16.0,
                1.0,
                SMOKE,
            );
        }
    }

    fn process_input(&mut self, input: Input) -> Option<()> {
        let command = 'get_command: {
            for i in 0..NUM_SUB_ENTITIES {
                let panel = &mut self.bot_panels[i];
                if let Some(c) = panel.process_input(input.clone()) {
                    break 'get_command Some(c);
                }
            }
            self.main_panel.process_input(input.clone())
        };
        match command {
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
            Some(Command::Token(tk_button, sign)) => match tk_button {
                TknButton::Tokens => match sign {
                    Sign::Plus => {
                        self.tokens += 1;
                    }
                    Sign::Minus => {
                        self.tokens = self.tokens.saturating_sub(1);
                    }
                },
                TknButton::MinTkns => match sign {
                    Sign::Plus => {
                        self.min_tokens += 1;
                    }
                    Sign::Minus => {
                        self.min_tokens = self.min_tokens.saturating_sub(1);
                    }
                },
            },
            Some(Command::MapLeftClk(pos)) => match self.brush {
                Brush::Carbon => {
                    self.tiles[pos.to_index()].materials.carbon += 1;
                    self.tiles[pos.invert().to_index()].materials.carbon += 1;
                }
                Brush::Silicon => {
                    self.tiles[pos.to_index()].materials.silicon += 1;
                    self.tiles[pos.invert().to_index()].materials.silicon += 1;
                }
                Brush::Plutonium => {
                    self.tiles[pos.to_index()].materials.plutonium += 1;
                    self.tiles[pos.invert().to_index()].materials.plutonium +=
                        1;
                }
                Brush::Copper => {
                    self.tiles[pos.to_index()].materials.copper += 1;
                    self.tiles[pos.invert().to_index()].materials.copper += 1;
                }
                _ => {}
            },
            Some(Command::BotNumber(i, sign)) => match sign {
                Sign::Minus => {
                    self.entities[i].1 = self.entities[i].1.saturating_sub(1)
                }
                Sign::Plus => self.entities[i].1 += 1,
            },
            Some(Command::BotBrush(i)) => self.brush = Brush::Bot(i),
        };
        if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
            Some(())
        } else {
            None
        }
    }
}
