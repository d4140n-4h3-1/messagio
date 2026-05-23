use colored::*;
use once_cell::sync::Lazy;
use std::fs;
use std::io::{self, Write};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use std::env;

pub mod colors;
pub mod sound;

// Re-export Color so callers don't need to import colored directly
pub use colored::Color;

pub static COLOR_ENABLED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));
pub static SOUND_ENABLED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));
pub static SYNC_COLOR_SOUND: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
pub static PULSE_ENABLED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));

// Speed settings
static PULSE_SPEED_MS: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(300));
static BEEP_INTERVAL_MS: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(50));
static SPINNER_SPEED_MS: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(100));

// Cached config so we don't hit the filesystem on every message call.
static CACHED_CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(Config::default()));

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Config {
    pub audio_enabled: bool,
    pub volume_percent: u32,
    pub sync_color_sound: bool,
    pub pulse_with_beeps: bool,
    // Speed settings
    pub pulse_speed_ms: u64,
    pub beep_interval_ms: u64,
    pub spinner_speed_ms: u64,
    // Sound frequencies
    pub success_freq1: f32,
    pub success_freq2: f32,
    pub success_duration1: u64,
    pub success_duration2: u64,
    pub success_beep_count: usize,
    pub error_freq1: f32,
    pub error_freq2: f32,
    pub error_duration1: u64,
    pub error_duration2: u64,
    pub error_beep_count: usize,
    pub warning_freq: f32,
    pub warning_duration: u64,
    pub warning_beep_count: usize,
    pub beep_freq: f32,
    pub beep_duration: u64,
    pub beep_count: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audio_enabled: true,
            volume_percent: 80,
            sync_color_sound: false,
            pulse_with_beeps: true,
            // Speed defaults
            pulse_speed_ms: 300,
            beep_interval_ms: 50,
            spinner_speed_ms: 100,
            // Sound defaults
            success_freq1: 440.0,
            success_freq2: 880.0,
            success_duration1: 200,
            success_duration2: 200,
            success_beep_count: 2,
            error_freq1: 220.0,
            error_freq2: 110.0,
            error_duration1: 300,
            error_duration2: 300,
            error_beep_count: 2,
            warning_freq: 660.0,
            warning_duration: 150,
            warning_beep_count: 2,
            beep_freq: 440.0,
            beep_duration: 150,
            beep_count: 1,
        }
    }
}

impl Config {
    /// Serialize to TOML-compatible string.
    pub fn to_toml(&self) -> String {
        format!(
            "audio_enabled = {}\n\
             volume_percent = {}\n\
             sync_color_sound = {}\n\
             pulse_with_beeps = {}\n\
             pulse_speed_ms = {}\n\
             beep_interval_ms = {}\n\
             spinner_speed_ms = {}\n\
             \n\
             [success]\n\
             freq1 = {}\n\
             freq2 = {}\n\
             duration1 = {}\n\
             duration2 = {}\n\
             beep_count = {}\n\
             \n\
             [error]\n\
             freq1 = {}\n\
             freq2 = {}\n\
             duration1 = {}\n\
             duration2 = {}\n\
             beep_count = {}\n\
             \n\
             [warning]\n\
             freq = {}\n\
             duration = {}\n\
             beep_count = {}\n\
             \n\
             [beep]\n\
             freq = {}\n\
             duration = {}\n\
             beep_count = {}\n",
            self.audio_enabled, self.volume_percent, self.sync_color_sound, self.pulse_with_beeps,
            self.pulse_speed_ms, self.beep_interval_ms, self.spinner_speed_ms,
            self.success_freq1, self.success_freq2, self.success_duration1, self.success_duration2, self.success_beep_count,
            self.error_freq1, self.error_freq2, self.error_duration1, self.error_duration2, self.error_beep_count,
            self.warning_freq, self.warning_duration, self.warning_beep_count,
            self.beep_freq, self.beep_duration, self.beep_count,
        )
    }

