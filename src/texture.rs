// texture.rs 
use raylib::prelude::*;
use std::collections::HashMap;

pub struct TextureManager {
    textures: HashMap<char, Texture2D>,
    texture_size: usize,
}

impl TextureManager {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let mut manager = TextureManager {
            textures: HashMap::new(),
            texture_size: 64,
        };
        manager.load_textures(rl, thread);
        manager
    }

    fn try_load_texture_any(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        paths: &[&str],
        label: &str,
    ) -> Option<Texture2D> {
        for p in paths {
            if let Ok(t) = rl.load_texture(thread, p) {
                println!("[textures] Loaded {label} via load_texture: {p}");
                return Some(t);
            }
        }
        for p in paths {
            if let Ok(img) = Image::load_image(p) {
                println!("[textures] Loaded {label} as Image: {p}, creating Texture2D");
                if let Ok(t) = rl.load_texture_from_image(thread, &img) {
                    return Some(t);
                }
            }
        }
        println!("[textures] Could not load {label} from {:?}", paths);
        None
    }
    
    fn load_textures(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        fn try_load(rl: &mut RaylibHandle, thread: &RaylibThread, paths: &[&str], label: &str)
            -> Option<Texture2D>
        {
            for p in paths {
                match rl.load_texture(thread, p) {
                    Ok(t) => { println!("[textures] Loaded {label} from: {p}"); return Some(t); }
                    Err(_) => { /* sigue probando */ }
                }
            }
            println!("[textures] PNG not found for {label}, using procedural fallback");
            None
        }

        //  WALLS ('#', '+', '-', '|') 
        if let Some(tex) = try_load(rl, thread, &[
            "textures/wall_horizontal.png",
            "wall_horizontal.png",
        ], "wall") {
            self.textures.insert('#', tex);
            if let Some(t) = try_load(rl, thread, &["textures/wall_horizontal.png","wall_horizontal.png"], "wall") { self.textures.insert('+', t); }
            if let Some(t) = try_load(rl, thread, &["textures/wall_horizontal.png","wall_horizontal.png"], "wall") { self.textures.insert('-', t); }
            if let Some(t) = try_load(rl, thread, &["textures/wall_horizontal.png","wall_horizontal.png"], "wall") { self.textures.insert('|', t); }
        } else {
            let img = Self::create_wall_horizontal_image(self.texture_size);
            if let Ok(t) = rl.load_texture_from_image(thread, &img) { self.textures.insert('#', t); }
            let img2 = Self::create_wall_horizontal_image(self.texture_size);
            if let Ok(t) = rl.load_texture_from_image(thread, &img2) { self.textures.insert('+', t); }
            let img3 = Self::create_wall_horizontal_image(self.texture_size);
            if let Ok(t) = rl.load_texture_from_image(thread, &img3) { self.textures.insert('-', t); }
            let img4 = Self::create_wall_horizontal_image(self.texture_size);
            if let Ok(t) = rl.load_texture_from_image(thread, &img4) { self.textures.insert('|', t); }
        }

        //  MEDICAL ('M') 
        if let Some(tex) = try_load(rl, thread, &[
            "textures/medical_supply.png",
            "medical_supply.png",
        ], "medical") {
            self.textures.insert('M', tex);
        } else {
            let img = Self::create_medical_supply_image(self.texture_size);
            if let Ok(t) = rl.load_texture_from_image(thread, &img) { self.textures.insert('M', t); }
        }

        // ZOMBIE ('Z') 
        if let Some(tex) = Self::try_load_texture_any(
            rl, thread,
            &["textures/zombie.png", "zombie.png"],
            "zombie",
        ) {
            self.textures.insert('Z', tex);
        } else {
            let mut img = Image::gen_image_color(self.texture_size as i32, self.texture_size as i32, Color::new(20,20,20,0));
            for y in 8..(self.texture_size-8) {
                for x in 8..(self.texture_size-8) {
                    img.draw_pixel(x as i32, y as i32, Color::new(120, 200, 120, 255));
                }
            }
            if let Ok(t) = rl.load_texture_from_image(thread, &img) {
                self.textures.insert('Z', t);
            }
        }

        //  EXIT ('g') 
        if let Some(tex) = try_load(rl, thread, &[
            "textures/exit_door.png",
            "exit_door.png",
        ], "exit") {
            self.textures.insert('g', tex);
        } else {
            let img = Self::create_exit_door_image(self.texture_size);
            if let Ok(t) = rl.load_texture_from_image(thread, &img) { self.textures.insert('g', t); }
        }

        //  FLOOR ('F') 
        if let Some(tex) = try_load(rl, thread, &[
            "textures/floor.png",
            "floor.png",
        ], "floor") {
            self.textures.insert('F', tex);
        } else {
            let img = Self::create_floor_image(self.texture_size);
            if let Ok(t) = rl.load_texture_from_image(thread, &img) { self.textures.insert('F', t); }
        }

        //  CEILING ('C') 
        if let Some(tex) = Self::try_load_texture_any(
            rl, thread,
            &["textures/ceiling.png", "ceiling.png"],
            "ceiling",
        ) {
            self.textures.insert('C', tex);
        } else {
            let img = Self::create_ceiling_image(self.texture_size);
            if let Ok(t) = rl.load_texture_from_image(thread, &img) { self.textures.insert('C', t); }
        }
    }

    //  Procedurales (fallback) 
    fn create_wall_horizontal_image(texture_size: usize) -> Image {
        let mut image = Image::gen_image_color(texture_size as i32, texture_size as i32, Color::LIGHTGRAY);
        for y in 0..texture_size {
            for x in 0..texture_size {
                let mut color = Color::new(200, 195, 190, 255);
                if y % 8 == 0 { color = Color::new(180, 175, 170, 255); }
                let brick_offset = if (y / 8) % 2 == 0 { 0 } else { 16 };
                if (x + brick_offset) % 32 == 0 { color = Color::new(180, 175, 170, 255); }
                image.draw_pixel(x as i32, y as i32, color);
            }
        }
        image
    }

    fn create_medical_supply_image(texture_size: usize) -> Image {
        let mut image = Image::gen_image_color(texture_size as i32, texture_size as i32, Color::WHITE);
        for y in 0..texture_size {
            for x in 0..texture_size {
                let mut color = Color::new(250, 250, 255, 255);
                if x < 3 || x >= texture_size - 3 || y < 3 || y >= texture_size - 3 {
                    color = Color::new(180, 180, 180, 255);
                }
                let cx = texture_size / 2;
                let cy = texture_size / 2;
                if y > cy - 6 && y < cy + 6 && x > 10 && x < texture_size - 10 { color = Color::new(220, 20, 20, 255); }
                if x > cx - 6 && x < cx + 6 && y > 10 && y < texture_size - 10 { color = Color::new(220, 20, 20, 255); }
                image.draw_pixel(x as i32, y as i32, color);
            }
        }
        image
    }

    fn create_exit_door_image(texture_size: usize) -> Image {
        let mut image = Image::gen_image_color(texture_size as i32, texture_size as i32, Color::GREEN);
        for y in 0..texture_size {
            for x in 0..texture_size {
                let mut color = Color::new(20, 120, 40, 255);
                if x < 4 || x >= texture_size - 4 || y < 4 || y >= texture_size - 4 {
                    color = Color::new(100, 100, 100, 255);
                }
                if x > 16 && x < 48 && y > 18 && y < 46 {
                    color = Color::new(0, 180, 0, 255);
                    if y > 22 && y < 42 {
                        if (x > 20 && x < 24) || (x > 26 && x < 30) ||
                           (x > 32 && x < 36) || (x > 38 && x < 44) {
                            color = Color::WHITE;
                        }
                    }
                }
                image.draw_pixel(x as i32, y as i32, color);
            }
        }
        image
    }

    fn create_floor_image(texture_size: usize) -> Image {
        let mut image = Image::gen_image_color(texture_size as i32, texture_size as i32, Color::LIGHTGRAY);
        for y in 0..texture_size {
            for x in 0..texture_size {
                let tile = 16;
                let tile_x = x / tile;
                let tile_y = y / tile;
                let mut color = if (tile_x + tile_y) % 2 == 0 {
                    Color::new(180, 180, 185, 255)
                } else {
                    Color::new(170, 170, 175, 255)
                };
                if x % tile < 2 || y % tile < 2 { color = Color::new(150, 150, 155, 255); }
                image.draw_pixel(x as i32, y as i32, color);
            }
        }
        image
    }

    fn create_ceiling_image(texture_size: usize) -> Image {
        let mut image = Image::gen_image_color(texture_size as i32, texture_size as i32, Color::new(230, 230, 240, 255));
        for y in 0..texture_size {
            for x in 0..texture_size {
                let panel = 32;
                let mut color = Color::new(230, 230, 240, 255);
                if x % panel < 2 || y % panel < 2 { color = Color::new(200, 200, 210, 255); }
                image.draw_pixel(x as i32, y as i32, color);
            }
        }
        image
    }

    //  Getters 
    pub fn get_wall_texture(&self, wall_type: char) -> Option<&Texture2D> {
        self.textures.get(&wall_type).or_else(|| self.textures.get(&'#'))
    }
    pub fn get_floor_texture(&self) -> Option<&Texture2D> { self.textures.get(&'F') }
    pub fn get_ceiling_texture(&self) -> Option<&Texture2D> { self.textures.get(&'C') }
    pub fn get_exit_texture(&self) -> Option<&Texture2D> { self.textures.get(&'g') }
    pub fn get_medical_texture(&self) -> Option<&Texture2D> { self.textures.get(&'M') }
    pub fn get_texture_size(&self) -> usize { self.texture_size }
    pub fn get_zombie_texture(&self) -> Option<&Texture2D> { self.textures.get(&'Z') }

}
