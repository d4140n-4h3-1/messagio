use messagio::*;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    println!("\n=== Messagio Demo ===\n");
    
    // Force config to be created/loaded
    let config = load_config();
    apply_config(&config);
    
    // Optional: Save config if it doesn't exist
    save_config(&config);
    
    // Basic messages
    success("File saved successfully");
    error("Failed to connect to database");
    warning("Disk space is running low");
    info("New version available");
    
    println!("\n=== Critical Message ===");
    critical("System failure detected!");
    
    println!("\n=== Progress Example ===");
    progress("Loading");
    sleep(Duration::from_millis(1500));
    progress_complete("Loading complete");
    
    progress("Processing");
    sleep(Duration::from_millis(1000));
    progress_fail("Processing failed");
    
    println!("\n=== Builder Pattern ===");
    message("Custom message")
        .color(Color::Magenta)
        .with_symbol("→")
        .send();
    
    message("Important notification")
        .color(Color::Cyan)
        .with_sound(SoundType::Notification)
        .blinking()
        .send();
    
    println!("\n=== Sound Tests ===");
    sound_success("Operation completed");
    sound_error("Connection lost");
    
    println!("\n=== Status Indicators ===");
    println!("{}", status_valid());
    println!("{}", status_warn());
    println!("{}", status_error());
    
    // Macro usage
    println!("\n=== Macros ===");
    color_println!(Blue, "This is blue text");
    color_print!(Green, "This is ");
    color_print!(Yellow, "colored ");
    color_print!(Cyan, "output\n");
    
    println!("\n=== Config Location ===");
    println!("Config file should be at: {}", get_config_path().display());
}