    /// Parse from a TOML-compatible string.
    /// Keys are scoped to their `[section]` headers so identically-named
    /// keys in different sections (e.g. `beep_count`, `freq`) don't collide.
    pub fn from_str(s: &str) -> Result<Self, String> {
        let mut config = Config::default();
        let mut section = "";

        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Section header
            if line.starts_with('[') {
                section = match line.trim_matches(|c| c == '[' || c == ']') {
                    "success" => "success",
                    "error"   => "error",
                    "warning" => "warning",
                    "beep"    => "beep",
                    _         => "",
                };
                continue;
            }

            let Some((key, value)) = line.split_once('=') else { continue };
            let key   = key.trim();
            let value = value.trim();

            // Top-level keys (no section)
            if section.is_empty() {
                match key {
                    "audio_enabled"    => config.audio_enabled    = value == "true",
                    "sync_color_sound" => config.sync_color_sound = value == "true",
                    "pulse_with_beeps" => config.pulse_with_beeps = value == "true",
                    "pulse_speed_ms"   => {
                        if let Ok(v) = value.parse::<u64>() {
                            config.pulse_speed_ms = v;
                        }
                    }
                    "beep_interval_ms" => {
                        if let Ok(v) = value.parse::<u64>() {
                            config.beep_interval_ms = v;
                        }
                    }
                    "spinner_speed_ms" => {
                        if let Ok(v) = value.parse::<u64>() {
                            config.spinner_speed_ms = v;
                        }
                    }
                    "volume_percent"   => {
                        if let Ok(v) = value.parse::<u32>() {
                            config.volume_percent = v.clamp(0, 100);
                        }
                    }
                    _ => {}
                }
                continue;
            }

            // Section-scoped keys
            match (section, key) {
                ("success", "freq1")      => { if let Ok(v) = value.parse() { config.success_freq1      = v; } }
                ("success", "freq2")      => { if let Ok(v) = value.parse() { config.success_freq2      = v; } }
                ("success", "duration1")  => { if let Ok(v) = value.parse() { config.success_duration1  = v; } }
                ("success", "duration2")  => { if let Ok(v) = value.parse() { config.success_duration2  = v; } }
                ("success", "beep_count") => { if let Ok(v) = value.parse() { config.success_beep_count = v; } }

                ("error", "freq1")        => { if let Ok(v) = value.parse() { config.error_freq1      = v; } }
                ("error", "freq2")        => { if let Ok(v) = value.parse() { config.error_freq2      = v; } }
                ("error", "duration1")    => { if let Ok(v) = value.parse() { config.error_duration1  = v; } }
                ("error", "duration2")    => { if let Ok(v) = value.parse() { config.error_duration2  = v; } }
                ("error", "beep_count")   => { if let Ok(v) = value.parse() { config.error_beep_count = v; } }

                ("warning", "freq")       => { if let Ok(v) = value.parse() { config.warning_freq         = v; } }
                ("warning", "duration")   => { if let Ok(v) = value.parse() { config.warning_duration     = v; } }
                ("warning", "beep_count") => { if let Ok(v) = value.parse() { config.warning_beep_count   = v; } }

                ("beep", "freq")          => { if let Ok(v) = value.parse() { config.beep_freq     = v; } }
                ("beep", "duration")      => { if let Ok(v) = value.parse() { config.beep_duration = v; } }
                ("beep", "beep_count")    => { if let Ok(v) = value.parse() { config.beep_count    = v; } }

                _ => {}
            }
        }

        Ok(config)
    }
}

// Get the config file path in the project root (same directory as Cargo.toml)
pub fn get_config_path() -> PathBuf {
    let mut current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    loop {
        if current.join("Cargo.toml").exists() {
            return current.join("messagio.toml");
        }
        if !current.pop() {
            break;
        }
    }

    PathBuf::from("messagio.toml")
}

pub fn load_config() -> Config {
    let config_path = get_config_path();

    match fs::read_to_string(&config_path) {
        Ok(contents) => match Config::from_str(&contents) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("  {} Failed to parse config: {}, using defaults", "⚠".yellow(), e);
                Config::default()
            }
        },
        Err(_) => first_run_setup(),
    }
}

pub fn save_config(config: &Config) {
    let config_path = get_config_path();

    match fs::write(&config_path, config.to_toml()) {
        Ok(_) => {
            if cfg!(debug_assertions) {
                println!("  {} Config saved to: {}", "✓".green(), config_path.display());
            }
        }
        Err(e) => {
            eprintln!("  {} Failed to save config to {}: {}", "⚠".yellow(), config_path.display(), e);
        }
    }
}

