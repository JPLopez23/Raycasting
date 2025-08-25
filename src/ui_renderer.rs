// ui_renderer.rs 

use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::game_state::{GameState, GameData};
use std::time::Duration;

fn draw_centered_text(
    d: &mut RaylibDrawHandle,
    text: &str,
    y: i32,
    font_size: i32,
    color: Color,
    window_width: i32,
) {
    let text_width = d.measure_text(text, font_size);
    let x = (window_width - text_width) / 2;
    d.draw_text(text, x, y, font_size, color);
}

pub fn draw_timer_into(d: &mut RaylibDrawHandle, pos_x: i32, pos_y: i32, remaining: Duration) {
    let secs = remaining.as_secs();
    let mins = (secs / 60) as i32;
    let s = (secs % 60) as i32;
    let text = format!("TIME: {:02}:{:02}", mins, s);

    d.draw_rectangle(pos_x, pos_y, 120, 22, Color::new(0, 0, 0, 200));
    let color = if secs <= 20 { Color::RED } else { Color::new(0, 255, 180, 255) };
    d.draw_text(&text, pos_x + 6, pos_y + 4, 18, color);
}

pub fn render_defeat_screen(
    window: &mut RaylibHandle,
    thread: &RaylibThread,
    _game_data: &GameData,
) {
    let sw = window.get_screen_width();
    let sh = window.get_screen_height();

    let mut d = window.begin_drawing(thread);
    d.clear_background(Color::new(10, 10, 15, 255));

    let mut y = (sh as f32 * 0.40) as i32;

    draw_centered_text(&mut d, "TIME'S UP!", y, 50, Color::RED, sw);

    y += 60;
    draw_centered_text(
        &mut d,
        "Zombies took over the hospital...",
        y,
        22,
        Color::LIGHTGRAY,
        sw,
    );

    y += 50;
    draw_centered_text(
        &mut d,
        "Press R to try again",
        y,
        22,
        Color::new(0, 255, 150, 255),
        sw,
    );
}

pub fn render_minimap(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
) {
    let minimap_radius = 150; 
    let center_x = framebuffer.width as i32 - minimap_radius - 25;
    let center_y = minimap_radius + 25;
    
    let maze_height = maze.len();
    let maze_width = if maze_height > 0 { maze[0].len() } else { 0 };
    
    let scale_x = (minimap_radius as f32 * 1.2) / maze_width as f32;
    let scale_y = (minimap_radius as f32 * 1.2) / maze_height as f32;
    let scale = scale_x.min(scale_y) * 1.5; 

    let in_circle = |x: i32, y: i32| -> bool {
        let dx = x - center_x;
        let dy = y - center_y;
        (dx * dx + dy * dy) <= (minimap_radius * minimap_radius)
    };

    framebuffer.set_current_color(Color::new(0, 0, 0, 180));
    for x in (center_x - minimap_radius)..(center_x + minimap_radius) {
        for y in (center_y - minimap_radius)..(center_y + minimap_radius) {
            let dx = x - center_x;
            let dy = y - center_y;
            if dx * dx + dy * dy <= minimap_radius * minimap_radius {
                if x >= 0 && y >= 0 && x < framebuffer.width as i32 && y < framebuffer.height as i32 {
                    framebuffer.set_pixel(x as u32, y as u32);
                }
            }
        }
    }

    framebuffer.set_current_color(Color::new(100, 150, 255, 255));
    for angle in 0..360 {
        let rad = angle as f32 * 3.14159 / 180.0;
        let border_x = center_x + (minimap_radius as f32 * rad.cos()) as i32;
        let border_y = center_y + (minimap_radius as f32 * rad.sin()) as i32;
        
        if border_x >= 0 && border_y >= 0 && 
           border_x < framebuffer.width as i32 && border_y < framebuffer.height as i32 {
            framebuffer.set_pixel(border_x as u32, border_y as u32);
        }
    }

    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            if cell != ' ' {
                let color = match cell {
                    'g' => Color::new(0, 255, 0, 255),     // Salida 
                    'M' => Color::new(0, 200, 255, 255),   // Suministros 
                    '+' => Color::new(200, 200, 200, 255), // Esquinas
                    '-' | '|' => Color::new(160, 160, 160, 255), // Paredes
                    '#' => Color::new(220, 100, 100, 255), // Paredes sólidas
                    _ => Color::new(140, 140, 140, 255),
                };
                framebuffer.set_current_color(color);

                let offset_x = center_x - (minimap_radius * 2 / 3);
                let offset_y = center_y - (minimap_radius * 2 / 3);
                
                let pixel_x = offset_x + (col_index as f32 * scale) as i32;
                let pixel_y = offset_y + (row_index as f32 * scale) as i32;
                
                let pixel_size = (scale.max(2.0)) as i32; 
                
                for px in pixel_x..(pixel_x + pixel_size) {
                    for py in pixel_y..(pixel_y + pixel_size) {
                        if px >= 0 && py >= 0 && 
                           px < framebuffer.width as i32 && py < framebuffer.height as i32 &&
                           in_circle(px, py) {
                            framebuffer.set_pixel(px as u32, py as u32);
                        }
                    }
                }
            }
        }
    }

    let player_maze_x = player.pos.x / block_size as f32;
    let player_maze_y = player.pos.y / block_size as f32;
    
    let offset_x = center_x - (minimap_radius * 2 / 3);
    let offset_y = center_y - (minimap_radius * 2 / 3);
    
    let player_x = offset_x + (player_maze_x * scale) as i32;
    let player_y = offset_y + (player_maze_y * scale) as i32;

    framebuffer.set_current_color(Color::new(255, 255, 0, 255));
    for dx in -3..=3 {
        for dy in -3..=3 {
            if dx * dx + dy * dy <= 9 { 
                let px = player_x + dx;
                let py = player_y + dy;
                if px >= 0 && py >= 0 && 
                   px < framebuffer.width as i32 && py < framebuffer.height as i32 &&
                   in_circle(px, py) {
                    framebuffer.set_pixel(px as u32, py as u32);
                }
            }
        }
    }

    framebuffer.set_current_color(Color::new(255, 100, 100, 255));
    let dir_length = 20.0;
    let steps = 20;
    for i in 0..steps {
        let t = (i as f32) / (steps as f32);
        let line_x = player_x as f32 + t * player.a.cos() * dir_length;
        let line_y = player_y as f32 + t * player.a.sin() * dir_length;
        
        let lx = line_x as i32;
        let ly = line_y as i32;
        
        if lx >= 0 && ly >= 0 && 
           lx < framebuffer.width as i32 && ly < framebuffer.height as i32 &&
           in_circle(lx, ly) {
            framebuffer.set_pixel(lx as u32, ly as u32);
        }
    }
}

