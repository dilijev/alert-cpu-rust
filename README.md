When long-running operations complete, the user may not be aware right away
unless they're watching the output of the process or monitoring a CPU usage
graph, however the user might still be at the computer and able to notice an
audible alert. This application will wait for the CPU to be above a certain
threshold and then play an alert sound when it drops and stays below that
threshold, indicating that the long-running operation has completed.

# Build

Assuming you have rust installed and on the path.

```
cargo install
cargo build
```

On Windows the output is located at `.\target\debug\alert-cpu.exe`

# Usage

```
alert-cpu [threshold [sound]]
    threshold: The CPU threshold to wait for in % of total CPU usage.
               [Defaults to 20 (20%)]
    sound:     The path (relative or absolute) to the alert sound file.
               [Defaults to "alert.wav"]
```

## Examples:

With defaults (20% CPU, alert.wav):

```
.\target\debug\alert-cpu.exe
```

With defaults made explicit on the command line:

```
.\target\debug\alert-cpu.exe 20 alert.wav
```

Alternative values:

```
.\target\debug\alert-cpu.exe 30.5 assets\notify.wav
```

The first argument is a value between 0 and 100 that represents the CPU
threshold in % total CPU usage. Default is 20%

The second argument is a path (relative or absolute) to the alert sound file
you would like to play (tested with .wav files).

A suggestion for an alert sound to play on Windows:

```
C:\Windows\Media\Windows Notify.wav
```

# Example Output

```
alert-cpu-rust> .\target\debug\alert-cpu.exe 15 .\alert.wav
[2025-01-28 17:09:38] CPU Threshold: 15%
[2025-01-28 17:09:38] Alert sound path: ".\alert.wav"
[2025-01-28 17:09:38] Monitoring CPU usage. Alert will sound if usage drops below 15%.
[2025-01-28 17:09:38] Press Ctrl+C to exit.
[2025-01-28 17:09:38] Current CPU Usage: 24.15% (above threshold)
[2025-01-28 17:09:39] Current CPU Usage: 8.33% (below threshold)
[2025-01-28 17:09:39] CPU usage below threshold! Playing alert sound.
[2025-01-28 17:09:49] Current CPU Usage: 6.13% (below threshold)
[2025-01-28 17:09:59] Current CPU Usage: 6.19% (below threshold)
```