pub fn apply_config(config: &Config) {
    *SOUND_ENABLED.lock().unwrap()    = config.audio_enabled;
    *SYNC_COLOR_SOUND.lock().unwrap() = config.sync_color_sound;
    *PULSE_ENABLED.lock().unwrap()    = config.pulse_with_beeps;
    *PULSE_SPEED_MS.lock().unwrap()   = config.pulse_speed_ms;
    *BEEP_INTERVAL_MS.lock().unwrap() = config.beep_interval_ms;
    *SPINNER_SPEED_MS.lock().unwrap() = config.spinner_speed_ms;

    // Update the in-memory cache so per-message calls don't touch the filesystem.
    *CACHED_CONFIG.lock().unwrap() = config.clone();

    sound::configure_sounds(config);

    if config.sync_color_sound {
        let sound_state = *SOUND_ENABLED.lock().unwrap();
        *COLOR_ENABLED.lock().unwrap() = sound_state;
    }
}

/// Returns a clone of the cached config.  Avoids filesystem I/O on every
/// message call once `apply_config` has been called.
fn get_config() -> Config {
    CACHED_CONFIG.lock().unwrap().clone()
}

fn prompt_bool(prompt: &str) -> bool {
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no"  => return false,
            _            => println!("  Please enter y or n."),
        }
    }
}

fn prompt_volume(prompt: &str) -> u32 {
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim().parse::<u32>() {
            Ok(v) if v <= 100 => return v,
            _                  => println!("  Please enter a number between 0 and 100."),
        }
    }
}

fn prompt_usize(prompt: &str) -> usize {
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim().parse::<usize>() {
            Ok(v) if v > 0 && v <= 10 => return v,
            _ => println!("  Please enter a number between 1 and 10."),
        }
    }
}

fn first_run_setup() -> Config {
    let config_path = get_config_path();

    println!();
    println!("  {} Welcome to Messagio — first run setup", "🔧".cyan());
    println!();
    println!("  Config will be saved to: {}", config_path.display());
    println!();

    let audio_enabled  = prompt_bool("  Enable audio feedback? [y/n]: ");
    let volume_percent = if audio_enabled { prompt_volume("  Volume (0–100): ") } else { 80 };
    let sync_color_sound = if audio_enabled {
        prompt_bool("  Sync colors with sound? [y/n]: ")
    } else {
        false
    };
    let pulse_with_beeps = prompt_bool("  Make text pulse (normal->bold) with each beep? [y/n]: ");

    println!();
    println!("  Now configure beep counts (1-10 beeps per sound type):");
    let success_beep_count = prompt_usize("  Success beep count: ");
    let error_beep_count   = prompt_usize("  Error beep count: ");
    let warning_beep_count = prompt_usize("  Warning beep count: ");
    println!();

    let config = Config {
        audio_enabled,
        volume_percent,
        sync_color_sound,
        pulse_with_beeps,
        success_beep_count,
        error_beep_count,
        warning_beep_count,
        ..Config::default()
    };
    save_config(&config);

    println!("  {} Preferences saved.", "✓".green());
    println!();

    config
}

// ── Helper functions for pulse ────────────────────────────────────────────────

fn get_beep_count(sound_type: Option<SoundType>) -> usize {
    let config = get_config();
    match sound_type {
        Some(SoundType::Success)      => config.success_beep_count,
        Some(SoundType::Error)        => config.error_beep_count,
        Some(SoundType::Warning)      => config.warning_beep_count,
        Some(SoundType::Notification) => config.warning_beep_count,
        Some(SoundType::Beep)         => config.beep_count,
        None                          => 0,
    }
}

fn make_bold(text: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", text)
}

/// Helper function to colorize text
fn colorize_text(text: &str, color: Color) -> String {
    if *COLOR_ENABLED.lock().unwrap() {
        text.color(color).to_string()
    } else {
        text.to_string()
    }
}

/// Print a message, optionally pulsing bold and playing a sound.
fn synchronized_message<F>(text: &str, beep_count: usize, sound_func: F)
where
    F: FnOnce() -> Result<(), String>,
{
    if beep_count == 0 {
        println!("{}", text);
        return;
    }

    if *PULSE_ENABLED.lock().unwrap() {
        print!("\r\x1b[K{}", make_bold(text));
        io::stdout().flush().unwrap();
        if *SOUND_ENABLED.lock().unwrap() {
            let _ = sound_func();
        }
        println!();
    } else {
        println!("{}", text);
        if *SOUND_ENABLED.lock().unwrap() {
            let _ = sound_func();
        }
    }
}

// ── Speed Helper Functions ────────────────────────────────────────────────────

pub fn get_pulse_speed_ms() -> u64 {
    *PULSE_SPEED_MS.lock().unwrap()
}

pub fn get_beep_interval_ms() -> u64 {
    *BEEP_INTERVAL_MS.lock().unwrap()
}

pub fn get_spinner_speed_ms() -> u64 {
    *SPINNER_SPEED_MS.lock().unwrap()
}

