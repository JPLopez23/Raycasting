// player.rs

use raylib::prelude::*;
use raylib::consts::{KeyboardKey, GamepadAxis, GamepadButton};

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32,
    last_mouse_x: f32,
}

impl Player {
    pub fn new(pos: Vector2, a: f32, fov: f32) -> Self {
        Player {
            pos,
            a,
            fov,
            last_mouse_x: 0.0,
        }
    }
}

pub fn process_events(
    player: &mut Player,
    rl: &RaylibHandle,
    maze: &crate::maze::Maze,
    block_size: usize,
) {
    let dt               = rl.get_frame_time().max(0.0001);
    let base_move_speed  = 230.0;
    let base_strafe_speed= 230.0;
    let sprint_mult      = 1.6;           // L3 para sprint
    let turn_speed_keys  = 2.9;           // rad/seg flechas
    let mouse_sens       = 0.0030;        // rad por pixel
    let pad_turn_sens    = 2.9;           // rad/seg por stick derecho
    let dz               = 0.15;          // deadzone

    // Vectores base 
    let fwd   = Vector2 { x: player.a.cos(),  y: player.a.sin()  }; // adelante
    let left  = Vector2 { x: player.a.sin(),  y: -player.a.cos() }; // izquierda 
    let right = Vector2 { x: -player.a.sin(), y: player.a.cos()  }; // derecha 

    let mut wish = Vector2 { x: 0.0, y: 0.0 };

    // W/S 
    if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
        wish.x += fwd.x;  wish.y += fwd.y;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN) {
        wish.x -= fwd.x;  wish.y -= fwd.y;
    }

    if rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_Q) {
        wish.x += left.x; wish.y += left.y;  
    }
    if rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_E) {
        wish.x += right.x; wish.y += right.y; 
    }

    // Rotación por teclas
    let mut turn_axis = 0.0f32;
    if rl.is_key_down(KeyboardKey::KEY_LEFT)  { turn_axis -= 1.0; }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) { turn_axis += 1.0; }
    player.a += turn_axis * turn_speed_keys * dt;

    // Rotación por mouse 
    let mdx = rl.get_mouse_delta().x;
    player.a += mdx * mouse_sens;

    // Gamepad 
    let mut move_speed   = base_move_speed;
    let mut strafe_speed = base_strafe_speed;

    if rl.is_gamepad_available(0) {
        if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_THUMB) {
            move_speed   *= sprint_mult;
            strafe_speed *= sprint_mult;
        }

        let mut lx = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_X);
        let mut ly = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);
        if lx.abs() < dz { lx = 0.0; }
        if ly.abs() < dz { ly = 0.0; }

        wish.x += (-ly) * fwd.x;   wish.y += (-ly) * fwd.y;   
        wish.x += ( lx) * right.x; wish.y += ( lx) * right.y; 

        let mut rx = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_RIGHT_X);
        if rx.abs() < dz { rx = 0.0; }
        player.a += rx * pad_turn_sens * dt;

        if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_1)
            || rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_2)
        {
            wish.x += left.x; wish.y += left.y; 
        }
        if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1)
            || rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_2)
        {
            wish.x += right.x; wish.y += right.y; 
        }
    }

    while player.a >  std::f32::consts::PI { player.a -= 2.0*std::f32::consts::PI; }
    while player.a <= -std::f32::consts::PI { player.a += 2.0*std::f32::consts::PI; }

    let mag = (wish.x*wish.x + wish.y*wish.y).sqrt();
    if mag > 0.0001 {
        let dir = Vector2 { x: wish.x / mag, y: wish.y / mag };

        let vx = dir.x * move_speed * dt;
        let vy = dir.y * strafe_speed * dt;

        let can_pass = |nx: f32, ny: f32| -> bool {
            let i = (nx / block_size as f32) as usize;
            let j = (ny / block_size as f32) as usize;
            if j >= maze.len() || i >= maze[0].len() { return false; }
            let cell = maze[j][i];
            cell == ' ' || cell == 'M' || cell == 'g'
        };

        // Eje X
        let nx = player.pos.x + vx;
        if can_pass(nx, player.pos.y) { player.pos.x = nx; }
        // Eje Y
        let ny = player.pos.y + vy;
        if can_pass(player.pos.x, ny) { player.pos.y = ny; }
    }
}


fn can_move_to(pos: Vector2, maze: &crate::maze::Maze, block_size: usize) -> bool {
    let i = (pos.x / block_size as f32) as usize;
    let j = (pos.y / block_size as f32) as usize;
    
    if maze.is_empty() || maze[0].is_empty() {
        return false;
    }
    
    if j >= maze.len() || i >= maze[0].len() {
        return false;
    }
    
    let cell = maze[j][i];
    cell == ' ' || cell == 'M' || cell == 'g'
}