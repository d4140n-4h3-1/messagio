use colored::*;

pub fn check() -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        format!("[{}]", "✓".green().bold())
    } else {
        "[✓]".to_string()
    }
}

pub fn cross() -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        format!("[{}]", "✗".red().bold())
    } else {
        "[✗]".to_string()
    }
}

pub fn warn() -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        format!("[{}]", "⚠".yellow().bold())
    } else {
        "[⚠]".to_string()
    }
}

pub fn info() -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        format!("[{}]", "i".blue().bold())
    } else {
        "[i]".to_string()
    }
}

pub fn valid_prefix() -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        format!("  {} VALID", "✓".green().bold())
    } else {
        "  [✓] VALID".to_string()
    }
}

pub fn warn_prefix() -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        format!("  {} WARN", "⚠".yellow().bold())
    } else {
        "  [⚠] WARN".to_string()
    }
}

pub fn error_prefix() -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        format!("  {} ERROR", "✗".red().bold())
    } else {
        "  [✗] ERROR".to_string()
    }
}

pub fn colorize<S: AsRef<str>>(text: S, color: Color) -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        text.as_ref().color(color).bold().to_string()
    } else {
        text.as_ref().to_string()
    }
}