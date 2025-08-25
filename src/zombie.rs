// zombie.rs
use raylib::prelude::*;

pub struct Zombie {
    pub pos: Vector2,
    pub speed: f32,
    pub radius: f32,
}

impl Zombie {
    pub fn new(pos: Vector2) -> Self {
        Self {
            pos,
            speed: 75.0,  
            radius: 18.0, 
        }
    }

    pub fn update(&mut self, maze: &Vec<Vec<char>>, block_size: usize, target: Vector2, dt: f32) {
        let mut dir = target - self.pos;
        let len = (dir.x * dir.x + dir.y * dir.y).sqrt();
        if len > 0.001 {
            dir.x /= len;
            dir.y /= len;
        } else {
            return;
        }

        let step = Vector2::new(dir.x * self.speed * dt, dir.y * self.speed * dt);
        let candidate = Vector2::new(self.pos.x + step.x, self.pos.y + step.y);

        if is_walkable(maze, candidate, block_size) {
            self.pos = candidate;
        } else {
            let cand_x = Vector2::new(self.pos.x + step.x, self.pos.y);
            let cand_y = Vector2::new(self.pos.x, self.pos.y + step.y);
            if is_walkable(maze, cand_x, block_size) {
                self.pos = cand_x;
            } else if is_walkable(maze, cand_y, block_size) {
                self.pos = cand_y;
            }
        }
    }
}

fn is_walkable(maze: &Vec<Vec<char>>, p: Vector2, block_size: usize) -> bool {
    if p.x < 0.0 || p.y < 0.0 { return false; }
    let i = (p.x / block_size as f32) as isize;
    let j = (p.y / block_size as f32) as isize;
    if i < 0 || j < 0 { return false; }
    let (i, j) = (i as usize, j as usize);
    if j >= maze.len() || i >= maze[0].len() { return false; }
    match maze[j][i] {
        ' ' | 'M' | 'g' => true, 
        _ => false,              
    }
}
