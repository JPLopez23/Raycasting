// main.rs 
#![allow(unused_imports)]
#![allow(dead_code)]

mod line;
mod framebuffer;
mod maze;
mod caster;
mod player;
mod game_state;
mod ui_renderer;
mod sprites;
mod zombie;
mod texture;
mod audio;

use line::line;
use maze::{Maze, load_maze};
use caster::{cast_ray, Intersect};
use framebuffer::Framebuffer;
use player::{Player, process_events};
use game_state::{GameState, GameData};
use ui_renderer::*;
use zombie::Zombie;
use texture::TextureManager;
use audio::AudioManager;
use raylib::prelude::*;
use std::thread;
use std::time::{Duration, Instant};
use std::f32::consts::PI;
use crate::ui_renderer::{render_main_menu_into, render_defeat_screen, draw_timer_into};
use rand::Rng;

fn cell_to_color(cell: char) -> Color {
  match cell {
    '+' => Color::new(120, 120, 180, 255),
    '-' => Color::new(140, 100, 100, 255),
    '|' => Color::new(100, 140, 100, 255),
    '#' => Color::new(200, 60, 60, 255),
    'X' => Color::new(60, 180, 60, 255),
    'g' => Color::new(0, 255, 0, 255),
    'M' => Color::new(0, 150, 255, 255),
    _ => Color::new(100, 100, 100, 255),
  }
}

fn draw_cell(
  framebuffer: &mut Framebuffer,
  xo: usize,
  yo: usize,
  block_size: usize,
  cell: char,
) {
  if cell == ' ' {
    return;
  }
  let color = cell_to_color(cell);
  framebuffer.set_current_color(color);

  for x in xo..xo + block_size {
    for y in yo..yo + block_size {
      framebuffer.set_pixel(x as u32, y as u32);
    }
  }
}

pub fn render_maze(
  framebuffer: &mut Framebuffer,
  maze: &Maze,
  block_size: usize,
  player: &Player,
) {
  for (row_index, row) in maze.iter().enumerate() {
    for (col_index, &cell) in row.iter().enumerate() {
      let xo = col_index * block_size;
      let yo = row_index * block_size;
      draw_cell(framebuffer, xo, yo, block_size, cell);
    }
  }

  framebuffer.set_current_color(Color::WHITESMOKE);

  let num_rays = 5;
  for i in 0..num_rays {
    let current_ray = i as f32 / num_rays as f32;
    let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
    cast_ray(framebuffer, &maze, player, a, block_size, true);
  }
}

fn random_free_spawn(maze: &Vec<Vec<char>>, block_size: usize, rng: &mut impl Rng) -> Option<(f32,f32)> {
    for _ in 0..500 {
        let j = rng.gen_range(0..maze.len());
        let i = rng.gen_range(0..maze[0].len());
        let c = maze[j][i];
        if c == ' ' || c == 'M' || c == 'g' {
            let x = i as f32 * block_size as f32 + (block_size as f32 * 0.5);
            let y = j as f32 * block_size as f32 + (block_size as f32 * 0.5);
            return Some((x, y));
        }
    }
    None
}

fn spawn_zombies(maze: &Vec<Vec<char>>, block_size: usize, count: usize, avoid: Vector2) -> Vec<Zombie> {
    let mut rng = rand::thread_rng();
    let mut zs = Vec::with_capacity(count);
    while zs.len() < count {
        if let Some((x, y)) = random_free_spawn(maze, block_size, &mut rng) {
            let d2 = (x - avoid.x)*(x - avoid.x) + (y - avoid.y)*(y - avoid.y);
            if d2 > (block_size as f32 * 4.0).powi(2) { // no spawnear pegado al jugador
                zs.push(Zombie::new(Vector2::new(x, y)));
            }
        } else { break; }
    }
    zs
}

