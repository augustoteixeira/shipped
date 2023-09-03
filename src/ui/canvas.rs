use ::rand::prelude::*;
use macroquad::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::state::constants::{HEIGHT, WIDTH};
use crate::state::entity::{ActiveEntity, FullEntity, MovementType, Team};
use crate::state::geometry::{board_iterator, Pos};
use crate::state::materials::Materials;
use crate::state::state::{State, Tile};

// TODO: Factor this code
pub async fn draw_materials(
  mat: Materials,
  h_displace: f32,
  v_displace: f32,
  pos: Pos,
  tileset: &Texture2D,
) {
  let mut rng = ChaCha8Rng::seed_from_u64((pos.x * HEIGHT + pos.y).try_into().unwrap());
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
      h_displace + ((16 * pos.x) + rng.gen_range(0..13)) as f32,
      v_displace + ((16 * (HEIGHT.saturating_sub(pos.y + 1))) + rng.gen_range(0..13)) as f32,
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
      h_displace + ((16 * pos.x) + rng.gen_range(0..13)) as f32,
      v_displace + ((16 * (HEIGHT.saturating_sub(pos.y + 1))) + rng.gen_range(0..13)) as f32,
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
      h_displace + ((16 * pos.x) + rng.gen_range(0..13)) as f32,
      v_displace + ((16 * (HEIGHT.saturating_sub(pos.y + 1))) + rng.gen_range(0..13)) as f32,
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
      h_displace + ((16 * pos.x) + rng.gen_range(0..13)) as f32,
      v_displace + ((16 * (HEIGHT.saturating_sub(pos.y + 1))) + rng.gen_range(0..13)) as f32,
      WHITE,
      draw_params,
    );
  }
}

pub async fn draw_entity(
  entity: Option<&FullEntity>,
  h_displace: f32,
  v_displace: f32,
  pos: Pos,
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
      h_displace + (pos.x as f32) * 16.,
      v_displace + ((HEIGHT.saturating_sub(pos.y + 1)) as f32) * 16.,
      WHITE,
      draw_params,
    );
    if e.tokens > 0 {
      draw_rectangle(
        h_displace + (pos.x as f32) * 16.,
        v_displace + ((HEIGHT.saturating_sub(pos.y + 1)) as f32) * 16.,
        2.0,
        2.0,
        LIGHTGRAY,
      );
    }
  }
}

pub async fn draw_active_entity(
  entity: Option<&ActiveEntity>,
  h_displace: f32,
  v_displace: f32,
  pos: Pos,
  tileset: &Texture2D,
) {
  if let Some(e) = entity {
    let x = get_texture_active_x(&e);
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
      h_displace + (pos.x as f32) * 16.,
      v_displace + ((HEIGHT.saturating_sub(pos.y + 1)) as f32) * 16.,
      WHITE,
      draw_params,
    );
    if e.tokens > 0 {
      draw_rectangle(
        h_displace + (pos.x as f32) * 16.,
        v_displace + ((HEIGHT.saturating_sub(pos.y + 1)) as f32) * 16.,
        2.0,
        2.0,
        LIGHTGRAY,
      );
    }
  }
}

fn get_texture_x(e: &FullEntity) -> f32 {
  let inventory = e.inventory_size;
  let can_walk = e.movement_type == MovementType::Walk;
  let can_drill = e.drill_damage > 0;
  let can_shoot = e.gun_damage > 0;
  match (inventory, can_walk, can_drill, can_shoot) {
    (0, false, false, false) => 7.0 * 16.0, // wall
    (_, false, false, false) => 2.0 * 16.0, // crate
    (_, false, true, false) => 4.0 * 16.0,  // drill tower
    (_, false, _, true) => 6.0 * 16.0,      // gun tower
    (_, true, false, false) => 0.0,         // arm tank
    (_, true, true, false) => 3.0 * 16.0,   // drill tank
    (_, true, _, true) => 5.0 * 16.0,       // gun tank
  }
}

fn get_texture_active_x(e: &ActiveEntity) -> f32 {
  let inventory = e.inventory_size;
  let can_walk = e.movement_type == MovementType::Walk;
  let can_drill = e.drill_damage > 0;
  let can_shoot = e.gun_damage > 0;
  match (inventory, can_walk, can_drill, can_shoot) {
    (0, false, false, false) => 7.0 * 16.0, // wall
    (_, false, false, false) => 2.0 * 16.0, // crate
    (_, false, true, false) => 4.0 * 16.0,  // drill tower
    (_, false, _, true) => 6.0 * 16.0,      // gun tower
    (_, true, false, false) => 0.0,         // arm tank
    (_, true, true, false) => 3.0 * 16.0,   // drill tank
    (_, true, _, true) => 5.0 * 16.0,       // gun tank
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

pub async fn draw_mat_map(
  tiles: &Vec<Tile>,
  h_displace: f32,
  v_displace: f32,
  tileset: &Texture2D,
) {
  for pos in board_iterator() {
    draw_materials(
      tiles[pos.to_index()].materials.clone(),
      h_displace,
      v_displace,
      pos,
      &tileset,
    )
    .await;
  }
}

pub async fn draw_ent_map(state: &State, h_displace: f32, v_displace: f32, tileset: &Texture2D) {
  for pos in board_iterator() {
    draw_active_entity(
      state.get_entity_option(pos),
      h_displace,
      v_displace,
      pos,
      &tileset,
    )
    .await;
  }
}
