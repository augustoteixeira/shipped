extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use futures::executor::block_on;
use init_array::init_array;
use macroquad::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use super::canvas::{draw_floor, draw_mat_map};
use super::ui::{
    build_incrementer, split, trim_margins, Button, ButtonPanel, Input, Rect,
    Ui,
};
use crate::state::constants::{HEIGHT, NUM_SUB_ENTITIES, WIDTH};
use crate::state::entity::{BareEntity, FullEntity, HalfEntity};
use crate::state::geometry::{board_iterator, Pos};
use crate::state::materials::Materials;
use crate::state::state::Tile;

const XDISPL: f32 = 800.0;
const YDISPL: f32 = 30.0;

const SMOKE: macroquad::color::Color = Color::new(0.0, 0.0, 0.0, 0.3);

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
    panel: ButtonPanel<Command>,
    //bot_panels: [ButtonPanel<Command>; NUM_SUB_ENTITIES],
}

impl NewBF {
    fn build_material_panel(&self, rect: &Rect) -> ButtonPanel<Command> {
        let mut rects: Vec<Rect> = split(
            &rect.clone(),
            vec![0.0, 0.25, 0.5, 0.75, 1.0],
            vec![0.0, 0.75],
        );
        let mut panel: ButtonPanel<Command> = build_incrementer::<Command>(
            &rects[0],
            "Carbon".to_string(),
            self.materials.carbon,
            Command::MatPM(MatName::Carbon, Sign::Plus),
            Command::MatPM(MatName::Carbon, Sign::Minus),
        );
        panel.append(&mut build_incrementer::<Command>(
            &rects[1],
            "Silicon".to_string(),
            self.materials.silicon,
            Command::MatPM(MatName::Silicon, Sign::Plus),
            Command::MatPM(MatName::Silicon, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
            &rects[2],
            "Pluton.".to_string(),
            self.materials.plutonium,
            Command::MatPM(MatName::Plutonium, Sign::Plus),
            Command::MatPM(MatName::Plutonium, Sign::Minus),
        ));
        panel.append(&mut build_incrementer::<Command>(
            &rects[3],
            "Copper".to_string(),
            self.materials.copper,
            Command::MatPM(MatName::Copper, Sign::Plus),
            Command::MatPM(MatName::Copper, Sign::Minus),
        ));
        // Material brush buttons
        rects = split(
            &rect.clone(),
            vec![0.0, 0.25, 0.5, 0.75, 1.0],
            vec![0.75, 1.0],
        )
        .into_iter()
        //.map(|r| split(&r.clone(), vec![0.0, 1.0], vec![0.75, 1.0])[0].clone())
        .collect();
        let labels = vec!["Use".to_string(); 4];
        let commands = vec![
            Command::MatBrush(MatName::Carbon),
            Command::MatBrush(MatName::Silicon),
            Command::MatBrush(MatName::Plutonium),
            Command::MatBrush(MatName::Copper),
        ];
        let activities: Vec<bool> = [true; 4].into();
        let mut alerts: Vec<bool> = [false; 4].into();
        match self.brush {
            Brush::Carbon => alerts[0] = true,
            Brush::Silicon => alerts[1] = true,
            Brush::Plutonium => alerts[2] = true,
            Brush::Copper => alerts[3] = true,
            Brush::Bot(_) => {}
        }
        let mut butts = ButtonPanel::<Command>::new(
            Rect::new(0.0, 0.0, 1000.0, 1000.0),
            (rects, labels, commands, activities, alerts),
        );
        panel.append(&mut butts);
        panel
    }

    fn build_token_panel(&self, rect: &Rect) -> ButtonPanel<Command> {
        let rects =
            split(&rect.clone(), vec![0.25, 0.5, 0.75], vec![0.0, 0.75]);
        let mut panel: ButtonPanel<Command> = build_incrementer::<Command>(
            &rects[0],
            "Tokens".to_string(),
            self.tokens,
            Command::Token(TknButton::Tokens, Sign::Plus),
            Command::Token(TknButton::Tokens, Sign::Minus),
        );
        panel.append(&mut build_incrementer(
            &rects[1],
            "Min Tks".to_string(),
            self.min_tokens,
            Command::Token(TknButton::MinTkns, Sign::Plus),
            Command::Token(TknButton::MinTkns, Sign::Minus),
        ));
        panel
    }

    fn build_bot_panels(&self, rect: &Rect) -> ButtonPanel<Command> {
        let mut panel = ButtonPanel::new(
            rect.clone(),
            (vec![], vec![], vec![], vec![], vec![]),
        );
        let bot_rect =
            split(rect, vec![0.0, 0.25, 0.5, 0.75, 1.0], vec![0.0, 1.0]);
        for i in 0..NUM_SUB_ENTITIES {
            panel.append(&mut self.bot_panel_builder(&bot_rect[i], i));
        }
        panel
    }

    fn bot_panel_builder(
        &self,
        rect: &Rect,
        index: usize,
    ) -> ButtonPanel<Command> {
        let rects: Vec<Rect> = split(rect, vec![0.0, 1.0], vec![0.0, 1.0]);
        let panel: ButtonPanel<Command> = build_incrementer::<Command>(
            &rects[0],
            format!("Bot {}", index).to_string(),
            0,
            Command::BotNumber(index, Sign::Plus),
            Command::BotNumber(index, Sign::Minus),
        );
        let labels: Vec<String> = vec!["Edt".to_string(), "Sel".to_string()];
        let commands = vec![Command::BotBrush(index), Command::BotBrush(index)];
        ButtonPanel::<Command>::new(
            Rect::new(0.0, 0.0, 1000.0, 1000.0),
            (rects, labels, commands, [true; 2].into(), [false; 2].into()),
        );
        panel
    }

    fn update_main_panel(&mut self) {
        let left_rect = trim_margins(
            split(&self.rect, vec![0.0, 0.45, 1.0], vec![0.0, 1.0])[0].clone(),
            0.05,
            0.05,
            0.05,
            0.05,
        );
        let rects: Vec<Rect> =
            split(&left_rect, vec![0.0, 1.0], vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        let mut button_panel = self.build_material_panel(&rects[0]);
        button_panel.append(&mut self.build_token_panel(&rects[1]));
        button_panel.append(&mut self.build_bot_panels(&rects[2]));
        for pos in board_iterator().into_iter() {
            button_panel.push(Button::<Command>::new(
                Rect::new(
                    XDISPL + 16.0 * (pos.x as f32),
                    YDISPL + 16.0 * (pos.y as f32),
                    16.0,
                    16.0,
                ),
                ("".to_string(), Command::MapLeftClk(pos), true, false),
            ))
        }
        self.panel = button_panel;
    }
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
        let mut new_bf = NewBF {
            rect: rect.clone(),
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
            panel: ButtonPanel::new(
                rect,
                (vec![], vec![], vec![], vec![], vec![]),
            ),
        };
        new_bf.update_main_panel();
        new_bf
    }

    async fn draw(&self) {
        self.panel.draw().await;
        draw_floor(XDISPL, YDISPL, &self.tileset, &self.floor).await;
        draw_mat_map(&self.tiles, XDISPL, YDISPL, &self.tileset).await;
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
        let command = self.panel.process_input(input.clone());
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
        self.update_main_panel();
        if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
            Some(())
        } else {
            None
        }
    }
}