fn render_world_optimized_into(
  d: &mut RaylibDrawHandle,
  screen_width: f32,
  screen_height: f32,
  maze: &Maze,
  block_size: usize,
  player: &Player,
  zombies: &[Zombie],
  texture_manager: &TextureManager,
  time_s: f32,
) {
  let num_rays = screen_width as usize;
  let hh = screen_height / 2.0;

  fn draw_band_repeat(
      d: &mut RaylibDrawHandle,
      tex: &Texture2D,
      dest: Rectangle,
      scale: f32,
      scroll_x: f32,
      tint: Color,
  ) {
      let tile_w = tex.width() as f32 * scale;
      if tile_w <= 0.5 { return; }
      let src = Rectangle::new(0.0, 0.0, tex.width() as f32, tex.height() as f32);

      let mut start_x = -((scroll_x % tile_w + tile_w) % tile_w);
      while start_x < dest.width {
          let dx = dest.x + start_x;
          d.draw_texture_pro(
              tex,
              src,
              Rectangle::new(dx, dest.y, tile_w, dest.height),
              Vector2::new(0.0, 0.0),
              0.0,
              tint,
          );
          start_x += tile_w;
      }
  }

  d.clear_background(Color::new(30, 30, 40, 255));

  let ceiling_tex = texture_manager.get_ceiling_texture().expect("ceiling tex");
  let floor_tex   = texture_manager.get_floor_texture().expect("floor tex");

  let bands: i32 = 60;
  let screen_w   = screen_width;
  let screen_h   = screen_height;

  let pos_factor  = 0.04;
  let rot_amp     = 2.5;
  let rot_speed   = 0.35;
  let uv_scroll_x = -(player.pos.x * pos_factor + player.a.cos() * (time_s * rot_speed) * rot_amp);

  // Techo
  for b in 0..bands {
      let b = b as f32;
      let y0 = (b / bands as f32) * hh;
      let y1 = ((b + 1.0) / bands as f32) * hh;
      let band_h = (y1 - y0).max(1.0);
      let dest = Rectangle::new(0.0, y0, screen_w, band_h);

      let t = (b + 0.5) / bands as f32;
      let scale = 0.10 + t * 0.90;
      let shade = (0.60 + 0.40 * t).min(1.0);
      let tint = Color::new((255.0*shade) as u8, (255.0*shade) as u8, (255.0*shade) as u8, 255);

      draw_band_repeat(d, ceiling_tex, dest, scale, uv_scroll_x, tint);
  }

  // Suelo
  for b in 0..bands {
      let b = b as f32;
      let y0 = hh + (b / bands as f32) * hh;
      let y1 = hh + ((b + 1.0) / bands as f32) * hh;
      let band_h = (y1 - y0).max(1.0);
      let dest = Rectangle::new(0.0, y0, screen_w, band_h);

      let t = (b + 0.5) / bands as f32;
      let scale = 0.10 + t * 1.10;
      let shade = (0.50 + 0.50 * t).min(1.0);
      let tint = Color::new((240.0*shade) as u8, (240.0*shade) as u8, (240.0*shade) as u8, 255);

      draw_band_repeat(d, floor_tex, dest, scale, uv_scroll_x * 1.2, tint);
  }

  // Muros
  let mut zbuffer = vec![f32::INFINITY; screen_width as usize];

  let distance_to_projection_plane = 100.0;

  for i in (0..num_rays).step_by(2) {
    let current_ray = i as f32 / num_rays as f32;
    let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);

    let mut dummy_fb = Framebuffer::new(1, 1);
    let intersect = cast_ray(&mut dummy_fb, maze, player, a, block_size, false);

    let distance_to_wall = (intersect.distance * (a - player.a).cos()).max(0.0001);
    let mut stake_height = (hh / distance_to_wall) * distance_to_projection_plane;
    if !stake_height.is_finite() { stake_height = screen_h; }
    let stake_height = stake_height.min(screen_h);

    let stake_top = ((hh - (stake_height / 2.0)) as i32).max(0);
    let stake_bottom = ((hh + (stake_height / 2.0)) as i32).min(screen_h as i32);

    let mut wall_color = match intersect.impact {
      '+' | '-' | '|' | '#' => Color::LIGHTGRAY,
      'g' => Color::GREEN,
      'M' => Color::BLUE,
      _   => Color::GRAY,
    };
    let fog_factor = (200.0 / distance_to_wall).clamp(0.3, 1.0);
    wall_color.r = (wall_color.r as f32 * fog_factor) as u8;
    wall_color.g = (wall_color.g as f32 * fog_factor) as u8;
    wall_color.b = (wall_color.b as f32 * fog_factor) as u8;

    if let Some(wtex) = texture_manager.get_wall_texture(intersect.impact) {
        let bs = block_size as f32;
        let fx = (intersect.hit_x.rem_euclid(bs)) / bs;
        let fy = (intersect.hit_y.rem_euclid(bs)) / bs;
        let edge_x = fx.min(1.0 - fx);
        let edge_y = fy.min(1.0 - fy);
        let tex_u = if edge_x < edge_y { fy } else { fx };

        let src_x = (tex_u * wtex.width() as f32).floor().clamp(0.0, (wtex.width() - 1) as f32);
        let src = Rectangle::new(src_x, 0.0, 1.0, wtex.height() as f32);

        let dest = Rectangle::new(i as f32, stake_top as f32, 2.0, stake_height);
        let tint = Color::new(
            (255.0 * fog_factor) as u8,
            (255.0 * fog_factor) as u8,
            (255.0 * fog_factor) as u8,
            255,
        );

        d.draw_texture_pro(wtex, src, dest, Vector2::new(0.0, 0.0), 0.0, tint);

        zbuffer[i] = distance_to_wall;
        if i + 1 < zbuffer.len() {
            zbuffer[i + 1] = distance_to_wall;
        }
    } else {
        d.draw_line(i as i32, stake_top, i as i32, stake_bottom, wall_color);
        zbuffer[i] = distance_to_wall;
        if i + 1 < zbuffer.len() {
            zbuffer[i + 1] = distance_to_wall;
        }
    }
  }

  if let Some(ztex) = texture_manager.get_zombie_texture() {
      let mut order: Vec<(usize, f32)> = zombies.iter()
          .enumerate()
          .map(|(idx, z)| {
              let dx = z.pos.x - player.pos.x;
              let dy = z.pos.y - player.pos.y;
              (idx, (dx*dx + dy*dy).sqrt())
          })
          .collect();
      order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

      let tex_w = ztex.width() as f32;
      let tex_h = ztex.height() as f32;
      let aspect = tex_w / tex_h;

      for (idx, _dist) in order {
          let z = &zombies[idx];
          let dx = z.pos.x - player.pos.x;
          let dy = z.pos.y - player.pos.y;

          let mut ang = dy.atan2(dx) - player.a;
          while ang >  std::f32::consts::PI { ang -= 2.0*std::f32::consts::PI; }
          while ang < -std::f32::consts::PI { ang += 2.0*std::f32::consts::PI; }

          if ang.abs() > player.fov * 0.75 { continue; }

          let perp = (dx*dx + dy*dy).sqrt() * ang.cos();
          if perp <= 0.01 { continue; }

          let sprite_h = (hh / perp) * distance_to_projection_plane;
          if !sprite_h.is_finite() || sprite_h <= 1.0 { continue; }
          let sprite_w = sprite_h * aspect;

          let center_x = ((ang / player.fov) + 0.5) * screen_w;
          let top = (hh - sprite_h/2.0).max(0.0);
          let left = (center_x - sprite_w/2.0).floor() as i32;
          let right = (center_x + sprite_w/2.0).ceil() as i32;

          let sx0 = left.max(0);
          let sx1 = right.min((screen_w as i32) - 1);
          if sx0 >= sx1 { continue; }

          let fog = (220.0 / perp).clamp(0.35, 1.0);
          let tint = Color::new((255.0*fog) as u8, (255.0*fog) as u8, (255.0*fog) as u8, 255);

          for sx in sx0..=sx1 {
              let col = sx as usize;
              if col >= zbuffer.len() { break; }
              if perp >= zbuffer[col] { continue; } 

              let u = ((sx as f32 - (center_x - sprite_w/2.0)) / sprite_w)
                      .clamp(0.0, 1.0);
              let src_x = (u * tex_w).floor().clamp(0.0, tex_w - 1.0);

              let src = Rectangle::new(src_x, 0.0, 1.0, tex_h);
              let dest = Rectangle::new(sx as f32, top, 1.0, sprite_h);

              d.draw_texture_pro(ztex, src, dest, Vector2::new(0.0, 0.0), 0.0, tint);
          }
      }
  }

  render_minimap_direct(d, maze, player, zombies, block_size, screen_width as i32, screen_height as i32);
}


