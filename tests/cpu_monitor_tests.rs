use alert_cpu::{CpuMonitor, evolve_cpu_state, CpuState};

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
    let usage_pattern = vec![5.0, 25.0, 30.0, 15.0, 10.0, 5.0, 25.0, 30.0];
    let mut mock_monitor = MockCpuMonitor::new(usage_pattern);

    let state_pattern = vec![
        CpuState::BelowThreshold,
        CpuState::RisingEdge,
        CpuState::OverThreshold,
        CpuState::FallingEdge,
        CpuState::BelowThreshold,
        CpuState::BelowThreshold,
        CpuState::RisingEdge,
        CpuState::OverThreshold,
    ];

    let threshold = 20.0;
    let mut above_threshold_count = 0;
    let mut below_threshold_count = 0;

    let mut state = CpuState::Initial;
    println!("State: {:?}", state);

    for i in 0..mock_monitor.usage_pattern.len() {
        let (next_state, _play_alert, _cpu_usage, _display_log) =
            evolve_cpu_state(
                &mut mock_monitor,
                state,
                threshold,
                &mut above_threshold_count,
                &mut below_threshold_count);

        println!("State: {:?}, Next State: {:?}", state, next_state);

        state = next_state;
        assert_eq!(next_state, state_pattern[i]);
    }

    // Add assertions as needed to verify the state transitions and behavior
    assert_eq!(state, CpuState::RisingEdge);
}
