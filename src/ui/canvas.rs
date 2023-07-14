use ::rand::prelude::*;
use macroquad::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::state::constants::{HEIGHT, WIDTH};
use crate::state::entity::{FullEntity, MovementType, Team};
use crate::state::geometry::Pos;
use crate::state::materials::Materials;
use crate::state::state::State;

// TODO: Factor this code
pub async fn draw_materials(
    mat: Materials,
    h_displace: f32,
    v_displace: f32,
    j: usize,
    i: usize,
    tileset: &Texture2D,
) {
    let mut rng =
        ChaCha8Rng::seed_from_u64((i * HEIGHT + j).try_into().unwrap());
    for _ in 0..mat.carbon {
        let draw_params = DrawTextureParams {
            source: Some(Rect {
                x: 0.0,
                y: 64.0,
                w: 16.0,
                h: 16.0,
            }),
            ..Default::default()
        };
        draw_texture_ex(
            *tileset,
            h_displace + ((16 * j) + rng.gen_range(0..13)) as f32,
            v_displace + ((16 * i) + rng.gen_range(0..13)) as f32,
            WHITE,
            draw_params,
        );
    }
    for _ in 0..mat.silicon {
        let draw_params = DrawTextureParams {
            source: Some(Rect {
                x: 16.0,
                y: 64.0,
                w: 16.0,
                h: 16.0,
            }),
            ..Default::default()
        };
        draw_texture_ex(
            *tileset,
            h_displace + ((16 * j) + rng.gen_range(0..13)) as f32,
            v_displace + ((16 * i) + rng.gen_range(0..13)) as f32,
            WHITE,
            draw_params,
        );
    }
    for _ in 0..mat.plutonium {
        let draw_params = DrawTextureParams {
            source: Some(Rect {
                x: 32.0,
                y: 64.0,
                w: 16.0,
                h: 16.0,
            }),
            ..Default::default()
        };
        draw_texture_ex(
            *tileset,
            h_displace + ((16 * j) + rng.gen_range(0..13)) as f32,
            v_displace + ((16 * i) + rng.gen_range(0..13)) as f32,
            WHITE,
            draw_params,
        );
    }
    for _ in 0..mat.copper {
        let draw_params = DrawTextureParams {
            source: Some(Rect {
                x: 48.0,
                y: 64.0,
                w: 16.0,
                h: 16.0,
            }),
            ..Default::default()
        };
        draw_texture_ex(
            *tileset,
            h_displace + ((16 * j) + rng.gen_range(0..13)) as f32,
            v_displace + ((16 * i) + rng.gen_range(0..13)) as f32,
            WHITE,
            draw_params,
        );
    }
}

pub async fn draw_entity(
    entity: Option<&FullEntity>,
    h_displace: f32,
    v_displace: f32,
    j: usize,
    i: usize,
    tileset: &Texture2D,
) {
    if let Some(e) = entity {
        let x = get_texture_x(&e);
        let y = match e.team {
            Team::BlueGray | Team::RedGray => 0.0,
            Team::Blue => 16.0,
            Team::Red => 32.0,
        };
        let draw_params = DrawTextureParams {
            source: Some(Rect {
                x,
                y,
                w: 16.0,
                h: 16.0,
            }),
            ..Default::default()
        };
        draw_texture_ex(
            *tileset,
            h_displace + (j as f32) * 16.,
            v_displace + (i as f32) * 16.,
            WHITE,
            draw_params,
        );
        if e.tokens > 0 {
            draw_rectangle(
                h_displace + (j as f32) * 16.,
                v_displace + (i as f32) * 16.,
                2.0,
                2.0,
                LIGHTGRAY,
            );
        }
    }
}

fn get_texture_x(e: &FullEntity) -> f32 {
    let inventory = e.inventory_size;
    let mut abilities = false;
    let mut can_walk = false;
    let mut can_drill = false;
    let mut can_shoot = false;
    if let Some(a) = &e.abilities {
        abilities = true;
        can_walk = a.movement_type == MovementType::Walk;
        can_drill = a.drill_damage > 0;
        can_shoot = a.gun_damage > 0;
    }
    match (inventory, abilities, can_walk, can_drill, can_shoot) {
        (0, false, false, false, false) => 7.0 * 16.0, // wall
        (_, false, false, false, false) => 2.0 * 16.0, // crate
        (_, true, false, false, false) => 1.0 * 16.0,  // arm tower
        (_, true, false, true, false) => 4.0 * 16.0,   // drill tower
        (_, true, false, _, true) => 6.0 * 16.0,       // gun tower
        (_, true, true, false, false) => 0.0,          // arm tank
        (_, true, true, true, false) => 3.0 * 16.0,    // drill tank
        (_, true, true, _, true) => 5.0 * 16.0,        // gun tank
        _ => unreachable! {},
    }
}

pub async fn draw_floor(
    h_displace: f32,
    v_displace: f32,
    tileset: &Texture2D,
    floor: &[usize; WIDTH * HEIGHT],
) {
    for i in 0..HEIGHT {
        for j in 0..WIDTH {
            let f = floor[i * WIDTH + j];
            let draw_params = DrawTextureParams {
                source: Some(Rect {
                    x: 16.0 * (f as f32),
                    y: 48.0,
                    w: 16.0,
                    h: 16.0,
                }),
                ..Default::default()
            };
            draw_texture_ex(
                *tileset,
                h_displace + (16 * j) as f32,
                v_displace + (16 * i) as f32,
                WHITE,
                draw_params,
            );
        }
    }
}

pub async fn draw_map(
    state: &State,
    h_displace: f32,
    v_displace: f32,
    tileset: &Texture2D,
) {
    let mut pos: Pos;
    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            pos = Pos::new(i, j);
            draw_materials(
                state.tiles[pos.to_index()].materials.clone(),
                h_displace,
                v_displace,
                i,
                j,
                &tileset,
            )
            .await;
            draw_entity(
                state.get_entity_option(pos),
                h_displace,
                v_displace,
                i,
                j,
                &tileset,
            )
            .await;
        }
    }
}
