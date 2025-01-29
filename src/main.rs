use std::env;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread::sleep;
use chrono;

use sysinfo::{System, CpuRefreshKind};
use rodio::{Decoder, OutputStream, Sink};

fn main() {
    // Get the CPU usage threshold from command line arguments or use default (20%)
    let args: Vec<String> = env::args().collect();
    let threshold: f32 = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(20.0);
    log(&format!("CPU Threshold: {}%", threshold));

    // Path to the alert sound file
    let alert_sound_path: &str = args.get(2)
        .and_then(|s| Some(s.as_str()))
        .unwrap_or("alert.wav");
    log(&format!("Alert sound path: \"{}\"", alert_sound_path));

    // Initialize the system information struct
    let mut sys = System::new_all();

    // Initialize the audio output stream
    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok(stream) => stream,
        Err(e) => {
            log(&format!("Error initializing audio output stream: {}", e));
            return;
        }
    };

    log(&format!("Monitoring CPU usage. Alert will sound if usage drops below {}%.", threshold));
    log("Press Ctrl+C to exit.");

    loop {
        // Refresh CPU data
        sys.refresh_cpu_specifics(CpuRefreshKind::everything());

        // Get the average CPU usage across all cores
        let cpu_usage = sys.global_cpu_usage();

        if cpu_usage < threshold {
            log(&format!("Current CPU Usage: {:.2}% (below threshold)", cpu_usage));

            log("CPU usage below threshold! Playing alert sound.");

            // Play the alert sound
            if let Err(e) = play_sound(&alert_sound_path, &stream_handle) {
                log(&format!("Error playing sound: {}", e));
            }

            // Optional: Wait until CPU usage rises above the threshold to avoid repeated alerts
            wait_until_above_threshold(&mut sys, threshold);
        } else {
            log(&format!("Current CPU Usage: {:.2}% (above threshold)", cpu_usage));
        }

        // Wait for a specified interval before checking again (e.g., 1 second)
        sleep(Duration::from_secs(1));
    }
}

/// Plays a sound from the given file path using the provided stream handle.
fn play_sound(file_path: &str, stream_handle: &rodio::OutputStreamHandle) -> Result<(), rodio::decoder::DecoderError> {
    // Open the audio file
    let file = File::open(file_path).expect("Failed to open alert sound file.");
    let source = Decoder::new(BufReader::new(file))?;

    // Create a new sink (audio queue)
    let sink = Sink::try_new(stream_handle).expect("Failed to create audio sink.");

    // Add the source to the sink
    sink.append(source);

    // Optionally, you can set the volume (0.0 to 1.0)
    // sink.set_volume(0.5);

    // Detach the sink to play the sound asynchronously
    sink.detach();

    Ok(())
}

/// Waits until the CPU usage rises above the specified threshold.
fn wait_until_above_threshold(sys: &mut System, threshold: f32) {
    loop {
        // Delay longer when below threshold to smooth out system activity.
        sleep(Duration::from_secs(10));
        sys.refresh_cpu_specifics(CpuRefreshKind::everything());
        let cpu_usage = sys.global_cpu_usage();
        if cpu_usage >= threshold {
            log(&format!("Current CPU Usage: {:.2}% (above threshold)", cpu_usage));
            log("CPU usage has risen above the threshold.");
            break;
        } else {
            log(&format!("Current CPU Usage: {:.2}% (below threshold)", cpu_usage));
        }
    }
}

/// Logs a message with the current datetime in ISO-8601 format.
fn log(message: &str) {
    let dtnow = chrono::Local::now();
    println!("[{}] {}", dtnow.format("%Y-%m-%d %H:%M:%S"), message);
}
