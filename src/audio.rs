// audio.rs 
use raylib::prelude::*; 
use raylib::ffi as rlffi;
use std::ffi::CString;

pub struct AudioManager {
    audio_ready: bool,
    music: Option<rlffi::Music>,      
    pickup: Option<rlffi::Sound>,     
    victory: Option<rlffi::Sound>,    
    ui_jingle: Option<rlffi::Sound>,  
    muted: bool,
}

impl AudioManager {
    pub fn new(_rl: &mut RaylibHandle, _thread: &RaylibThread) -> Result<Self, String> {
        unsafe { rlffi::InitAudioDevice(); }
        let ready = unsafe { rlffi::IsAudioDeviceReady() };
        if !ready {
            return Err("Audio device not ready".into());
        }

        let music = unsafe {
            let cpath = CString::new("sounds/background_music.mp3").unwrap();
            let m = rlffi::LoadMusicStream(cpath.as_ptr());
            if m.ctxData.is_null() {
                eprintln!("[audio] Could not load sounds/background_music.mp3");
                None
            } else {
                rlffi::SetMusicVolume(m, 0.65);
                Some(m)
            }
        };

        let pickup = unsafe {
            let cpath = CString::new("sounds/medical_pickup.wav").unwrap();
            let s = rlffi::LoadSound(cpath.as_ptr());
            if s.frameCount == 0 {
                eprintln!("[audio] Could not load sounds/medical_pickup.wav");
                None
            } else {
                Some(s)
            }
        };

        let victory = unsafe {
            let cpath = CString::new("sounds/victory_sound.wav").unwrap();
            let s = rlffi::LoadSound(cpath.as_ptr());
            if s.frameCount == 0 {
                eprintln!("[audio] Could not load sounds/victory_sound.wav");
                None
            } else {
                Some(s)
            }
        };

        let ui_jingle = unsafe {
            let candidate_paths = ["sounds/hola.wav", "hola.wav"];
            let mut loaded: Option<rlffi::Sound> = None;

            for path in candidate_paths.iter() {
                if let Ok(cpath) = CString::new(*path) {
                    let s = rlffi::LoadSound(cpath.as_ptr());
                    if s.frameCount > 0 && s.stream.sampleRate > 0 {
                        println!("[audio] Loaded UI jingle from {}", path);
                        loaded = Some(s);
                        break;
                    } else {
                        rlffi::UnloadSound(s);
                    }
                }
            }
            if loaded.is_none() {
                eprintln!("[audio] Could not load hola.wav from any known path");
            }
            loaded
        };

        Ok(Self {
            audio_ready: true,
            music,
            pickup,
            victory,
            ui_jingle,   
            muted: false,
        })
    }

    pub fn update(&mut self, _rl: &mut RaylibHandle) {
        if !self.audio_ready { return; }
        if let Some(m) = &mut self.music {
            unsafe { rlffi::UpdateMusicStream(*m); }
        }
    }

    pub fn play_background_music(&mut self, _rl: &mut RaylibHandle) {
        if !self.audio_ready || self.muted { return; }
        if let Some(m) = &mut self.music {
            unsafe { 
                rlffi::PlayMusicStream(*m); 
                rlffi::SetMusicVolume(*m, 0.65);
            }
            println!("Background music started");
        }
    }

    pub fn stop_background_music(&mut self, _rl: &mut RaylibHandle) {
        if !self.audio_ready { return; }
        if let Some(m) = &mut self.music {
            unsafe { rlffi::StopMusicStream(*m); }
            println!("Background music stopped");
        }
    }

    pub fn play_medical_pickup(&mut self, _rl: &mut RaylibHandle) {
        if !self.audio_ready || self.muted { return; }
        if let Some(s) = &self.pickup {
            unsafe { rlffi::PlaySound(*s); }
            println!("Medical pickup sound played");
        }
    }

    pub fn play_victory_sound(&mut self, _rl: &mut RaylibHandle) {
        if !self.audio_ready || self.muted { return; }
        if let Some(s) = &self.victory {
            unsafe { rlffi::PlaySound(*s); }
            println!("Victory sound played");
        }
    }

    pub fn play_hola_sound(&mut self, _rl: &mut RaylibHandle) {
        if !self.audio_ready || self.muted { return; }
        if let Some(s) = &self.ui_jingle {
            unsafe { rlffi::PlaySound(*s); }
            println!("Hola sound played");
        }
    }

    pub fn is_music_playing(&self) -> bool {
        if !self.audio_ready { return false; }
        if let Some(m) = &self.music {
            unsafe { rlffi::IsMusicStreamPlaying(*m) }
        } else {
            false
        }
    }
}