fn render_minimap_direct(
  d: &mut RaylibDrawHandle,
  maze: &Maze,
  player: &Player,
  zombies: &[Zombie],
  block_size: usize,
  screen_width: i32,
  _screen_height: i32,
) {
  let diameter = 140;          
  let radius = diameter / 2;
  let cx = screen_width - radius - 20; 
  let cy = 20 + radius;                

  d.draw_circle(cx, cy, (radius + 4) as f32, Color::new(0, 0, 0, 180));
  d.draw_circle_lines(cx, cy, radius as f32, Color::new(220, 220, 220, 200));

  let rows = maze.len() as f32;
  let cols = maze[0].len() as f32;

  let fill = 0.92;
  let usable = diameter as f32 * fill;

  let scale = (usable / cols).min(usable / rows);

  let draw_w = (cols * scale).round() as i32;
  let draw_h = (rows * scale).round() as i32;

  let origin_x = cx - (draw_w / 2);
  let origin_y = cy - (draw_h / 2);

  let mut draw_rect_in_circle = |x: i32, y: i32, w: i32, h: i32, color: Color| {
    let mx = x + w / 2;
    let my = y + h / 2;
    let dx = mx - cx;
    let dy = my - cy;
    if (dx * dx + dy * dy) <= (radius * radius) {
      d.draw_rectangle(x, y, w, h, color);
    }
  };

  for (r, row) in maze.iter().enumerate() {
    for (c, &cell) in row.iter().enumerate() {
      if cell != ' ' {
        let color = match cell {
          'g' => Color::GREEN,
          'M' => Color::CYAN,
          _   => Color::WHITE,
        };
        let x = origin_x + (c as f32 * scale) as i32;
        let y = origin_y + (r as f32 * scale) as i32;
        let sz = scale.max(2.0) as i32; 
        draw_rect_in_circle(x, y, sz, sz, color);
      }
    }
  }

  let px = origin_x + ((player.pos.x / block_size as f32) * scale) as i32;
  let py = origin_y + ((player.pos.y / block_size as f32) * scale) as i32;

  let dxp = px - cx;
  let dyp = py - cy;
  if (dxp * dxp + dyp * dyp) <= (radius * radius) {
    d.draw_circle(px, py, 3.0, Color::YELLOW);
    let dir_x = px + (player.a.cos() * (radius as f32 * 0.3)) as i32;
    let dir_y = py + (player.a.sin() * (radius as f32 * 0.3)) as i32;
    d.draw_line(px, py, dir_x, dir_y, Color::RED);
  }

  for z in zombies {
    let zx = origin_x + ((z.pos.x / block_size as f32) * scale) as i32;
    let zy = origin_y + ((z.pos.y / block_size as f32) * scale) as i32;

    let dxz = zx - cx;
    let dyz = zy - cy;
    if (dxz * dxz + dyz * dyz) <= (radius * radius) {
      d.draw_circle(zx, zy, 2.0, Color::RED);
    }
  }
}

