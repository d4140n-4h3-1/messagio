use messagio::*;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    println!("=== Messagio Demo ===");

    // Load and apply configuration
    let config = load_config();
    apply_config(&config);

    // Display current config
    println!("=== Current Configuration ===");
    println!("Audio Enabled: {}", config.audio_enabled);
    println!("Volume: {}%", config.volume_percent);
    println!("Color-Sound Sync: {}", config.sync_color_sound);
    println!("Pulse With Beeps: {}", config.pulse_with_beeps);
    println!(
        "Success Beeps: {}, {}Hz + {}Hz ({}ms + {}ms)",
        config.success_beep_count,
        config.success_freq1, config.success_freq2,
        config.success_duration1, config.success_duration2
    );
    println!(
        "Error Beeps: {}, {}Hz + {}Hz ({}ms + {}ms)",
        config.error_beep_count,
        config.error_freq1, config.error_freq2,
        config.error_duration1, config.error_duration2
    );
    println!(
        "Warning Beeps: {}, {}Hz ({}ms)",
        config.warning_beep_count,
        config.warning_freq, config.warning_duration
    );
    println!();

    // Basic messages
    println!("=== Basic Messages ===");
    success("File saved successfully");
    error("Failed to connect to database");
    warning("Disk space is running low");
    info("New version available");
    println!();

    // Critical message
    println!("=== Critical Message ===");
    critical("System failure detected!");
    println!();

    // Progress indicators
    println!("=== Progress Indicators ===");
    progress("Loading data");
    sleep(Duration::from_millis(1500));
    progress_complete("Data loaded successfully");

    progress("Processing files");
    sleep(Duration::from_millis(1000));
    progress_fail("Processing failed");
    println!();

    // Spinner with handle
    println!("=== Spinner Handle ===");
    let spinner1 = spinner("Downloading updates");
    sleep(Duration::from_millis(2000));
    spinner1.finish_success("Updates downloaded");

    let spinner2 = spinner("Installing packages");
    sleep(Duration::from_millis(1500));
    spinner2.finish_error("Installation failed");
    println!();

    // Builder pattern
    println!("=== Builder Pattern ===");
    message("Custom info message")
        .color(Color::Cyan)
        .with_symbol("ℹ")
        .send();

    message("Warning with sound")
        .color(Color::Yellow)
        .with_symbol("⚠")
        .with_sound(SoundType::Warning)
        .send();

    message("Success with custom style")
        .color(Color::Green)
        .with_symbol("✨")
        .with_sound(SoundType::Success)
        .blinking()
        .send();
    println!();

    // Chained colored text
    println!("=== Colored Text Chaining ===");
    message("Status:")
        .colored_text("Online", Color::Green)
        .text("|")
        .colored_text("Authenticated", Color::Blue)
        .text("|")
        .colored_text("Ready", Color::Yellow)
        .send();
    println!();

    // Sound tests
    println!("=== Sound Tests ===");
    sound_success("Success sound test");
    sleep(Duration::from_millis(500));
    sound_error("Error sound test");
    sleep(Duration::from_millis(500));
    println!();

    // Standalone sounds
    println!("=== Standalone Sounds ===");
    println!("Playing success sound...");
    let _ = play_success_sound();
    sleep(Duration::from_millis(500));

    println!("Playing error sound...");
    let _ = play_error_sound();
    sleep(Duration::from_millis(500));

    println!("Playing warning sound...");
    let _ = play_warning_sound();
    println!();

    // Status indicators
    println!("=== Status Indicators ===");
    println!("{} Database connection OK", status_valid());
    println!("{} High memory usage", status_warn());
    println!("{} Disk full", status_error());
    println!();

    // Macro usage
    println!("=== Color Macros ===");
    color_println!(Blue, "This is blue text");
    color_print!(Green, "This is ");
    color_print!(Yellow, "colored ");
    color_print!(Cyan, "output");
    println!();

    // Pulse text demo
    println!("=== Pulse Text Demo ===");
    println!("Text pulses from normal to bold with each beep:");
    success("This message pulses with the success sound");
    error("This message pulses with the error sound");
    warning("This message pulses with the warning sound");

    println!("You can force pulse with .blinking():");
    message("Forced pulse even with single beep")
        .with_sound(SoundType::Beep)
        .blinking()
        .send();

    println!("Control pulsing with:");
    println!("  enable_pulse()   - Turn on pulsing");
    println!("  disable_pulse()  - Turn off pulsing");
    println!();

    // Feature toggles
    println!("=== Feature Toggles ===");
    println!("Available runtime toggles:");
    println!("  disable_colors()             - Turn off colors");
    println!("  enable_colors()              - Turn on colors");
    println!("  disable_sound()              - Turn off sounds");
    println!("  enable_sound()               - Turn on sounds");
    println!("  enable_all()                 - Turn on everything");
    println!("  disable_all()                - Turn off everything");
    println!("  sync_color_with_sound(true)  - Sync colors and sounds");
    println!("  enable_pulse()               - Turn on pulsing");
    println!("  disable_pulse()              - Turn off pulsing");
    println!();

    println!("=== Demo Complete ===");
    println!("Configuration file location: {}", get_config_path().display());
    println!("Edit messagio.toml to customize beep counts and sound frequencies");
    println!();
}