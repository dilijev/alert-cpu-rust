# alert-cpu

When long-running operations complete, the user may not be aware right away
unless they're watching the output of the process or monitoring a CPU usage
graph. However, the user might still be at the computer and able to notice an
audible alert. This application will wait for the CPU to be above a certain
threshold and then play an alert sound when it drops and stays below that
threshold, indicating that the long-running operation has completed.

## Build and Install

Assuming you have Rust installed and on the path.

### Build

To build the application (debug configuration):

```sh
cargo build
```

- On Windows, the output is located at `.\target\debug\alert-cpu.exe`.
- On Unix-like systems, the output is located at `./target/debug/alert-cpu`.

In the example commands below, I will simply use `alert-cpu` for brevity. The
user will need to use the appropriate path for the executable and alert sound
file.

### Install

To install the application:

```sh
cargo install --path .
```

The `--path .` option tells Cargo to install the application from the current
directory.

Once installed, you can run the application from anywhere.

## Usage

```sh
alert-cpu [threshold [sound]]
    threshold: The CPU threshold to wait for in % of total CPU usage.
               [Defaults to 20 (20%)]
    sound:     The path (relative or absolute) to the alert sound file.
               [Defaults to "alert.wav"]
```

### Examples

With defaults (20% CPU, alert.wav):

```sh
alert-cpu
```

With defaults made explicit on the command line:

```sh
alert-cpu 20 alert.wav
```

Alternative values:

```sh
alert-cpu 30.5 assets/notify.wav
```

The first argument is a value between 0 and 100 that represents the CPU
threshold in % total CPU usage. Default is 20%.

The second argument is a path (relative or absolute) to the alert sound file
you would like to play. Default is `"alert.wav"`.

## Supported Alert Sound Formats

Recommend a sound shorter than 1 second. This has not been tested with longer sounds.

This project uses the `rodio` crate for audio playback. `rodio` supports
various audio formats including `.wav`, `.mp3`, `.flac`, and `.vorbis`.

A suggestion for an alert sound to play on Windows:

```sh
C:\Windows\Media\Windows Notify.wav
```

On Unix-like systems, system sounds are typically in `.ogg` or `.mp3` format,
but `.wav` files are also supported.

## Example Output

```sh
alert-cpu-rust> .\target\debug\alert-cpu.exe 15 .\alert.wav
[2025-01-28 17:09:38] CPU Threshold: 15%
[2025-01-28 17:09:38] Alert sound path: ".\alert.wav"
[2025-01-28 17:09:38] Monitoring CPU usage. Alert will sound if usage drops below 15%.
[2025-01-28 17:09:38] Press Ctrl+C to exit.
[2025-01-28 17:09:38] Current CPU Usage: 24.15% (above threshold)
[2025-01-28 17:09:39] Current CPU Usage: 8.33%
[2025-01-28 17:09:39] CPU usage below threshold! Playing alert sound.
[2025-01-28 17:09:49] Current CPU Usage: 6.13%
[2025-01-28 17:09:59] Current CPU Usage: 6.19%
```