fn check_collisions_and_pickups(
  player: &Player,
  maze: &mut Maze,
  game_data: &mut GameData,
  block_size: usize,
  audio_manager: &mut Option<AudioManager>,
  rl: &mut RaylibHandle, 
) -> bool {
  let player_i = (player.pos.x / block_size as f32) as usize;
  let player_j = (player.pos.y / block_size as f32) as usize;
  
  if player_j >= maze.len() || player_i >= maze[0].len() {
    return false;
  }
  
  let cell = maze[player_j][player_i];
  
  match cell {
    'M' => {
      maze[player_j][player_i] = ' ';
      game_data.collect_medical_supply();
      if let Some(ref mut audio) = audio_manager {
        audio.play_medical_pickup(rl);
      }
      println!("Medical supply collected! ({}/{})",
        game_data.medical_supplies_collected, game_data.total_medical_supplies);
      false
    },
    'g' => {
      if game_data.can_escape() {
        game_data.game_state = GameState::Victory;
        if let Some(ref mut audio) = audio_manager {
          audio.stop_background_music(rl);
        }
        println!("Victory! You escaped the lab!");
        true
      } else {
        println!("You need to collect all medical supplies before escaping!");
        false
      }
    },
    _ => false
  }
}

fn count_medical_supplies(maze: &Maze) -> u32 {
  let mut count = 0;
  for row in maze {
    for &cell in row {
      if cell == 'M' {
        count += 1;
      }
    }
  }
  count
}

