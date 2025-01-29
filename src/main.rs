use std::env;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use std::thread::sleep;
use chrono;

use sysinfo::{System, CpuRefreshKind};
use rodio::{Decoder, OutputStream, Sink};

#[derive(Copy, Clone, Debug)]
enum CpuState {
    Initial,
    RisingEdge,
    OverThreshold,
    FallingEdge,
    BelowThreshold,
}

trait CpuMonitor {
    fn get_cpu_usage(&mut self) -> f32;
}

impl CpuMonitor for System {
    fn get_cpu_usage(&mut self) -> f32 {
        self.refresh_cpu_specifics(CpuRefreshKind::everything());
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

    // Get the interval time from the command line arguments
    // or use the default value of 1 second.
    let interval: f64 = args.get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0);
    log(&format!("Interval: {} seconds", interval));

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

    monitor_cpu(&mut sys, threshold, interval, &alert_sound_path, &stream_handle)
        .unwrap_or_else(|e| log(&format!("Error playing alert sound: {}", e)));
}

fn monitor_cpu<T: CpuMonitor>(sys: &mut T, threshold: f32, interval: f64, alert_sound_path: &str, stream_handle: &rodio::OutputStreamHandle) -> Result<(), rodio::decoder::DecoderError> {
    log(&format!("Playing alert sound once on startup."));
    play_sound(alert_sound_path, stream_handle)?;

    log(&format!("Monitoring CPU usage. Alert will sound if usage drops below {}%.", threshold));
    log("Press Ctrl+C to exit.");

    let mut state = CpuState::Initial;
    let mut above_threshold_count = 0;
    let mut below_threshold_count = 0;

    // Each iteration of the loop should be 1 interval.
    loop {
        // Sleep for 1 interval before doing anything.
        sleep(Duration::from_secs_f64(interval));

        let (next_state, play_alert, cpu_usage, display_log) =
            evolve_cpu_state(
                sys,
                state,
                threshold,
                &mut above_threshold_count,
                &mut below_threshold_count);
        state = next_state;

        // Debug the state evolution
        println!("State: {:?}", state);
        println!("Above Threshold Count: {:?}", above_threshold_count);
        println!("Below Threshold Count: {:?}", below_threshold_count);
        println!("Play Alert: {:?}", play_alert);
        println!("CPU Usage: {:?}", cpu_usage);
        println!("Display Log: {:?}", display_log);
        log_as_per_threshold(cpu_usage, threshold);
        println!();
    }
}

fn evolve_cpu_state<T: CpuMonitor>(
    sys: &mut T,
    current_state: CpuState,
    threshold: f32,
    _above_threshold_count: &mut i32,
    _below_threshold_count: &mut i32,
) -> (CpuState, bool, f32, bool) {
    let cpu_usage = sys.get_cpu_usage();
    let mut next_state = current_state;
    let mut play_alert = false;
    let mut display_log = false;

    match current_state {
        CpuState::Initial => {
            if cpu_usage > threshold {
                next_state = CpuState::RisingEdge;
            } else {
                next_state = CpuState::BelowThreshold;
            }
            display_log = true;
        }
        CpuState::RisingEdge => {
            if cpu_usage > threshold {
                next_state = CpuState::OverThreshold;
            } else {
                next_state = CpuState::FallingEdge;
            }
            display_log = true;
        }
        CpuState::OverThreshold => {
            if cpu_usage <= threshold {
                next_state = CpuState::FallingEdge;
                display_log = true;
            }
        }
        CpuState::FallingEdge => {
            if cpu_usage <= threshold {
                next_state = CpuState::BelowThreshold;
                play_alert = true;
                display_log = true;
            } else {
                next_state = CpuState::OverThreshold;
            }
        }
        CpuState::BelowThreshold => {
            if cpu_usage > threshold {
                next_state = CpuState::RisingEdge;
                display_log = true;
            }
        }
    }

    (next_state, play_alert, cpu_usage, display_log)
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
