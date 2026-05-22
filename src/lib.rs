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

// ── Config ────────────────────────────────────────────────────────────────────

pub struct Config {
    pub audio_enabled: bool,
    pub volume_percent: u32,
    pub sync_color_sound: bool,
    pub pulse_with_beeps: bool,
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

impl Config {
    pub fn default() -> Self {
        Self {
            audio_enabled: true,
            volume_percent: 80,
            sync_color_sound: false,
            pulse_with_beeps: true,
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

    pub fn to_string(&self) -> String {
        format!(
            "audio_enabled = {}\n\
             volume_percent = {}\n\
             sync_color_sound = {}\n\
             pulse_with_beeps = {}\n\
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
            self.audio_enabled,
            self.volume_percent,
            self.sync_color_sound,
            self.pulse_with_beeps,
            self.success_freq1,
            self.success_freq2,
            self.success_duration1,
            self.success_duration2,
            self.success_beep_count,
            self.error_freq1,
            self.error_freq2,
            self.error_duration1,
            self.error_duration2,
            self.error_beep_count,
            self.warning_freq,
            self.warning_duration,
            self.warning_beep_count,
            self.beep_freq,
            self.beep_duration,
            self.beep_count,
        )
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        let mut config = Config::default();
        
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('[') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                match key {
                    "audio_enabled" => config.audio_enabled = value == "true",
                    "volume_percent" => {
                        if let Ok(v) = value.parse::<u32>() {
                            config.volume_percent = v.clamp(0, 100);
                        }
                    }
                    "sync_color_sound" => config.sync_color_sound = value == "true",
                    "pulse_with_beeps" => config.pulse_with_beeps = value == "true",
                    "freq1" => {
                        if let Ok(v) = value.parse::<f32>() {
                            config.success_freq1 = v;
                        }
                    }
                    "freq2" => {
                        if let Ok(v) = value.parse::<f32>() {
                            config.success_freq2 = v;
                        }
                    }
                    "duration1" => {
                        if let Ok(v) = value.parse::<u64>() {
                            config.success_duration1 = v;
                        }
                    }
                    "duration2" => {
                        if let Ok(v) = value.parse::<u64>() {
                            config.success_duration2 = v;
                        }
                    }
                    "beep_count" => {
                        if let Ok(v) = value.parse::<usize>() {
                            config.success_beep_count = v;
                            config.error_beep_count = v;
                            config.warning_beep_count = v;
                            config.beep_count = v;
                        }
                    }
                    "freq" => {
                        if let Ok(v) = value.parse::<f32>() {
                            config.warning_freq = v;
                            config.beep_freq = v;
                            config.error_freq1 = v;
                        }
                    }
                    "duration" => {
                        if let Ok(v) = value.parse::<u64>() {
                            config.warning_duration = v;
                            config.beep_duration = v;
                            config.error_duration1 = v;
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Ok(config)
    }
}

// Get the config file path in the project root (same directory as Cargo.toml)
pub fn get_config_path() -> PathBuf {
    let mut current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
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
        Ok(contents) => {
            match Config::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("  {} Failed to parse config: {}, using defaults", "⚠".yellow(), e);
                    Config::default()
                }
            }
        }
        Err(_) => first_run_setup(),
    }
}

pub fn save_config(config: &Config) {
    let config_path = get_config_path();
    
    match fs::write(&config_path, config.to_string()) {
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
    *SOUND_ENABLED.lock().unwrap() = config.audio_enabled;
    *SYNC_COLOR_SOUND.lock().unwrap() = config.sync_color_sound;
    *PULSE_ENABLED.lock().unwrap() = config.pulse_with_beeps;
    
    sound::configure_sounds(config);
    
    if config.sync_color_sound {
        let sound_state = *SOUND_ENABLED.lock().unwrap();
        *COLOR_ENABLED.lock().unwrap() = sound_state;
    }
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
            _           => println!("  Please enter y or n."),
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
            _                 => println!("  Please enter a number between 0 and 100."),
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

    let audio_enabled = prompt_bool("  Enable audio feedback? [y/n]: ");
    let volume_percent = if audio_enabled {
        prompt_volume("  Volume (0–100): ")
    } else {
        80
    };
    
    let sync_color_sound = if audio_enabled {
        prompt_bool("  Sync colors with sound? [y/n]: ")
    } else {
        false
    };
    
    let pulse_with_beeps = prompt_bool("  Make text pulse (normal->bold) with each beep? [y/n]: ");
    
    println!();
    println!("  Now configure beep counts (1-10 beeps per sound type):");
    let success_beep_count = prompt_usize("  Success beep count: ");
    let error_beep_count = prompt_usize("  Error beep count: ");
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
    let config = load_config();
    match sound_type {
        Some(SoundType::Success) => config.success_beep_count,
        Some(SoundType::Error) => config.error_beep_count,
        Some(SoundType::Warning) => config.warning_beep_count,
        Some(SoundType::Notification) => config.warning_beep_count,
        Some(SoundType::Beep) => config.beep_count,
        None => 0,
    }
}

// Apply bold formatting to text
fn make_bold(text: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", text)
}

// Synchronized pulse and beep - plays all beeps while text is bold
fn synchronized_message<F>(text: &str, beep_count: usize, duration_per_beep: u64, sound_func: F, _is_colored: bool)
where
    F: FnOnce() -> Result<(), String>,
{
    if beep_count == 0 {
        println!("{}", text);
        return;
    }
    
    let should_pulse = *PULSE_ENABLED.lock().unwrap();
    let original_text = text.to_string();
    let bold_text = make_bold(&original_text);
    
    if should_pulse {
        // Show bold text before any beeps
        print!("\r\x1b[K{}", bold_text);
        io::stdout().flush().unwrap();
        
        // Play all sounds (this will play multiple beeps)
        if *SOUND_ENABLED.lock().unwrap() {
            let _ = sound_func();
        }
        
        // Keep bold at the end
        println!();
    } else {
        println!("{}", text);
        if *SOUND_ENABLED.lock().unwrap() {
            let _ = sound_func();
        }
    }
}

// ── Color / sound toggles with sync support ───────────────────────────────────

pub fn sync_color_with_sound(enable: bool) {
    *SYNC_COLOR_SOUND.lock().unwrap() = enable;
    if enable {
        let sound_state = *SOUND_ENABLED.lock().unwrap();
        *COLOR_ENABLED.lock().unwrap() = sound_state;
    }
}

pub fn enable_all() {
    enable_colors();
    enable_sound();
}

pub fn disable_all() {
    disable_colors();
    disable_sound();
}

pub fn enable_colors() {
    *COLOR_ENABLED.lock().unwrap() = true;
    if *SYNC_COLOR_SOUND.lock().unwrap() {
        *SOUND_ENABLED.lock().unwrap() = true;
    }
}

pub fn disable_colors() {
    *COLOR_ENABLED.lock().unwrap() = false;
    if *SYNC_COLOR_SOUND.lock().unwrap() {
        *SOUND_ENABLED.lock().unwrap() = false;
    }
}

pub fn enable_sound() {
    *SOUND_ENABLED.lock().unwrap() = true;
    if *SYNC_COLOR_SOUND.lock().unwrap() {
        *COLOR_ENABLED.lock().unwrap() = true;
    }
}

pub fn disable_sound() {
    *SOUND_ENABLED.lock().unwrap() = false;
    if *SYNC_COLOR_SOUND.lock().unwrap() {
        *COLOR_ENABLED.lock().unwrap() = false;
    }
}

pub fn enable_pulse() {
    *PULSE_ENABLED.lock().unwrap() = true;
}

pub fn disable_pulse() {
    *PULSE_ENABLED.lock().unwrap() = false;
}

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

pub fn play_success_sound() -> Result<(), String> {
    sound::play_success()
}

pub fn play_error_sound() -> Result<(), String> {
    sound::play_error()
}

pub fn play_warning_sound() -> Result<(), String> {
    sound::play_warning()
}

// ── Core message functions ───────────────────────────────────────────────────

pub fn success<M: AsRef<str>>(msg: M) {
    let msg_str = msg.as_ref();
    let config = load_config();
    let beep_count = config.success_beep_count;
    let duration_per_beep = config.success_duration1;
    let is_colored = *COLOR_ENABLED.lock().unwrap();
    
    let text = if is_colored {
        format!("{} {}", colors::check(), msg_str.green())
    } else {
        format!("[✓] {}", msg_str)
    };
    
    synchronized_message(&text, beep_count, duration_per_beep, || {
        sound::play_success()
    }, is_colored);
}

pub fn error<M: AsRef<str>>(msg: M) {
    let msg_str = msg.as_ref();
    let config = load_config();
    let beep_count = config.error_beep_count;
    let duration_per_beep = config.error_duration1;
    let is_colored = *COLOR_ENABLED.lock().unwrap();
    
    let text = if is_colored {
        format!("{} {}", colors::cross(), msg_str.red().bold())
    } else {
        format!("[✗] {}", msg_str)
    };
    
    synchronized_message(&text, beep_count, duration_per_beep, || {
        sound::play_error()
    }, is_colored);
}

pub fn warning<M: AsRef<str>>(msg: M) {
    warn(msg);
}

pub fn warn<M: AsRef<str>>(msg: M) {
    let msg_str = msg.as_ref();
    let config = load_config();
    let beep_count = config.warning_beep_count;
    let duration_per_beep = config.warning_duration;
    let is_colored = *COLOR_ENABLED.lock().unwrap();
    
    let text = if is_colored {
        format!("{} {}", colors::warn(), msg_str.yellow())
    } else {
        format!("[⚠] {}", msg_str)
    };
    
    synchronized_message(&text, beep_count, duration_per_beep, || {
        sound::play_warning()
    }, is_colored);
}

pub fn info<M: AsRef<str>>(msg: M) {
    let msg_str = msg.as_ref();
    let beep_count = 0;
    let is_colored = *COLOR_ENABLED.lock().unwrap();
    
    let text = if is_colored {
        format!("{} {}", colors::info(), msg_str.blue())
    } else {
        format!("[i] {}", msg_str)
    };
    
    synchronized_message(&text, beep_count, 0, || {
        Ok(())
    }, is_colored);
}

pub fn critical<M: AsRef<str>>(msg: M) {
    let msg_str = msg.as_ref();
    let config = load_config();
    let beep_count = config.error_beep_count;
    let duration_per_beep = config.error_duration1;
    let is_colored = *COLOR_ENABLED.lock().unwrap();
    
    let text = if is_colored {
        format!("{} {}", colors::cross(), msg_str.red().bold().on_black())
    } else {
        format!("[✗] CRITICAL: {}", msg_str)
    };
    
    synchronized_message(&text, beep_count, duration_per_beep, || {
        sound::play_error()
    }, is_colored);
}

pub fn colored<M: AsRef<str>>(msg: M, color: Color) {
    if *COLOR_ENABLED.lock().unwrap() {
        println!("{}", msg.as_ref().color(color).bold());
    } else {
        println!("{}", msg.as_ref());
    }
}

// ── Sound-prefixed aliases ────────────────────────────────────────────────────

pub fn sound_success<M: AsRef<str>>(msg: M) {
    success(msg);
}

pub fn sound_error<M: AsRef<str>>(msg: M) {
    error(msg);
}

// ── Status indicator strings ─────────────────────────────────────────────────

pub fn status_valid() -> String {
    colors::valid_prefix()
}

pub fn status_warn() -> String {
    colors::warn_prefix()
}

pub fn status_error() -> String {
    colors::error_prefix()
}

// ── Progress / spinner ────────────────────────────────────────────────────────

static SPINNER_RUNNING: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

fn spinner_running() -> &'static Mutex<bool> {
    &SPINNER_RUNNING
}

pub fn progress<M: AsRef<str>>(msg: M) {
    *spinner_running().lock().unwrap() = true;
    let message = msg.as_ref().to_string();
    let colors_enabled = *COLOR_ENABLED.lock().unwrap();

    thread::spawn(move || {
        let frames = ["◐", "◓", "◑", "◒"];
        let mut i = 0;
        while *spinner_running().lock().unwrap() {
            if colors_enabled {
                print!("\r{} {}", frames[i].cyan(), message);
            } else {
                print!("\r{} {}", frames[i], message);
            }
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            thread::sleep(Duration::from_millis(100));
            i = (i + 1) % frames.len();
        }
    });
}

pub fn progress_complete<M: AsRef<str>>(msg: M) {
    *spinner_running().lock().unwrap() = false;
    thread::sleep(Duration::from_millis(120));
    if *COLOR_ENABLED.lock().unwrap() {
        println!("\r{} {}", colors::check(), msg.as_ref().green());
    } else {
        println!("\r[✓] {}", msg.as_ref());
    }
}

pub fn progress_fail<M: AsRef<str>>(msg: M) {
    *spinner_running().lock().unwrap() = false;
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

pub struct SpinnerHandle {
    _message: String,
}

impl SpinnerHandle {
    fn new<M: AsRef<str>>(msg: M) -> Self {
        progress(msg.as_ref());
        SpinnerHandle { _message: msg.as_ref().to_string() }
    }

    pub fn finish_success<M: AsRef<str>>(&self, msg: M) {
        progress_complete(msg);
    }

    pub fn finish_error<M: AsRef<str>>(&self, msg: M) {
        progress_fail(msg);
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

    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }

    pub fn with_symbol<S: AsRef<str>>(mut self, sym: S) -> Self {
        self.symbol = Some(sym.as_ref().to_string());
        self
    }

    pub fn with_sound(mut self, s: SoundType) -> Self {
        self.sound = Some(s);
        self
    }

    pub fn blinking(mut self) -> Self {
        self.force_pulse = true;
        self
    }

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
        let beep_count = get_beep_count(self.sound);
        let should_pulse = self.force_pulse || (beep_count > 0 && *PULSE_ENABLED.lock().unwrap());
        let output = self.build();
        
        // Get sound function based on sound type
        let sound_func: Box<dyn FnOnce() -> Result<(), String>> = match self.sound {
            Some(SoundType::Success) => Box::new(|| sound::play_success()),
            Some(SoundType::Error) => Box::new(|| sound::play_error()),
            Some(SoundType::Warning) | Some(SoundType::Notification) => Box::new(|| sound::play_warning()),
            Some(SoundType::Beep) => {
                let config = load_config();
                let freq = config.beep_freq;
                let duration = config.beep_duration;
                Box::new(move || sound::play_beep_pub(freq, duration))
            }
            None => Box::new(|| Ok(())),
        };
        
        if should_pulse && beep_count > 0 {
            let bold_text = make_bold(&output);
            
            // Show bold text
            print!("\r\x1b[K{}", bold_text);
            io::stdout().flush().unwrap();
            
            // Play all sounds (this will play multiple beeps)
            if *SOUND_ENABLED.lock().unwrap() {
                let _ = sound_func();
            }
            
            println!();
        } else {
            // Just print without pulsing
            println!("{}", output);
            if *SOUND_ENABLED.lock().unwrap() {
                let _ = sound_func();
            }
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
    fn default() -> Self {
        Self::new("")
    }
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
        let b = message("Test").color(Color::Red).with_symbol("!").with_sound(SoundType::Beep);
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
        assert_eq!(config.error_beep_count, 2);
        assert_eq!(config.warning_beep_count, 2);
    }
}