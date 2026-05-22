use once_cell::sync::Lazy;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use std::thread;
use colored::Colorize;

// ── Audio handle ─────────────────────────────────────────────────────────────
//
// `OutputStream` is !Send because CPAL's platform stream contains a raw pointer.
// The correct approach is to keep the `OutputStream` alive on a dedicated
// background thread (so it never crosses a thread boundary) and only share the
// `OutputStreamHandle`, which *is* Send + Sync.

static STREAM_HANDLE: Lazy<OutputStreamHandle> = Lazy::new(|| {
    let (tx, rx) = mpsc::sync_channel::<OutputStreamHandle>(1);

    thread::Builder::new()
        .name("messagio-audio".into())
        .spawn(move || {
            // OutputStream must be created and dropped on the same thread.
            let (_stream, handle) = OutputStream::try_default()
                .expect("messagio: failed to open audio output");
            tx.send(handle).expect("messagio: audio handle send failed");
            // Park forever — dropping `_stream` would close the device.
            loop {
                thread::park();
            }
        })
        .expect("messagio: failed to spawn audio thread");

    rx.recv().expect("messagio: audio handle recv failed")
});

pub fn play_success() -> Result<(), String> {
    play_beep(440.0, 200)?;
    thread::sleep(std::time::Duration::from_millis(50));
    play_beep(880.0, 200)?;
    Ok(())
}

pub fn play_error() -> Result<(), String> {
    play_beep(220.0, 300)?;
    thread::sleep(std::time::Duration::from_millis(100));
    play_beep(110.0, 300)?;
    Ok(())
}

pub fn play_warning() -> Result<(), String> {
    play_beep(660.0, 150)?;
    thread::sleep(std::time::Duration::from_millis(50));
    play_beep(660.0, 150)?;
    Ok(())
}

/// Public single-beep helper for use from lib.rs (e.g. SoundType::Beep).
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

/// Convenience: print a green success line and play the success sound.
pub fn success_message<M: AsRef<str>>(message: M) {
    let colors_enabled = *crate::COLOR_ENABLED.lock().unwrap();
    if colors_enabled {
        println!("[✓] {}", message.as_ref().green());
    } else {
        println!("[✓] {}", message.as_ref());
    }
    let _ = play_success();
}

/// Convenience: print a red error line and play the error sound.
pub fn error_message<M: AsRef<str>>(message: M) {
    let colors_enabled = *crate::COLOR_ENABLED.lock().unwrap();
    if colors_enabled {
        println!("[✗] {}", message.as_ref().red().bold());
    } else {
        println!("[✗] {}", message.as_ref());
    }
    let _ = play_error();
}