pub fn set_pulse_speed_ms(speed_ms: u64) {
    *PULSE_SPEED_MS.lock().unwrap() = speed_ms;
    let mut config = get_config();
    config.pulse_speed_ms = speed_ms;
    save_config(&config);
}

pub fn set_beep_interval_ms(interval_ms: u64) {
    *BEEP_INTERVAL_MS.lock().unwrap() = interval_ms;
    let mut config = get_config();
    config.beep_interval_ms = interval_ms;
    save_config(&config);
}

pub fn set_spinner_speed_ms(speed_ms: u64) {
    *SPINNER_SPEED_MS.lock().unwrap() = speed_ms;
    let mut config = get_config();
    config.spinner_speed_ms = speed_ms;
    save_config(&config);
}

// ── Pulse Functions ──────────────────────────────────────────────────────────

/// Display a message that pulses (alternates between normal and bold) with musical beeps
pub fn pulse_musical(text: &str, color: Color, times: usize) {
    let beep_count = get_beep_count(Some(SoundType::Success));
    let pulse_speed = get_pulse_speed_ms();
    
    for i in 0..times {
        if *PULSE_ENABLED.lock().unwrap() {
            print!("\r\x1b[K{}", make_bold(&colorize_text(text, color)));
        } else {
            print!("\r\x1b[K{}", colorize_text(text, color));
        }
        io::stdout().flush().unwrap();
        
        if *SOUND_ENABLED.lock().unwrap() && beep_count > 0 {
            let _ = sound::play_success();
        }
        
        if i < times - 1 {
            thread::sleep(Duration::from_millis(pulse_speed));
            
            if *PULSE_ENABLED.lock().unwrap() {
                print!("\r\x1b[K{}", colorize_text(text, color));
            } else {
                print!("\r\x1b[K{}", colorize_text(text, color));
            }
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(pulse_speed));
        }
    }
    println!();
}

/// Display a message that pulses gently with warning sounds
pub fn pulse_gentle(text: &str, color: Color, times: usize) {
    let beep_count = get_beep_count(Some(SoundType::Warning));
    let pulse_speed = get_pulse_speed_ms();
    
    for i in 0..times {
        if *PULSE_ENABLED.lock().unwrap() {
            print!("\r\x1b[K{}", make_bold(&colorize_text(text, color)));
        } else {
            print!("\r\x1b[K{}", colorize_text(text, color));
        }
        io::stdout().flush().unwrap();
        
        if *SOUND_ENABLED.lock().unwrap() && beep_count > 0 {
            let _ = sound::play_warning();
        }
        
        if i < times - 1 {
            thread::sleep(Duration::from_millis(pulse_speed));
            
            if *PULSE_ENABLED.lock().unwrap() {
                print!("\r\x1b[K{}", colorize_text(text, color));
            } else {
                print!("\r\x1b[K{}", colorize_text(text, color));
            }
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(pulse_speed));
        }
    }
    println!();
}

/// Display a message that pulses with error sounds (dissonant)
pub fn pulse_with_error_sound(text: &str, color: Color, times: usize) {
    let beep_count = get_beep_count(Some(SoundType::Error));
    let pulse_speed = get_pulse_speed_ms();
    
    for i in 0..times {
        if *PULSE_ENABLED.lock().unwrap() {
            print!("\r\x1b[K{}", make_bold(&colorize_text(text, color)));
        } else {
            print!("\r\x1b[K{}", colorize_text(text, color));
        }
        io::stdout().flush().unwrap();
        
        if *SOUND_ENABLED.lock().unwrap() && beep_count > 0 {
            let _ = sound::play_error();
        }
        
        if i < times - 1 {
            thread::sleep(Duration::from_millis(pulse_speed));
            
            if *PULSE_ENABLED.lock().unwrap() {
                print!("\r\x1b[K{}", colorize_text(text, color));
            } else {
                print!("\r\x1b[K{}", colorize_text(text, color));
            }
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(pulse_speed));
        }
    }
    println!();
}

