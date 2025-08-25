// caster.rs
use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub struct Intersect {
  pub distance: f32,
  pub impact: char,
  pub hit_x: f32,
  pub hit_y: f32,
}

pub fn cast_ray(
  framebuffer: &mut Framebuffer,
  maze: &Maze,
  player: &Player,
  a: f32,
  block_size: usize,
  draw_line: bool,
) -> Intersect {
  let mut d = 1.0;
  framebuffer.set_current_color(Color::WHITESMOKE);

  loop {
    let cos = d * a.cos();
    let sin = d * a.sin();
    let xf = player.pos.x + cos;
    let yf = player.pos.y + sin;
    let x = xf as usize;
    let y = yf as usize;

    let i = x / block_size;
    let j = y / block_size;

    if j >= maze.len() || i >= maze[j].len() {
      return Intersect{
        distance: d,
        impact: '#',
        hit_x: player.pos.x + d * a.cos(),   
        hit_y: player.pos.y + d * a.sin(),   
      };
    }

    if maze[j][i] != ' ' {
      return Intersect{
        distance: d,
        impact: maze[j][i],
        hit_x: player.pos.x + d * a.cos(),   
        hit_y: player.pos.y + d * a.sin(),   
      };
    }

    if draw_line {
      framebuffer.set_pixel(x as u32, y as u32);
    }

    d += 1.0;
  }
}
