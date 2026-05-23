use messagio::*;
use colored::Colorize;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    println!("{}", "═".repeat(60).cyan());
    println!("{}", "                    MESSAGIO DEMO".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();

    // Load and apply configuration
    let config = load_config();
    apply_config(&config);

    // Display current config
    println!("{}", "📋 CURRENT CONFIGURATION".bold().underline());
    println!("  Audio Enabled: {}", if config.audio_enabled { "✅ ON".green() } else { "❌ OFF".red() });
    println!("  Volume: {}%", config.volume_percent);
    println!("  Color-Sound Sync: {}", if config.sync_color_sound { "✅ ON".green() } else { "❌ OFF".red() });
    println!("  Pulse With Beeps: {}", if config.pulse_with_beeps { "✅ ON".green() } else { "❌ OFF".red() });
    println!();
    
    println!("{}", "⚡ SPEED SETTINGS".bold().underline());
    println!("  Pulse Speed: {}ms (between state changes)", get_pulse_speed_ms());
    println!("  Beep Interval: {}ms (between multiple beeps)", get_beep_interval_ms());
    println!("  Spinner Speed: {}ms (per frame)", get_spinner_speed_ms());
    println!();
    
    println!("  Success Beeps: {}, {}Hz + {}Hz ({}ms + {}ms)",
        config.success_beep_count,
        config.success_freq1, config.success_freq2,
        config.success_duration1, config.success_duration2
    );
    println!("  Error Beeps: {}, {}Hz + {}Hz ({}ms + {}ms)",
        config.error_beep_count,
        config.error_freq1, config.error_freq2,
        config.error_duration1, config.error_duration2
    );
    println!("  Warning Beeps: {}, {}Hz ({}ms)",
        config.warning_beep_count,
        config.warning_freq, config.warning_duration
    );
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // Basic Messages
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "📝 BASIC MESSAGES".bold().underline());
    success("File saved successfully");
    error("Failed to connect to database");
    warning("Disk space is running low");
    println!("{}", info("New version available"));
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // Critical Message
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "⚠️ CRITICAL MESSAGE".bold().underline());
    critical("System failure detected!");
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // Pulse Functions (NEW!)
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "✨ PULSE FUNCTIONS (NEW!)".bold().underline());
    
    println!("\n  → Musical pulse (success sound):");
    pulse_musical("🎵 MUSICAL: This pulses with a success chime!", Color::Cyan, 3);
    
    println!("\n  → Gentle pulse (warning sound):");
    pulse_gentle("⚠️ GENTLE: This pulses with a warning sound", Color::Yellow, 3);
    
    println!("\n  → Error pulse (error sound):");
    pulse_with_error_sound("❌ ERROR: This pulses with an error sound", Color::Red, 3);
    
    println!("\n  → Custom pulse (beep sound):");
    pulse_custom("🔔 CUSTOM: This pulses with a single beep", Color::Green, SoundType::Beep, 3);
    
    println!("\n  → Custom pulse (success sound):");
    pulse_custom("✅ SUCCESS: Custom pulse with success sound", Color::Green, SoundType::Success, 2);
    
    println!();
    sleep(Duration::from_millis(500));

    // ─────────────────────────────────────────────────────────────────────────
    // Progress Indicators
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "⏳ PROGRESS INDICATORS".bold().underline());
    progress("Loading data");
    sleep(Duration::from_millis(1500));
    progress_complete("Data loaded successfully");

    progress("Processing files");
    sleep(Duration::from_millis(1000));
    progress_fail("Processing failed");
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // Spinner with Handle
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "🔄 SPINNER HANDLE".bold().underline());
    let spinner1 = spinner("Downloading updates");
    sleep(Duration::from_millis(2000));
    spinner1.finish_success("Updates downloaded");

    let spinner2 = spinner("Installing packages");
    sleep(Duration::from_millis(1500));
    spinner2.finish_error("Installation failed");
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // Builder Pattern
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "🏗️ BUILDER PATTERN".bold().underline());
    message("Custom info message")
        .color(Color::Cyan)
        .with_symbol("ℹ️")
        .send();

    message("Warning with sound")
        .color(Color::Yellow)
        .with_symbol("⚠️")
        .with_sound(SoundType::Warning)
        .send();

    message("Success with custom style")
        .color(Color::Green)
        .with_symbol("✨")
        .with_sound(SoundType::Success)
        .blinking()
        .send();

    message("Error with pulse")
        .color(Color::Red)
        .with_symbol("💥")
        .with_sound(SoundType::Error)
        .blinking()
        .send();
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // Chained Colored Text
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "🌈 COLORED TEXT CHAINING".bold().underline());
    message("Status:")
        .colored_text("Online", Color::Green)
        .text("|")
        .colored_text("Authenticated", Color::Blue)
        .text("|")
        .colored_text("Ready", Color::Yellow)
        .send();
    
    message("Server:")
        .colored_text("Running", Color::Green)
        .text("•")
        .colored_text("Port 8080", Color::Cyan)
        .text("•")
        .colored_text("SSL Enabled", Color::Green)
        .send();
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // Standalone Sounds
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "🔊 STANDALONE SOUNDS".bold().underline());
    println!("  Playing success sound...");
    let _ = play_success_sound();
    sleep(Duration::from_millis(500));

    println!("  Playing error sound...");
    let _ = play_error_sound();
    sleep(Duration::from_millis(500));

    println!("  Playing warning sound...");
    let _ = play_warning_sound();
    println!();
    sleep(Duration::from_millis(500));

    // ─────────────────────────────────────────────────────────────────────────
    // Status Indicators
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "📊 STATUS INDICATORS".bold().underline());
    println!("{} Database connection OK", status_valid());
    println!("{} High memory usage", status_warn());
    println!("{} Disk full", status_error());
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // Color Macros
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "🎨 COLOR MACROS".bold().underline());
    color_println!(Blue, "This is blue text");
    color_print!(Green, "This is ");
    color_print!(Yellow, "colored ");
    color_print!(Cyan, "output");
    println!();
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // Feature Toggles Demo
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "⚙️ FEATURE TOGGLES".bold().underline());
    println!("  Available runtime toggles:");
    println!("    • disable_colors()     - Turn off colors");
    println!("    • enable_colors()      - Turn on colors");
    println!("    • disable_sound()      - Turn off sounds");
    println!("    • enable_sound()       - Turn on sounds");
    println!("    • enable_all()         - Turn on everything");
    println!("    • disable_all()        - Turn off everything");
    println!("    • sync_color_with_sound(true) - Sync colors and sounds");
    println!("    • enable_pulse()       - Turn on pulsing");
    println!("    • disable_pulse()      - Turn off pulsing");
    println!("    • set_pulse_speed_ms() - Change pulse animation speed");
    println!("    • set_spinner_speed_ms() - Change spinner speed");
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // Pulse Settings Demo
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "🔧 PULSE SETTINGS DEMO".bold().underline());
    println!("  Pulse is currently: {}", 
        if *PULSE_ENABLED.lock().unwrap() { "ENABLED".green() } else { "DISABLED".red() }
    );
    println!("  Pulse speed: {}ms", get_pulse_speed_ms());
    
    println!("\n  Disabling pulse for next message...");
    disable_pulse();
    pulse_musical("  This message pulses WITHOUT visual effect", Color::Magenta, 2);
    
    println!("\n  Re-enabling pulse...");
    enable_pulse();
    pulse_musical("  This message pulses WITH visual effect", Color::Magenta, 2);
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // Speed Settings Demo
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "🏃 SPEED SETTINGS DEMO".bold().underline());
    println!("  Current pulse speed: {}ms", get_pulse_speed_ms());
    println!("  Testing fast pulse (100ms)...");
    let original_speed = get_pulse_speed_ms();
    set_pulse_speed_ms(100);
    pulse_musical("  FAST PULSE", Color::Cyan, 3);
    set_pulse_speed_ms(original_speed);
    println!("  Restored to {}ms", original_speed);
    println!();

    // ─────────────────────────────────────────────────────────────────────────
    // Complete Demo
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "🎉 DEMO COMPLETE".bold().underline());
    println!("  Configuration file location: {}", get_config_path().display());
    println!("  Edit messagio.toml to customize:");
    println!("    • Beep counts (1-10 per sound type)");
    println!("    • Sound frequencies (Hz)");
    println!("    • Sound durations (ms)");
    println!("    • Volume (0-100)");
    println!("    • Pulse with beeps (on/off)");
    println!("    • Color-sound sync (on/off)");
    println!("    • Pulse speed (ms between pulses)");
    println!("    • Spinner speed (ms per frame)");
    println!();
    
    success("Messagio demo completed successfully!");
    println!();
    println!("{}", "═".repeat(60).cyan());
}