/// Display a message that pulses with custom sound configuration
pub fn pulse_custom(text: &str, color: Color, sound_type: SoundType, times: usize) {
    let beep_count = get_beep_count(Some(sound_type));
    let pulse_speed = get_pulse_speed_ms();
    
    for i in 0..times {
        if *PULSE_ENABLED.lock().unwrap() {
            print!("\r\x1b[K{}", make_bold(&colorize_text(text, color)));
        } else {
            print!("\r\x1b[K{}", colorize_text(text, color));
        }
        io::stdout().flush().unwrap();
        
        if *SOUND_ENABLED.lock().unwrap() && beep_count > 0 {
            match sound_type {
                SoundType::Success => { let _ = sound::play_success(); }
                SoundType::Error => { let _ = sound::play_error(); }
                SoundType::Warning | SoundType::Notification => { let _ = sound::play_warning(); }
                SoundType::Beep => {
                    let config = get_config();
                    let _ = sound::play_beep_pub(config.beep_freq, config.beep_duration);
                }
            }
        }
        
        if i < times - 1 {
            thread::sleep(Duration::from_millis(pulse_speed));
            
            if *PULSE_ENABLED.lock().unwrap() {
                print!("\r\x1b[K{}", colorize_text(text, color));
            } else {
                print!("\r\x1b[K{}", colorize_text(text, color));
            }
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(pulse_speed));
        }
    }
    println!();
}

// ── Color / sound toggles with sync support ───────────────────────────────────

pub fn sync_color_with_sound(enable: bool) {
    *SYNC_COLOR_SOUND.lock().unwrap() = enable;
    if enable {
        let sound_state = *SOUND_ENABLED.lock().unwrap();
        *COLOR_ENABLED.lock().unwrap() = sound_state;
    }
}

pub fn enable_all()     { enable_colors();  enable_sound();  }
pub fn disable_all()    { disable_colors(); disable_sound(); }

pub fn enable_colors() {
    *COLOR_ENABLED.lock().unwrap() = true;
    if *SYNC_COLOR_SOUND.lock().unwrap() { *SOUND_ENABLED.lock().unwrap() = true; }
}

pub fn disable_colors() {
    *COLOR_ENABLED.lock().unwrap() = false;
    if *SYNC_COLOR_SOUND.lock().unwrap() { *SOUND_ENABLED.lock().unwrap() = false; }
}

pub fn enable_sound() {
    *SOUND_ENABLED.lock().unwrap() = true;
    if *SYNC_COLOR_SOUND.lock().unwrap() { *COLOR_ENABLED.lock().unwrap() = true; }
}

pub fn disable_sound() {
    *SOUND_ENABLED.lock().unwrap() = false;
    if *SYNC_COLOR_SOUND.lock().unwrap() { *COLOR_ENABLED.lock().unwrap() = false; }
}

pub fn enable_pulse()  { *PULSE_ENABLED.lock().unwrap() = true;  }
pub fn disable_pulse() { *PULSE_ENABLED.lock().unwrap() = false; }

// ── Sound type enum ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundType {
    Success,
    Error,
    Warning,
    Notification,
    Beep,
}

// ── Standalone sound helpers ──────────────────────────────────────────────────

pub fn play_success_sound() -> Result<(), String> { sound::play_success() }
pub fn play_error_sound()   -> Result<(), String> { sound::play_error()   }
pub fn play_warning_sound() -> Result<(), String> { sound::play_warning() }

// ── Core message functions ───────────────────────────────────────────────────

pub fn success<M: AsRef<str>>(msg: M) {
    let config     = get_config();
    let beep_count = config.success_beep_count;
    let is_colored = *COLOR_ENABLED.lock().unwrap();
    let text = if is_colored {
        format!("{} {}", colors::check(), msg.as_ref().green())
    } else {
        format!("[✓] {}", msg.as_ref())
    };
    synchronized_message(&text, beep_count, || sound::play_success());
}

pub fn error<M: AsRef<str>>(msg: M) {
    let config     = get_config();
    let beep_count = config.error_beep_count;
    let is_colored = *COLOR_ENABLED.lock().unwrap();
    let text = if is_colored {
        format!("{} {}", colors::cross(), msg.as_ref().red().bold())
    } else {
        format!("[✗] {}", msg.as_ref())
    };
    synchronized_message(&text, beep_count, || sound::play_error());
}

/// Alias for [`warn`].
pub fn warning<M: AsRef<str>>(msg: M) { warn(msg); }

pub fn warn<M: AsRef<str>>(msg: M) {
    let config     = get_config();
    let beep_count = config.warning_beep_count;
    let is_colored = *COLOR_ENABLED.lock().unwrap();
    let text = if is_colored {
        format!("{} {}", colors::warn(), msg.as_ref().yellow())
    } else {
        format!("[⚠] {}", msg.as_ref())
    };
    synchronized_message(&text, beep_count, || sound::play_warning());
}

