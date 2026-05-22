use colored::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub mod colors;
pub mod sound;

// Re-export Color so callers don't need to import colored directly
pub use colored::Color;

pub static COLOR_ENABLED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));
pub static SOUND_ENABLED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));

// ── Color / sound toggles ────────────────────────────────────────────────────

pub fn enable_colors() {
    *COLOR_ENABLED.lock().unwrap() = true;
}

pub fn disable_colors() {
    *COLOR_ENABLED.lock().unwrap() = false;
}

pub fn enable_sound() {
    *SOUND_ENABLED.lock().unwrap() = true;
}

pub fn disable_sound() {
    *SOUND_ENABLED.lock().unwrap() = false;
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

// ── Standalone sound helpers (used in tests) ─────────────────────────────────

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
    if *COLOR_ENABLED.lock().unwrap() {
        println!("{} {}", colors::check(), msg.as_ref().green());
    } else {
        println!("[✓] {}", msg.as_ref());
    }
    if *SOUND_ENABLED.lock().unwrap() {
        let _ = sound::play_success();
    }
}

pub fn error<M: AsRef<str>>(msg: M) {
    if *COLOR_ENABLED.lock().unwrap() {
        println!("{} {}", colors::cross(), msg.as_ref().red().bold());
    } else {
        println!("[✗] {}", msg.as_ref());
    }
    if *SOUND_ENABLED.lock().unwrap() {
        let _ = sound::play_error();
    }
}

/// Alias matching the demo/tests (`warning` vs `warn`).
pub fn warning<M: AsRef<str>>(msg: M) {
    warn(msg);
}

pub fn warn<M: AsRef<str>>(msg: M) {
    if *COLOR_ENABLED.lock().unwrap() {
        println!("{} {}", colors::warn(), msg.as_ref().yellow());
    } else {
        println!("[⚠] {}", msg.as_ref());
    }
    if *SOUND_ENABLED.lock().unwrap() {
        let _ = sound::play_warning();
    }
}

pub fn info<M: AsRef<str>>(msg: M) {
    if *COLOR_ENABLED.lock().unwrap() {
        println!("{} {}", colors::info(), msg.as_ref().blue());
    } else {
        println!("[i] {}", msg.as_ref());
    }
}

/// High-visibility critical message (bold red, no sound gate).
pub fn critical<M: AsRef<str>>(msg: M) {
    if *COLOR_ENABLED.lock().unwrap() {
        println!("{} {}", colors::cross(), msg.as_ref().red().bold().on_black());
    } else {
        println!("[✗] CRITICAL: {}", msg.as_ref());
    }
    if *SOUND_ENABLED.lock().unwrap() {
        let _ = sound::play_error();
    }
}

pub fn colored<M: AsRef<str>>(msg: M, color: Color) {
    if *COLOR_ENABLED.lock().unwrap() {
        println!("{}", msg.as_ref().color(color).bold());
    } else {
        println!("{}", msg.as_ref());
    }
}

// ── Sound-prefixed aliases (used in demo) ────────────────────────────────────

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

/// Starts an animated progress spinner with a label.
/// Call `progress_complete` or `progress_fail` to stop it.
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
    thread::sleep(Duration::from_millis(120)); // let spinner thread exit
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

/// Convenience wrapper that returns a `SpinnerHandle` for structured start/stop.
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
    blink: bool,
}

impl MessageBuilder {
    pub fn new<M: AsRef<str>>(msg: M) -> Self {
        MessageBuilder {
            text: msg.as_ref().to_string(),
            color: None,
            symbol: None,
            sound: None,
            blink: false,
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

    /// Request blinking text (rendered as bold when the terminal doesn't support blink).
    pub fn blinking(mut self) -> Self {
        self.blink = true;
        self
    }

    pub fn build(&self) -> String {
        let colors_enabled = *COLOR_ENABLED.lock().unwrap();
        let mut out = self.text.clone();

        if colors_enabled {
            if let Some(c) = self.color {
                out = if self.blink {
                    out.color(c).bold().to_string()
                } else {
                    out.color(c).to_string()
                };
            }
        }

        if let Some(ref sym) = self.symbol {
            out = format!("{} {}", sym, out);
        }

        out
    }

    pub fn send(&self) {
        println!("{}", self.build());

        if *SOUND_ENABLED.lock().unwrap() {
            if let Some(s) = self.sound {
                let _ = match s {
                    SoundType::Success => sound::play_success(),
                    SoundType::Error => sound::play_error(),
                    SoundType::Warning | SoundType::Notification => sound::play_warning(),
                    SoundType::Beep => sound::play_beep_pub(440.0, 150),
                };
            }
        }
    }

    // Legacy compat for the builder-as-struct pattern in lib.rs tests
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

/// `color_println!(Blue, "some {} text", value)`
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

/// `color_print!(Green, "partial ")`  — no newline
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
        // The original test inside lib.rs used the old API;
        // keep it working via the new builder.
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
}