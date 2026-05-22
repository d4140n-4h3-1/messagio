use once_cell::sync::Lazy;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use colored::Colorize;

// Sound configuration
static SOUND_CONFIG: Lazy<Mutex<SoundConfig>> = Lazy::new(|| Mutex::new(SoundConfig::default()));

#[derive(Debug, Clone)]
pub struct SoundConfig {
    pub success_freq1: f32,
    pub success_freq2: f32,
    pub success_duration1: u64,
    pub success_duration2: u64,
    pub error_freq1: f32,
    pub error_freq2: f32,
    pub error_duration1: u64,
    pub error_duration2: u64,
    pub warning_freq: f32,
    pub warning_duration: u64,
    pub beep_freq: f32,
    pub beep_duration: u64,
}

impl SoundConfig {
    pub fn default() -> Self {
        Self {
            success_freq1: 440.0,
            success_freq2: 880.0,
            success_duration1: 200,
            success_duration2: 200,
            error_freq1: 220.0,
            error_freq2: 110.0,
            error_duration1: 300,
            error_duration2: 300,
            warning_freq: 660.0,
            warning_duration: 150,
            beep_freq: 440.0,
            beep_duration: 150,
        }
    }
}

pub fn configure_sounds(config: &crate::Config) {
    let mut sound_config = SOUND_CONFIG.lock().unwrap();
    sound_config.success_freq1 = config.success_freq1;
    sound_config.success_freq2 = config.success_freq2;
    sound_config.success_duration1 = config.success_duration1;
    sound_config.success_duration2 = config.success_duration2;
    sound_config.error_freq1 = config.error_freq1;
    sound_config.error_freq2 = config.error_freq2;
    sound_config.error_duration1 = config.error_duration1;
    sound_config.error_duration2 = config.error_duration2;
    sound_config.warning_freq = config.warning_freq;
    sound_config.warning_duration = config.warning_duration;
    sound_config.beep_freq = config.beep_freq;
    sound_config.beep_duration = config.beep_duration;
}

// ── Audio handle ─────────────────────────────────────────────────────────────

static STREAM_HANDLE: Lazy<OutputStreamHandle> = Lazy::new(|| {
    let (tx, rx) = mpsc::sync_channel::<OutputStreamHandle>(1);

    thread::Builder::new()
        .name("messagio-audio".into())
        .spawn(move || {
            let (_stream, handle) = OutputStream::try_default()
                .expect("messagio: failed to open audio output");
            tx.send(handle).expect("messagio: audio handle send failed");
            loop {
                thread::park();
            }
        })
        .expect("messagio: failed to spawn audio thread");

    rx.recv().expect("messagio: audio handle recv failed")
});

pub fn play_success() -> Result<(), String> {
    let config = SOUND_CONFIG.lock().unwrap();
    play_beep(config.success_freq1, config.success_duration1)?;
    thread::sleep(std::time::Duration::from_millis(50));
    play_beep(config.success_freq2, config.success_duration2)?;
    Ok(())
}

pub fn play_error() -> Result<(), String> {
    let config = SOUND_CONFIG.lock().unwrap();
    play_beep(config.error_freq1, config.error_duration1)?;
    thread::sleep(std::time::Duration::from_millis(100));
    play_beep(config.error_freq2, config.error_duration2)?;
    Ok(())
}

pub fn play_warning() -> Result<(), String> {
    let config = SOUND_CONFIG.lock().unwrap();
    play_beep(config.warning_freq, config.warning_duration)?;
    thread::sleep(std::time::Duration::from_millis(50));
    play_beep(config.warning_freq, config.warning_duration)?;
    Ok(())
}

pub fn play_beep_pub(frequency: f32, duration_ms: u64) -> Result<(), String> {
    play_beep(frequency, duration_ms)
}

fn play_beep(frequency: f32, duration_ms: u64) -> Result<(), String> {
    let sink = match Sink::try_new(&*STREAM_HANDLE) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to create sink: {}", e)),
    };

    let source = BeepSource {
        frequency,
        sample_rate: 44100,
        num_samples: (duration_ms * 44100 / 1000) as usize,
        current_sample: 0,
    };

    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

struct BeepSource {
    frequency: f32,
    sample_rate: u32,
    num_samples: usize,
    current_sample: usize,
}

impl Iterator for BeepSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.num_samples {
            None
        } else {
            let t = self.current_sample as f32 / self.sample_rate as f32;
            let value = (2.0 * std::f32::consts::PI * self.frequency * t).sin();
            self.current_sample += 1;
            Some(value)
        }
    }
}

impl Source for BeepSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.num_samples - self.current_sample)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_secs_f32(
            self.num_samples as f32 / self.sample_rate as f32,
        ))
    }
}

pub fn play_wav_file<P: AsRef<std::path::Path>>(path: P) -> Result<(), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let source = Decoder::new(BufReader::new(file))
        .map_err(|e: rodio::decoder::DecoderError| e.to_string())?;

    let sink = Sink::try_new(&*STREAM_HANDLE)
        .map_err(|e| e.to_string())?;

    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

pub fn success_message<M: AsRef<str>>(message: M) {
    let colors_enabled = *crate::COLOR_ENABLED.lock().unwrap();
    if colors_enabled {
        println!("[✓] {}", message.as_ref().green());
    } else {
        println!("[✓] {}", message.as_ref());
    }
    let _ = play_success();
}

pub fn error_message<M: AsRef<str>>(message: M) {
    let colors_enabled = *crate::COLOR_ENABLED.lock().unwrap();
    if colors_enabled {
        println!("[✗] {}", message.as_ref().red().bold());
    } else {
        println!("[✗] {}", message.as_ref());
    }
    let _ = play_error();
}