pub fn info<M: AsRef<str>>(msg: M) -> String {
    if *COLOR_ENABLED.lock().unwrap() {
        format!("{} {}", colors::info(), msg.as_ref().blue())
    } else {
        format!("[i] {}", msg.as_ref())
    }
}

pub fn critical<M: AsRef<str>>(msg: M) {
    let config     = get_config();
    let beep_count = config.error_beep_count;
    let is_colored = *COLOR_ENABLED.lock().unwrap();
    let text = if is_colored {
        format!("{} {}", colors::cross(), msg.as_ref().red().bold().on_black())
    } else {
        format!("[✗] CRITICAL: {}", msg.as_ref())
    };
    synchronized_message(&text, beep_count, || sound::play_error());
}

pub fn colored<M: AsRef<str>>(msg: M, color: Color) -> String {
    if *COLOR_ENABLED.lock().unwrap() {
        msg.as_ref().color(color).bold().to_string()
    } else {
        msg.as_ref().to_string()
    }
}

// ── Sound-prefixed aliases ────────────────────────────────────────────────────

/// Alias for [`success`] — prints and plays the success sound.
pub fn sound_success<M: AsRef<str>>(msg: M) { success(msg); }

/// Alias for [`error`] — prints and plays the error sound.
pub fn sound_error<M: AsRef<str>>(msg: M) { error(msg); }

// ── Status indicator strings ─────────────────────────────────────────────────

pub fn status_valid() -> String { colors::valid_prefix() }
pub fn status_warn()  -> String { colors::warn_prefix()  }
pub fn status_error() -> String { colors::error_prefix() }

// ── Progress / spinner ────────────────────────────────────────────────────────

/// Global running flag used only by the standalone `progress` / `progress_complete` helpers.
static GLOBAL_SPINNER_RUNNING: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub fn progress<M: AsRef<str>>(msg: M) {
    *GLOBAL_SPINNER_RUNNING.lock().unwrap() = true;
    let message = msg.as_ref().to_string();
    let colors_enabled = *COLOR_ENABLED.lock().unwrap();
    let spinner_speed = get_spinner_speed_ms();

    thread::spawn(move || {
        let frames = ["◐", "◓", "◑", "◒"];
        let mut i = 0;
        while *GLOBAL_SPINNER_RUNNING.lock().unwrap() {
            if colors_enabled {
                print!("\r{} {}", frames[i].bold().cyan(), message);
            } else {
                print!("\r{} {}", frames[i], message);
            }
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            thread::sleep(Duration::from_millis(spinner_speed));
            i = (i + 1) % frames.len();
        }
    });
}

pub fn progress_complete<M: AsRef<str>>(msg: M) {
    *GLOBAL_SPINNER_RUNNING.lock().unwrap() = false;
    thread::sleep(Duration::from_millis(120));
    if *COLOR_ENABLED.lock().unwrap() {
        println!("\r{} {}", colors::check(), msg.as_ref().green());
    } else {
        println!("\r[✓] {}", msg.as_ref());
    }
}

pub fn progress_fail<M: AsRef<str>>(msg: M) {
    *GLOBAL_SPINNER_RUNNING.lock().unwrap() = false;
    thread::sleep(Duration::from_millis(120));
    if *COLOR_ENABLED.lock().unwrap() {
        println!("\r{} {}", colors::cross(), msg.as_ref().red());
    } else {
        println!("\r[✗] {}", msg.as_ref());
    }
}

pub fn spinner<M: AsRef<str>>(msg: M) -> SpinnerHandle {
    SpinnerHandle::new(msg)
}

/// A spinner handle with its own per-instance running flag, so multiple
/// concurrent spinners don't interfere with each other.
pub struct SpinnerHandle {
    running: std::sync::Arc<Mutex<bool>>,
}

impl SpinnerHandle {
    fn new<M: AsRef<str>>(msg: M) -> Self {
        let running       = std::sync::Arc::new(Mutex::new(true));
        let running_clone = running.clone();
        let message       = msg.as_ref().to_string();
        let colors_enabled = *COLOR_ENABLED.lock().unwrap();
        let spinner_speed = get_spinner_speed_ms();

        thread::spawn(move || {
            let frames = ["◐", "◓", "◑", "◒"];
            let mut i = 0;
            while *running_clone.lock().unwrap() {
                if colors_enabled {
                    print!("\r{} {}", frames[i].bold().cyan(), message);
                } else {
                    print!("\r{} {}", frames[i], message);
                }
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                thread::sleep(Duration::from_millis(spinner_speed));
                i = (i + 1) % frames.len();
            }
        });

        SpinnerHandle { running }
    }