pub fn render_main_menu_into(
    d: &mut RaylibDrawHandle,
    window_width: i32,
    window_height: i32,
    blink_color: Color,
) {
    d.clear_background(Color::new(20, 20, 30, 255));

    draw_centered_text(d, "ZOMBIE HOSPITAL ESCAPE", 100, 42, Color::new(220, 60, 60, 255), window_width);
    draw_centered_text(d, "ABANDONED MEDICAL FACILITY", 145, 22, Color::new(170, 170, 170, 255), window_width);

    draw_centered_text(
        d,
        "Find all medical supplies and reach the emergency exit.",
        200, 20, Color::RAYWHITE, window_width,
    );
    draw_centered_text(
        d,
        "You have 5 minutes before the facility is lost.",
        225, 20, Color::SKYBLUE, window_width,
    );

    // Controles - Teclado/Mouse
    draw_centered_text(d, "KEYBOARD & MOUSE", 270, 20, Color::ORANGE, window_width);
    draw_centered_text(d, "W/S = Forward/Back | A/Q = Strafe Left | D/E = Strafe Right", 300, 18, Color::LIGHTGRAY, window_width);
    draw_centered_text(d, "Mouse = Look (Left/Right) | Arrow Left/Right = Rotate", 325, 18, Color::LIGHTGRAY, window_width);

    // Controles - Gamepad
    draw_centered_text(d, "GAMEPAD SUPPORTED", 365, 20, Color::ORANGE, window_width);
    draw_centered_text(d, "Left Stick = Move (X=Strafe, Y=Forward/Back) | Right Stick X = Look", 395, 18, Color::LIGHTGRAY, window_width);
    draw_centered_text(d, "L1/LB or L2/LT = Strafe Left | R1/RB or R2/RT = Strafe Right | L3 = Sprint", 420, 18, Color::LIGHTGRAY, window_width);

    draw_centered_text(
        d,
        "Emergency power active. Systems unstable.",
        460, 18, Color::YELLOW, window_width,
    );

    draw_centered_text(d, "Press ENTER", 510, 24, blink_color, window_width);

    draw_centered_text(d, "Good luck, survivor...", 560, 18, Color::DARKGRAY, window_width);
}

pub fn render_victory_screen_into(
    d: &mut RaylibDrawHandle,
    game_data: &GameData,
    screen_width: i32,
    screen_height: i32,
) {
    d.clear_background(Color::new(10, 10, 10, 255));

    let title = "YOU ESCAPED!";
    let subtitle = "The lab doors open. Fresh air at last.";
    let stats = format!(
        "Supplies collected: {}/{}",
        game_data.medical_supplies_collected, game_data.total_medical_supplies
    );
    let hint = "Press R to restart";

    let mut y = (screen_height as f32 * 0.35) as i32;

    draw_centered_text(d, title, y, 48, Color::new(0, 230, 120, 255), screen_width);

    y += 60;
    draw_centered_text(d, subtitle, y, 22, Color::LIGHTGRAY, screen_width);

    y += 40;
    draw_centered_text(d, &stats, y, 22, Color::new(100, 200, 255, 255), screen_width);

    y += 70;
    draw_centered_text(d, hint, y, 20, Color::new(255, 220, 100, 255), screen_width);
}
