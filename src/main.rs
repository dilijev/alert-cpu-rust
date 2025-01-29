use std::env;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use std::thread::sleep;
use chrono;

use sysinfo::{System, CpuRefreshKind};
use rodio::{Decoder, OutputStream, Sink};

enum CpuState {
    Initial,
    RisingEdge,
    OverThreshold,
    FallingEdge,
    BelowThreshold,
}

trait CpuMonitor {
    fn refresh_cpu(&mut self);
    fn global_cpu_usage(&self) -> f32;
}

impl CpuMonitor for System {
    fn refresh_cpu(&mut self) {
        self.refresh_cpu_specifics(CpuRefreshKind::everything());
    }

    fn global_cpu_usage(&self) -> f32 {
        self.global_cpu_usage()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Get the CPU usage threshold from the command line arguments
    // or use the default value (20% CPU).
    let threshold: f32 = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(20.0);
    log(&format!("CPU Threshold: {}%", threshold));

    // Get the path to the alert sound file from the command line arguments
    // or use the default value ("alert.wav").
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

    monitor_cpu(&mut sys, threshold, &alert_sound_path, &stream_handle)
        .unwrap_or_else(|e| log(&format!("Error playing alert sound: {}", e)));
}

fn monitor_cpu<T: CpuMonitor>(sys: &mut T, threshold: f32, alert_sound_path: &str, stream_handle: &rodio::OutputStreamHandle) -> Result<(), rodio::decoder::DecoderError> {
    log(&format!("Playing alert sound once on startup."));
    play_sound(alert_sound_path, stream_handle)?;

    log(&format!("Monitoring CPU usage. Alert will sound if usage drops below {}%.", threshold));
    log("Press Ctrl+C to exit.");

    let mut state = CpuState::Initial;
    let mut above_threshold_count = 0;
    let mut below_threshold_count = 0;

    loop {
        // Refresh CPU data
        sys.refresh_cpu();

        // Get the average CPU usage across all cores
        let cpu_usage = sys.global_cpu_usage();

        state = match state {
            CpuState::Initial => {
                log_as_per_threshold(cpu_usage, threshold);
                if cpu_usage >= threshold {
                    CpuState::RisingEdge
                } else {
                    CpuState::Initial
                }
            }
            CpuState::RisingEdge => {
                if cpu_usage >= threshold {
                    above_threshold_count += 1;
                    if above_threshold_count >= 1 {
                        log("CPU usage has risen above the threshold.");
                        CpuState::OverThreshold
                    } else {
                        CpuState::RisingEdge
                    }
                } else {
                    above_threshold_count = 0;
                    CpuState::Initial
                }
            }
            CpuState::OverThreshold => {
                if cpu_usage < threshold {
                    below_threshold_count += 1;
                    if below_threshold_count >= 2 {
                        log("CPU usage has fallen below the threshold.");
                        CpuState::FallingEdge
                    } else {
                        CpuState::OverThreshold
                    }
                } else {
                    below_threshold_count = 0;
                    CpuState::OverThreshold
                }
            }
            CpuState::FallingEdge => {
                log_below_threshold(cpu_usage);
                log("CPU usage below threshold! Playing alert sound.");

                // Play the alert sound up to 5 times, interrupt if CPU goes above threshold
                for _ in 0..5 {
                    play_sound(alert_sound_path, stream_handle)?;
                    sleep(Duration::from_secs(1));

                    // Refresh CPU data and check if it goes above the threshold
                    sys.refresh_cpu();
                    let cpu_usage = sys.global_cpu_usage();
                    if cpu_usage >= threshold {
                        log_above_threshold(cpu_usage);
                    } else {
                        log_below_threshold(cpu_usage);
                    }
                }
                CpuState::BelowThreshold
            }
            CpuState::BelowThreshold => {
                if cpu_usage >= threshold {
                    CpuState::RisingEdge
                } else {
                    CpuState::BelowThreshold
                }
            }
        };

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

/// Logs a message with the current datetime in ISO-8601 format.
fn log(message: &str) {
    let dtnow = chrono::Local::now();
    println!("[{}] {}", dtnow.format("%Y-%m-%d %H:%M:%S"), message);
}

fn log_above_threshold(cpu_usage: f32) {
    log(&format!("Current CPU Usage: {:.2}% (above threshold)", cpu_usage));
}

fn log_below_threshold(cpu_usage: f32) {
    log(&format!("Current CPU Usage: {:.2}%", cpu_usage));
}

/// Log the CPU usage as above threshold or not.
/// Return true if it's above the threshold; false otherwise.
fn log_as_per_threshold(cpu_usage: f32, threshold: f32) -> bool {
    if cpu_usage >= threshold {
        log_above_threshold(cpu_usage);
        true
    } else {
        log_below_threshold(cpu_usage);
        false
    }
}