    pub fn finish_success<M: AsRef<str>>(&self, msg: M) {
        *self.running.lock().unwrap() = false;
        thread::sleep(Duration::from_millis(120));
        if *COLOR_ENABLED.lock().unwrap() {
            println!("\r{} {}", colors::check(), msg.as_ref().green());
        } else {
            println!("\r[✓] {}", msg.as_ref());
        }
    }

    pub fn finish_error<M: AsRef<str>>(&self, msg: M) {
        *self.running.lock().unwrap() = false;
        thread::sleep(Duration::from_millis(120));
        if *COLOR_ENABLED.lock().unwrap() {
            println!("\r{} {}", colors::cross(), msg.as_ref().red());
        } else {
            println!("\r[✗] {}", msg.as_ref());
        }
    }
}

// ── MessageBuilder ────────────────────────────────────────────────────────────

pub fn message<M: AsRef<str>>(msg: M) -> MessageBuilder {
    MessageBuilder::new(msg)
}

pub struct MessageBuilder {
    text: String,
    color: Option<Color>,
    symbol: Option<String>,
    sound: Option<SoundType>,
    force_pulse: bool,
}

impl MessageBuilder {
    pub fn new<M: AsRef<str>>(msg: M) -> Self {
        MessageBuilder {
            text: msg.as_ref().to_string(),
            color: None,
            symbol: None,
            sound: None,
            force_pulse: false,
        }
    }

    pub fn color(mut self, c: Color) -> Self { self.color = Some(c); self }

    pub fn with_symbol<S: AsRef<str>>(mut self, sym: S) -> Self {
        self.symbol = Some(sym.as_ref().to_string());
        self
    }

    pub fn with_sound(mut self, s: SoundType) -> Self { self.sound = Some(s); self }

    /// Force the text to pulse bold even on single-beep sounds.
    pub fn blinking(mut self) -> Self { self.force_pulse = true; self }

    pub fn build(&self) -> String {
        let colors_enabled = *COLOR_ENABLED.lock().unwrap();
        let mut out = self.text.clone();

        if colors_enabled {
            if let Some(c) = self.color {
                out = out.color(c).to_string();
            }
        }

        if let Some(ref sym) = self.symbol {
            out = format!("{} {}", sym, out);
        }

        out
    }

    pub fn send(&self) {
        let beep_count   = get_beep_count(self.sound);
        let should_pulse = self.force_pulse || (beep_count > 0 && *PULSE_ENABLED.lock().unwrap());
        let output       = self.build();

        let sound_func: Box<dyn FnOnce() -> Result<(), String>> = match self.sound {
            Some(SoundType::Success) => Box::new(|| sound::play_success()),
            Some(SoundType::Error)   => Box::new(|| sound::play_error()),
            Some(SoundType::Warning) | Some(SoundType::Notification) => Box::new(|| sound::play_warning()),
            Some(SoundType::Beep) => {
                let config   = get_config();
                let freq     = config.beep_freq;
                let duration = config.beep_duration;
                Box::new(move || sound::play_beep_pub(freq, duration))
            }
            None => Box::new(|| Ok(())),
        };

        if should_pulse && beep_count > 0 {
            print!("\r\x1b[K{}", make_bold(&output));
            io::stdout().flush().unwrap();
            if *SOUND_ENABLED.lock().unwrap() { let _ = sound_func(); }
            println!();
        } else {
            println!("{}", output);
            if *SOUND_ENABLED.lock().unwrap() { let _ = sound_func(); }
        }
    }

    pub fn text<S: ToString>(mut self, t: S) -> Self {
        self.text = format!("{} {}", self.text, t.to_string());
        self
    }

    pub fn colored_text<S: ToString>(mut self, t: S, c: Color) -> Self {
        let colors_enabled = *COLOR_ENABLED.lock().unwrap();
        let fragment = if colors_enabled {
            t.to_string().color(c).to_string()
        } else {
            t.to_string()
        };
        self.text = format!("{} {}", self.text, fragment);
        self
    }
}

impl Default for MessageBuilder {
    fn default() -> Self { Self::new("") }
}

// ── Compatibility modules for backward compatibility ─────────────────────────

pub mod color_utils {
    pub fn info() -> String { crate::info("") }
    pub fn status_valid() -> String { crate::status_valid() }
    pub fn status_warn() -> String { crate::status_warn() }
    pub fn status_error() -> String { crate::status_error() }
}

