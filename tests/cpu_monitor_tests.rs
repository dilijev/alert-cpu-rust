use std::time::Duration;
use std::thread::sleep;
use alert_cpu::{CpuMonitor, evolve_cpu_state, CpuState};
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
    fn get_cpu_usage(&mut self) -> f32 {
        if self.index < self.usage_pattern.len() - 1 {
            self.index += 1;
        }
        self.usage_pattern[self.index]
    }
}

#[test]
fn test_evolve_cpu_state() {
    let usage_pattern = vec![25.0, 30.0, 15.0, 10.0, 5.0, 25.0, 30.0];
    let mut mock_monitor = MockCpuMonitor::new(usage_pattern);

    let threshold = 20.0;
    let mut above_threshold_count = 0;
    let mut below_threshold_count = 0;

    let mut state = CpuState::Initial;

    for _ in 0..mock_monitor.usage_pattern.len() {
        let (next_state, play_alert, cpu_usage, display_log) =
            evolve_cpu_state(
                &mut mock_monitor,
                state,
                threshold,
                &mut above_threshold_count,
                &mut below_threshold_count);

        println!("State: {:?}", next_state);
        println!("Play Alert: {:?}", play_alert);
        println!("CPU Usage: {:?}", cpu_usage);
        println!("Display Log: {:?}", display_log);
        println!();

        state = next_state;
    }

    // Add assertions as needed to verify the state transitions and behavior
    assert_eq!(state, CpuState::RisingEdge);
}
