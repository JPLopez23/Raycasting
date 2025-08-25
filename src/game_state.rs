// game_state.rs
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Clone)]
pub enum GameState {
    MainMenu,
    Playing,
    Victory,
    Defeat, 
}

pub struct GameData {
    pub medical_supplies_collected: u32,
    pub total_medical_supplies: u32,
    pub game_state: GameState,
    pub victory_sound_played: bool,
    pub start_time: Instant,        
    pub time_limit: Duration,       
}

impl GameData {
    pub fn new() -> Self {
        GameData {
            medical_supplies_collected: 0,
            total_medical_supplies: 3,
            game_state: GameState::MainMenu,
            victory_sound_played: false,
            start_time: Instant::now(),
            time_limit: Duration::from_secs(5 * 60), 
        }
    }

    pub fn collect_medical_supply(&mut self) {
        self.medical_supplies_collected += 1;
    }

    pub fn can_escape(&self) -> bool {
        self.medical_supplies_collected >= self.total_medical_supplies
    }

    pub fn reset(&mut self) {
        self.medical_supplies_collected = 0;
        self.game_state = GameState::Playing;
        self.victory_sound_played = false;
        self.start_time = Instant::now(); 
    }

    pub fn time_remaining(&self) -> Duration {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.time_limit { Duration::from_secs(0) } else { self.time_limit - elapsed }
    }
}
