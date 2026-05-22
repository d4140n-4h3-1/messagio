use messagio::*;

#[test]
fn test_message_functions() {
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
    let _ = play_success_sound();
    let _ = play_error_sound();
    let _ = play_warning_sound();
}

#[test]
fn test_blink_feature() {
    enable_blink();
    assert!(*BLINK_ENABLED.lock().unwrap());
    
    disable_blink();
    assert!(!*BLINK_ENABLED.lock().unwrap());
    
    enable_blink();
}

#[test]
fn test_sync_feature() {
    sync_color_with_sound(true);
    assert!(*SYNC_COLOR_SOUND.lock().unwrap());
    
    disable_sound();
    assert!(!*SOUND_ENABLED.lock().unwrap());
    assert!(!*COLOR_ENABLED.lock().unwrap());
    
    enable_colors();
    assert!(*SOUND_ENABLED.lock().unwrap());
    assert!(*COLOR_ENABLED.lock().unwrap());
    
    sync_color_with_sound(false);
}