pub mod audio_utils {
    pub use crate::{pulse_musical, pulse_gentle, pulse_with_error_sound, pulse_custom};
}

// ── Macros ────────────────────────────────────────────────────────────────────

#[macro_export]
macro_rules! color_println {
    ($color:ident, $($arg:tt)*) => {{
        use colored::Colorize;
        let msg = format!($($arg)*);
        if *$crate::COLOR_ENABLED.lock().unwrap() {
            println!("{}", msg.color(colored::Color::$color));
        } else {
            println!("{}", msg);
        }
    }};
}

#[macro_export]
macro_rules! color_print {
    ($color:ident, $($arg:tt)*) => {{
        use colored::Colorize;
        let msg = format!($($arg)*);
        if *$crate::COLOR_ENABLED.lock().unwrap() {
            print!("{}", msg.color(colored::Color::$color));
        } else {
            print!("{}", msg);
        }
    }};
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_builder_legacy() {
        let msg = MessageBuilder::new("Hello")
            .colored_text("World", Color::Green)
            .build();
        assert!(msg.contains("Hello"));
        assert!(msg.contains("World"));
    }

    #[test]
    fn test_builder_chaining() {
        let b   = message("Test").color(Color::Red).with_symbol("!").with_sound(SoundType::Beep);
        let out = b.build();
        assert!(out.contains("Test"));
        assert!(out.contains("!"));
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.audio_enabled);
        assert_eq!(config.volume_percent, 80);
        assert!(!config.sync_color_sound);
        assert!(config.pulse_with_beeps);
        assert_eq!(config.success_beep_count, 2);
        assert_eq!(config.error_beep_count,   2);
        assert_eq!(config.warning_beep_count, 2);
        assert_eq!(config.pulse_speed_ms, 300);
        assert_eq!(config.beep_interval_ms, 50);
        assert_eq!(config.spinner_speed_ms, 100);
    }

    #[test]
    fn test_config_roundtrip() {
        let original = Config::default();
        let parsed   = Config::from_str(&original.to_toml()).expect("parse failed");

        assert_eq!(parsed.audio_enabled,       original.audio_enabled);
        assert_eq!(parsed.success_beep_count,  original.success_beep_count);
        assert_eq!(parsed.error_beep_count,    original.error_beep_count);
        assert_eq!(parsed.warning_beep_count,  original.warning_beep_count);
        assert_eq!(parsed.beep_count,          original.beep_count);
        assert_eq!(parsed.pulse_speed_ms,      original.pulse_speed_ms);
        assert_eq!(parsed.beep_interval_ms,    original.beep_interval_ms);
        assert_eq!(parsed.spinner_speed_ms,    original.spinner_speed_ms);
        assert!((parsed.success_freq1 - original.success_freq1).abs() < 0.01);
        assert!((parsed.error_freq1   - original.error_freq1  ).abs() < 0.01);
        assert!((parsed.warning_freq  - original.warning_freq ).abs() < 0.01);
        assert!((parsed.beep_freq     - original.beep_freq    ).abs() < 0.01);
    }

    #[test]
    fn test_config_section_isolation() {
        // Verifies that beep_count in [error] doesn't clobber [success], etc.
        let toml = "\
audio_enabled = true
volume_percent = 80
sync_color_sound = false
pulse_with_beeps = true
pulse_speed_ms = 200
beep_interval_ms = 30
spinner_speed_ms = 50

[success]
freq1 = 440
freq2 = 880
duration1 = 200
duration2 = 200
beep_count = 2

[error]
freq1 = 220
freq2 = 110
duration1 = 300
duration2 = 300
beep_count = 4

[warning]
freq = 660
duration = 150
beep_count = 3

[beep]
freq = 440
duration = 150
beep_count = 1
";
        let cfg = Config::from_str(toml).unwrap();
        assert_eq!(cfg.success_beep_count, 2);
        assert_eq!(cfg.error_beep_count,   4);
        assert_eq!(cfg.warning_beep_count, 3);
        assert_eq!(cfg.beep_count,         1);
        assert_eq!(cfg.pulse_speed_ms,     200);
        assert_eq!(cfg.beep_interval_ms,   30);
        assert_eq!(cfg.spinner_speed_ms,   50);
        assert!((cfg.success_freq1 - 440.0).abs() < 0.01);
        assert!((cfg.error_freq1   - 220.0).abs() < 0.01);
        assert!((cfg.warning_freq  - 660.0).abs() < 0.01);
    }
}