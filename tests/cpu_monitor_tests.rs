use alert_cpu::{CpuMonitor, evolve_cpu_state, CpuMonitorState, CpuMonitorArgs, Settings};

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
        let cpu_usage_value = self.usage_pattern[self.index];
        if self.index < self.usage_pattern.len() - 1 {
            self.index += 1;
        }
        cpu_usage_value
    }
}

/// Basic test of state transitions.
#[test]
fn test_evolve_cpu_state_basics() {
    let usage_pattern = vec![5.0, 25.0, 30.0, 15.0, 10.0, 5.0, 25.0, 30.0];
    let mut mock_monitor = MockCpuMonitor::new(usage_pattern);

    let state_pattern = vec![
        CpuMonitorState::BelowThreshold,
        CpuMonitorState::RisingEdge,
        CpuMonitorState::OverThreshold,
        CpuMonitorState::FallingEdge,
        CpuMonitorState::BelowThreshold,
        CpuMonitorState::BelowThreshold,
        CpuMonitorState::RisingEdge,
        CpuMonitorState::OverThreshold,
    ];

    let settings = Settings {
        threshold: 20.0,
        debounce_count: 2,
        alert_repeat_count: 5,
    };

    let mut args = CpuMonitorArgs {
        above_threshold_count: 0,
        below_threshold_count: 0,
        alert_repeat_count: 0,
    };

    let mut state = CpuMonitorState::Initial;
    println!("State: {:?}", state);

    for i in 0..mock_monitor.usage_pattern.len() {
        let (next_state, _play_alert, cpu_usage, _display_log) =
            evolve_cpu_state(
                &mut mock_monitor,
                state,
                &settings,
                &mut args);

        println!("{:?} \t -> {:2} -> \t {:?} \t  (^{:2} _{:2} #{:2})", state, cpu_usage, next_state, args.above_threshold_count, args.below_threshold_count, args.alert_repeat_count);

        state = next_state;
        assert_eq!(next_state, state_pattern[i]);
    }
}

/// Stay longer in the OverThreshold and BelowThreshold states.
#[test]
fn test_evolve_cpu_state_longer_stay() {
    let usage_pattern = vec![5.0, 25.0, 30.0, 35.0, 15.0, 10.0, 5.0, 8.0, 25.0, 30.0, 35.0];
    let mut mock_monitor = MockCpuMonitor::new(usage_pattern);

    let state_pattern = vec![
        CpuMonitorState::BelowThreshold,
        CpuMonitorState::RisingEdge,
        CpuMonitorState::OverThreshold,
        CpuMonitorState::OverThreshold,
        CpuMonitorState::FallingEdge,
        CpuMonitorState::BelowThreshold,
        CpuMonitorState::BelowThreshold,
        CpuMonitorState::BelowThreshold,
        CpuMonitorState::RisingEdge,
        CpuMonitorState::OverThreshold,
        CpuMonitorState::OverThreshold,
    ];

    let settings = Settings {
        threshold: 20.0,
        debounce_count: 2,
        alert_repeat_count: 5,
    };

    let mut args = CpuMonitorArgs {
        above_threshold_count: 0,
        below_threshold_count: 0,
        alert_repeat_count: 0,
    };

    let mut state = CpuMonitorState::Initial;
    println!("State: {:?}", state);

    for i in 0..mock_monitor.usage_pattern.len() {
        let (next_state, _play_alert, cpu_usage, _display_log) =
            evolve_cpu_state(
                &mut mock_monitor,
                state,
                &settings,
                &mut args);

        println!("{:?} \t -> {:2} -> \t {:?} \t  (^{:2} _{:2} #{:2})", state, cpu_usage, next_state, args.above_threshold_count, args.below_threshold_count, args.alert_repeat_count);

        state = next_state;
        assert_eq!(next_state, state_pattern[i]);
    }
}

/// Test that alerts are played for 5 intervals after transitioning to BelowThreshold.
#[test]
fn test_alerts_played_for_5_intervals() {
    let usage_pattern = vec![25.0, 25.0, 15.0, 10.0, 5.0, 5.0, 5.0, 5.0, 25.0];
    let mut mock_monitor = MockCpuMonitor::new(usage_pattern);

    let state_pattern = vec![
        CpuMonitorState::RisingEdge,
        CpuMonitorState::OverThreshold,
        CpuMonitorState::FallingEdge,
        CpuMonitorState::BelowThreshold,
        CpuMonitorState::BelowThreshold,
        CpuMonitorState::BelowThreshold,
        CpuMonitorState::BelowThreshold,
        CpuMonitorState::BelowThreshold,
        CpuMonitorState::RisingEdge,
    ];

    let settings = Settings {
        threshold: 20.0,
        debounce_count: 2,
        alert_repeat_count: 5,
    };

    let mut args = CpuMonitorArgs {
        above_threshold_count: 0,
        below_threshold_count: 0,
        alert_repeat_count: 0,
    };

    let mut state = CpuMonitorState::Initial;
    println!("State: {:?}", state);

    for i in 0..mock_monitor.usage_pattern.len() {
        let (next_state, play_alert, cpu_usage, _display_log) =
            evolve_cpu_state(
                &mut mock_monitor,
                state,
                &settings,
                &mut args);

        println!("{:?} \t -> {:2} -> \t {:?} \t  (^{:2} _{:2} #{:2}){}",
            state,
            cpu_usage,
            next_state,
            args.above_threshold_count,
            args.below_threshold_count,
            args.alert_repeat_count,
            if play_alert {" !!!"} else {""});

        state = next_state;
        assert_eq!(next_state, state_pattern[i]);

        if state == CpuMonitorState::BelowThreshold {
            assert!(play_alert);
        }
    }

    // Ensure that alerts are played for 5 intervals
    assert_eq!(args.alert_repeat_count, 5);
}
