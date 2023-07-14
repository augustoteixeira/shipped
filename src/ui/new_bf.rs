extern crate rand;
extern crate rand_chacha;

use async_trait::async_trait;
use futures::executor::block_on;
use macroquad::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use super::canvas::draw_floor;
use super::ui::{Input, Rect, Ui};
use crate::state::constants::{HEIGHT, WIDTH};
use crate::state::materials::Materials;
use crate::state::state::Tile;

#[derive(Clone, Debug)]
pub struct NewBF {
    rect: Rect,
    tiles: Vec<Tile>,
    floor: [usize; WIDTH * HEIGHT],
    tileset: Texture2D,
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
        NewBF {
            rect,
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
        }
    }
    async fn draw(&self) {
        draw_floor(800.0, 30.0, &self.tileset, &self.floor).await;
        draw_text("bla", 200.0, 200.0, 40.0, DARKGREEN);
        draw_rectangle(
            800.0,
            30.0,
            16.0 * 60.0,
            16.0 * 30.0,
            Color::new(0.0, 0.0, 0.0, 0.5),
        );
    }
    fn process_input(&mut self, input: Input) -> Option<()> {
        if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
            Some(())
        } else {
            None
        }
    }
}
