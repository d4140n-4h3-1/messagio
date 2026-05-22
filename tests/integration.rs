use messagio::*;

#[test]
fn test_message_functions() {
    // These just test that functions don't panic
    success("Test success");
    error("Test error");
    warning("Test warning");
    info("Test info");
}

#[test]
fn test_builder() {
    let builder = message("Test")
        .color(Color::Red)
        .with_symbol("!")
        .with_sound(SoundType::Beep);
    
    builder.send();
}

#[test]
fn test_sound_types() {
    // Test that sound functions don't crash
    let _ = play_success_sound();
    let _ = play_error_sound();
    let _ = play_warning_sound();
}