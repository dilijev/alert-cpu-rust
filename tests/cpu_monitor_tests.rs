use std::time::Duration;
use std::thread::sleep;
use alert_cpu::{CpuMonitor, monitor_cpu};
use rodio::{OutputStream, Sink};

struct MockCpuMonitor {
    usage_pattern: Vec<f32>,
    index: usize,
}

impl MockCpuMonitor {
    fn new(usage_pattern: Vec<f32>) -> Self {
        MockCpuMonitor {
            usage_pattern,
            index: 0,
        }
    }
}

impl CpuMonitor for MockCpuMonitor {
    fn refresh_cpu(&mut self) {
        if self.index < self.usage_pattern.len() - 1 {
            self.index += 1;
        }
    }

    fn global_cpu_usage(&self) -> f32 {
        self.usage_pattern[self.index]
    }
}

#[test]
fn test_cpu_monitor() {
    let usage_pattern = vec![25.0, 30.0, 15.0, 10.0, 5.0, 25.0, 30.0];
    let mut mock_monitor = MockCpuMonitor::new(usage_pattern);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    monitor_cpu(&mut mock_monitor, 20.0, "alert.wav", &stream_handle);

    // Add assertions as needed to verify the state transitions and behavior
}
