use alert_cpu::{CpuMonitor, evolve_cpu_state, CpuMonitorState, CpuMonitorArgs, Settings, CpuMonitorOutput};

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
        let CpuMonitorOutput {
            next_state,
            cpu_usage,
            play_alert: _,
            display_log: _,
        } = evolve_cpu_state(&mut mock_monitor, state, &settings, &mut args);

        println!("{:?} \t -> {:2} -> \t {:?} \t  (^{:2} _{:2} #{:2})", state, cpu_usage, next_state, args.above_threshold_count, args.below_threshold_count, args.alert_repeat_count);

        state = next_state;
        assert_eq!(next_state, state_pattern[i]);
    }
}

/// Streaks must be mutually exclusive. One must be 0 while the other is non-0.
#[test]
fn test_mutually_exclusive_streaks() {
    let usage_pattern = vec![5.0, 25.0, 30.0, 15.0, 10.0, 5.0, 25.0, 30.0];
    let mut mock_monitor = MockCpuMonitor::new(usage_pattern);

    let _state_pattern = vec![
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

    for _i in 0..mock_monitor.usage_pattern.len() {
        let CpuMonitorOutput {
            next_state,
            cpu_usage,
            play_alert: _,
            display_log: _,
        } = evolve_cpu_state(&mut mock_monitor, state, &settings, &mut args);

        println!("{:?} \t -> {:2} -> \t {:?} \t  (^{:2} _{:2} #{:2})", state, cpu_usage, next_state, args.above_threshold_count, args.below_threshold_count, args.alert_repeat_count);

        // Streaks above or below must be mutually exclusive.
        // If one is 0 the other is non-zero.
        // Neither can be 0 at the same time after the first state change.
        assert!(args.above_threshold_count + args.below_threshold_count != 0);
        assert!(args.above_threshold_count == 0 || args.below_threshold_count == 0);
        assert!(args.above_threshold_count > 0 || args.below_threshold_count > 0);
        assert_ne!(args.above_threshold_count, args.below_threshold_count);
        if args.above_threshold_count > 0 {
            assert_eq!(args.below_threshold_count, 0);
        } else if args.below_threshold_count > 0 {
            assert_eq!(args.above_threshold_count, 0);
        } else {
            // There must be no other options.
            assert!(false, "Invalid state.");
        }

        state = next_state;
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
        let CpuMonitorOutput {
            next_state,
            cpu_usage,
            play_alert: _,
            display_log: _,
        } = evolve_cpu_state(&mut mock_monitor, state, &settings, &mut args);

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
        let CpuMonitorOutput {
            next_state,
            cpu_usage,
            play_alert,
            display_log: _,
        } = evolve_cpu_state(&mut mock_monitor, state, &settings, &mut args);

        println!("{:?} \t -> {:2} -> \t {:?} \t  (^{:2} _{:2} #{:2}){}",
            state,
            cpu_usage,
            next_state,
            args.above_threshold_count,
            args.below_threshold_count,
            args.alert_repeat_count,
            if play_alert {" !!!"} else {" ---"});

        state = next_state;
        assert_eq!(next_state, state_pattern[i]);

        if state == CpuMonitorState::BelowThreshold {
            assert!(play_alert);
        }
    }

    // Ensure that alerts are played for 5 intervals
    assert_eq!(args.alert_repeat_count, 5);
}
