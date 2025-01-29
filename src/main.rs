use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use std::thread::sleep;

use sysinfo::{System, CpuRefreshKind};
use rodio::{Decoder, OutputStream, Sink};

fn main() {
    // Define the CPU usage threshold (e.g., 10%)
    let threshold = 10.0;

    // Path to the alert sound file
    let alert_sound_path = "alert.wav";

    // Initialize the system information struct
    let mut sys = System::new_all();

    // Initialize the audio output stream
    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Error initializing audio output stream: {}", e);
            return;
        }
    };

    println!("Monitoring CPU usage. Alert will sound if usage drops below {}%.", threshold);
    println!("Press Ctrl+C to exit.");

    loop {
        // Refresh CPU data
        sys.refresh_cpu_specifics(CpuRefreshKind::everything());

        // Get the average CPU usage across all cores
        let cpu_usage = sys.global_cpu_usage();

        println!("Current CPU Usage: {:.2}%", cpu_usage);

        if cpu_usage < threshold {
            println!("CPU usage below threshold! Playing alert sound.");

            // Play the alert sound
            if let Err(e) = play_sound(&alert_sound_path, &stream_handle) {
                eprintln!("Error playing sound: {}", e);
            }

            // Optional: Wait until CPU usage rises above the threshold to avoid repeated alerts
            wait_until_above_threshold(&mut sys, threshold);
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
        sleep(Duration::from_secs(1));
        sys.refresh_cpu_specifics(CpuRefreshKind::everything());
        let cpu_usage = sys.global_cpu_usage();
        if cpu_usage >= threshold {
            println!("CPU usage has risen above the threshold.");
            break;
        }
    }
}
