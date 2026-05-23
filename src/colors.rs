use colored::*;

pub fn check() -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        format!("[{}]", " OK ".green().bold())
    } else {
        "[ OK ]".to_string()
    }
}

pub fn cross() -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        format!("[{}]", "FAIL".red().bold())
    } else {
        "[FAIL]".to_string()
    }
}

pub fn warn() -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        format!("[{}]", "WARN".yellow().bold())
    } else {
        "[WARN]".to_string()
    }
}

pub fn info() -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        format!("[{}]", "INFO".cyan().bold())
    } else {
        "[INFO]".to_string()
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

pub fn status_valid() -> String { valid_prefix() }
pub fn status_warn() -> String { warn_prefix() }
pub fn status_error() -> String { error_prefix() }

pub fn colorize<S: AsRef<str>>(text: S, color: Color) -> String {
    if *crate::COLOR_ENABLED.lock().unwrap() {
        text.as_ref().color(color).bold().to_string()
    } else {
        text.as_ref().to_string()
    }
}