fn main() {
  let window_width = 1300;
  let window_height = 900;
  let block_size = 80;

  let (mut rl, thread) = raylib::init()
    .size(window_width, window_height)
    .title("Zombie Lab Escape - Hospital Edition")
    .build();

  let mut audio_manager = match AudioManager::new(&mut rl, &thread) {
      Ok(manager) => Some(manager),
      Err(e) => {
          eprintln!("Audio initialization failed: {:?}, continuing without audio", e);
          None
      }
  };

  let texture_manager = TextureManager::new(&mut rl, &thread);

  let mut maze = load_maze("maze.txt");
  let mut game_data = GameData::new();
  game_data.total_medical_supplies = count_medical_supplies(&maze);

  let mut player = Player::new(
    Vector2::new(120.0, 120.0),
    PI / 4.0,
    PI / 3.0,
  );

  let mut rng = rand::thread_rng();

  let mut zombies: Vec<Zombie> = {
      let desired = 10usize;
      let mut zs = Vec::with_capacity(desired);
      let mut tries = 0;
      while zs.len() < desired && tries < 2000 {
          tries += 1;
          let j = rng.gen_range(0..maze.len());
          let i = rng.gen_range(0..maze[0].len());
          let c = maze[j][i];
          if c == ' ' || c == 'M' || c == 'g' {
              let x = i as f32 * block_size as f32 + (block_size as f32 * 0.5);
              let y = j as f32 * block_size as f32 + (block_size as f32 * 0.5);
              let dx = x - player.pos.x;
              let dy = y - player.pos.y;
              if dx*dx + dy*dy > (block_size as f32 * 4.0).powi(2) {
                  zs.push(Zombie::new(Vector2::new(x, y)));
              }
          }
      }
      if zs.is_empty() {
          vec![
              Zombie::new(Vector2::new(200.0, 200.0)),
              Zombie::new(Vector2::new(400.0, 300.0)),
              Zombie::new(Vector2::new(600.0, 400.0)),
          ]
      } else { zs }
  };

  let mut frame_count = 0;
  let mut fps = 0;
  let mut last_fps_update = Instant::now();

  let mut defeat_sound_played = false;
  let mut victory_sound_played = false;
  let mut menu_sound_played = false;

  rl.set_target_fps(60);

  println!("Hospital Escape initialized. Starting main loop...");
  println!("Current game state: {:?}", game_data.game_state);

  while !rl.window_should_close() {
    frame_count += 1;
    let current_time = Instant::now();

    if current_time.duration_since(last_fps_update).as_secs() >= 1 {
      fps = frame_count;
      frame_count = 0;
      last_fps_update = current_time;
    }

    if let Some(ref mut audio) = audio_manager {
      audio.update(&mut rl);
    }

    match game_data.game_state {
      GameState::MainMenu => {
        if !menu_sound_played {
          if let Some(ref mut audio) = audio_manager {
            audio.play_hola_sound(&mut rl);
          }
          menu_sound_played = true;
        }

        let start_pressed =
          rl.is_key_pressed(KeyboardKey::KEY_ENTER) ||
          rl.is_key_pressed(KeyboardKey::KEY_SPACE);

        if start_pressed {
          println!("Entering the abandoned hospital...");
          game_data.reset();                         
          game_data.game_state = GameState::Playing; 

          defeat_sound_played = false;
          victory_sound_played = false;
          menu_sound_played = false;

          // Respawn zombies
          zombies.clear();
          let desired = 10usize;
          let mut tries = 0;
          while zombies.len() < desired && tries < 2000 {
              tries += 1;
              let j = rng.gen_range(0..maze.len());
              let i = rng.gen_range(0..maze[0].len());
              let c = maze[j][i];
              if c == ' ' || c == 'M' || c == 'g' {
                  let x = i as f32 * block_size as f32 + (block_size as f32 * 0.5);
                  let y = j as f32 * block_size as f32 + (block_size as f32 * 0.5);
                  let dx = x - player.pos.x;
                  let dy = y - player.pos.y;
                  if dx*dx + dy*dy > (block_size as f32 * 4.0).powi(2) {
                      zombies.push(Zombie::new(Vector2::new(x, y)));
                  }
              }
          }

          if let Some(ref mut audio) = audio_manager {
            audio.play_background_music(&mut rl);
          }

          continue; 
        }

        let millis = (rl.get_time() * 1000.0) as u128;
        let blink_color = if (millis / 600) % 2 == 0 {
          Color::new(0, 255, 100, 255)
        } else {
          Color::new(0, 150, 50, 255)
        };

        {
          let mut d = rl.begin_drawing(&thread);
          render_main_menu_into(&mut d, window_width, window_height, blink_color);
        }
      }

      GameState::Playing => {
        if let Some(ref mut audio) = audio_manager {
          if !audio.is_music_playing() {
            audio.play_background_music(&mut rl);
          }
        }

        if game_data.start_time.elapsed() >= game_data.time_limit {
            game_data.game_state = GameState::Defeat;
            if let Some(ref mut audio) = audio_manager {
                audio.stop_background_music(&mut rl);
            }
            println!("Time's up! You lose.");
            continue;
        }

        let dt = rl.get_frame_time();       
        let time_s = rl.get_time() as f32;  

        process_events(&mut player, &rl, &maze, block_size);

        for z in &mut zombies {
            z.update(&maze, block_size, player.pos, dt);
        }

        check_collisions_and_pickups(
            &player, &mut maze, &mut game_data, block_size, &mut audio_manager, &mut rl
        );

        {
            let lose_dist = 22.0_f32; 
            let lose_dist2 = lose_dist * lose_dist;
            let mut touched = false;
            for z in &zombies {
                let dx = z.pos.x - player.pos.x;
                let dy = z.pos.y - player.pos.y;
                if dx*dx + dy*dy <= lose_dist2 {
                    touched = true;
                    break;
                }
            }
            if touched {
                game_data.game_state = GameState::Defeat;
                if let Some(ref mut audio) = audio_manager {
                    audio.stop_background_music(&mut rl);
                }
                println!("A zombie got you!");
                continue;
            }
        }

        let screen_w = rl.get_screen_width() as f32;
        let screen_h = rl.get_screen_height() as f32;

        {
            let mut d = rl.begin_drawing(&thread);

            render_world_optimized_into(
                &mut d, screen_w, screen_h,
                &maze, block_size, &player, &zombies, &texture_manager,
                time_s,
            );

            let fps_color = if fps >= 15 { Color::new(0, 255, 100, 255) } else { Color::RED };
            d.draw_rectangle(10, 10, 140, 25, Color::new(0, 0, 0, 200));
            d.draw_text(&format!("SYSTEMS: {} FPS", fps), 15, 15, 18, fps_color);

            let supplies_text = format!(
                "MEDICAL SUPPLIES: {}/{}",
                game_data.medical_supplies_collected, game_data.total_medical_supplies
            );
            d.draw_rectangle(10, 45, 300, 25, Color::new(0, 0, 0, 200));
            d.draw_text(&supplies_text, 15, 50, 18, Color::new(100, 200, 255, 255));

            let remaining = game_data.time_remaining();
            draw_timer_into(&mut d, 10, 80, remaining);

            if !game_data.can_escape() {
                d.draw_rectangle(10, 115, 250, 25, Color::new(50, 0, 0, 200));
                d.draw_text("FIND ANTIDOTE SUPPLIES!", 15, 120, 16, Color::new(255, 200, 0, 255));
            } else {
                d.draw_rectangle(10, 115, 200, 25, Color::new(0, 50, 0, 200));
                d.draw_text("REACH EMERGENCY EXIT!", 15, 120, 16, Color::new(0, 255, 100, 255));
            }

            d.draw_text(
                "Mouse - Look | WASD - Move",
                10, window_height - 30, 14, Color::new(180, 180, 180, 255)
            );
        } 
      }

      GameState::Defeat => {
        if !defeat_sound_played {
            if let Some(ref mut audio) = audio_manager {
                audio.stop_background_music(&mut rl);
                audio.play_hola_sound(&mut rl);
            }
            defeat_sound_played = true;
        }

        render_defeat_screen(&mut rl, &thread, &game_data);

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            defeat_sound_played = false;
            victory_sound_played = false;
            menu_sound_played = false;

            maze = load_maze("maze.txt");
            game_data = GameData::new();
            game_data.total_medical_supplies = count_medical_supplies(&maze);
            player = Player::new(Vector2::new(120.0, 120.0), std::f32::consts::PI / 4.0, std::f32::consts::PI / 3.0);

            // respawn aleatorio
            zombies.clear();
            let desired = 10usize;
            let mut tries = 0;
            while zombies.len() < desired && tries < 2000 {
                tries += 1;
                let j = rng.gen_range(0..maze.len());
                let i = rng.gen_range(0..maze[0].len());
                let c = maze[j][i];
                if c == ' ' || c == 'M' || c == 'g' {
                    let x = i as f32 * block_size as f32 + (block_size as f32 * 0.5);
                    let y = j as f32 * block_size as f32 + (block_size as f32 * 0.5);
                    let dx = x - player.pos.x;
                    let dy = y - player.pos.y;
                    if dx*dx + dy*dy > (block_size as f32 * 4.0).powi(2) {
                        zombies.push(Zombie::new(Vector2::new(x, y)));
                    }
                }
            }

            game_data.game_state = GameState::MainMenu;
        }
      }

      GameState::Victory => {
          if !victory_sound_played {
              if let Some(ref mut audio) = audio_manager {
                  audio.stop_background_music(&mut rl);
                  audio.play_victory_sound(&mut rl);
              }
              victory_sound_played = true;
          }

          {
            let mut d = rl.begin_drawing(&thread);
            render_victory_screen_into(&mut d, &game_data, window_width, window_height);
          }

          if rl.is_key_pressed(KeyboardKey::KEY_R) {
            defeat_sound_played = false;
            victory_sound_played = false;
            menu_sound_played = false;

            maze = load_maze("maze.txt");
            game_data = GameData::new();
            game_data.total_medical_supplies = count_medical_supplies(&maze);
            player = Player::new(Vector2::new(120.0, 120.0), std::f32::consts::PI / 4.0, std::f32::consts::PI / 3.0);

            // respawn aleatorio
            zombies.clear();
            let desired = 10usize;
            let mut tries = 0;
            while zombies.len() < desired && tries < 2000 {
                tries += 1;
                let j = rng.gen_range(0..maze.len());
                let i = rng.gen_range(0..maze[0].len());
                let c = maze[j][i];
                if c == ' ' || c == 'M' || c == 'g' {
                    let x = i as f32 * block_size as f32 + (block_size as f32 * 0.5);
                    let y = j as f32 * block_size as f32 + (block_size as f32 * 0.5);
                    let dx = x - player.pos.x;
                    let dy = y - player.pos.y;
                    if dx*dx + dy*dy > (block_size as f32 * 4.0).powi(2) {
                        zombies.push(Zombie::new(Vector2::new(x, y)));
                    }
                }
            }

            game_data.game_state = GameState::MainMenu;
          }
      }
    }
  }
}