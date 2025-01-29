use sysinfo::{System, CpuRefreshKind};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CpuMonitorState {
    Initial,
    RisingEdge,
    OverThreshold,
    FallingEdge,
    BelowThreshold,
}

pub trait CpuMonitor {
    fn get_cpu_usage(&mut self) -> f32;
}

impl CpuMonitor for System {
    fn get_cpu_usage(&mut self) -> f32 {
        self.refresh_cpu_specifics(CpuRefreshKind::everything());
        self.global_cpu_usage()
    }
}

const DEBOUNCE_COUNT: i32 = 2;
const ALERT_REPEAT_COUNT: i32 = 5;

pub fn evolve_cpu_state<T: CpuMonitor>(
    sys: &mut T,
    current_state: CpuMonitorState,
    threshold: f32,
    above_threshold_count: &mut i32,
    below_threshold_count: &mut i32,
    alert_repeat_count: &mut i32,
) -> (CpuMonitorState, bool, f32, bool) {
    let cpu_usage = sys.get_cpu_usage();
    let next_state;
    let mut play_alert = false;
    let mut display_log = false;

    // Make these updates regardless of current state
    if cpu_usage > threshold {
        // above
        *below_threshold_count = 0;
        *above_threshold_count += 1;
    } else {
        // below
        if *below_threshold_count == 0 {
            *alert_repeat_count = 0;
        }
        *above_threshold_count = 0;
        *below_threshold_count += 1;
    }

    match current_state {
        CpuMonitorState::Initial => {
            if cpu_usage > threshold {
                if *above_threshold_count >= DEBOUNCE_COUNT {
                    next_state = CpuMonitorState::OverThreshold;
                } else {
                    next_state = CpuMonitorState::RisingEdge;
                }
            } else {
                next_state = CpuMonitorState::BelowThreshold;
            }
            display_log = true;
        }
        CpuMonitorState::RisingEdge => {
            if cpu_usage > threshold {
                if *above_threshold_count >= DEBOUNCE_COUNT {
                    next_state = CpuMonitorState::OverThreshold;
                } else {
                    next_state = CpuMonitorState::RisingEdge;
                }
            } else {
                next_state = CpuMonitorState::BelowThreshold;
            }
            display_log = true;
        }
        CpuMonitorState::OverThreshold => {
            if cpu_usage <= threshold {
                if *below_threshold_count >= DEBOUNCE_COUNT {
                    next_state = CpuMonitorState::BelowThreshold;
                } else {
                    next_state = CpuMonitorState::FallingEdge;
                }
            } else {
                next_state = CpuMonitorState::OverThreshold;
            }
        }
        CpuMonitorState::FallingEdge => {
            if cpu_usage <= threshold {
                if *below_threshold_count >= DEBOUNCE_COUNT {
                    next_state = CpuMonitorState::BelowThreshold;
                } else {
                    next_state = CpuMonitorState::FallingEdge;
                }
            } else {
                next_state = CpuMonitorState::OverThreshold;
            }
        }
        CpuMonitorState::BelowThreshold => {
            if cpu_usage > threshold {
                if *above_threshold_count >= DEBOUNCE_COUNT {
                    next_state = CpuMonitorState::OverThreshold;
                } else {
                    next_state = CpuMonitorState::RisingEdge;
                }
            } else {
                next_state = CpuMonitorState::BelowThreshold;

                // Play alerts if applicable.
                if *alert_repeat_count < ALERT_REPEAT_COUNT {
                    play_alert = true;
                }
                *alert_repeat_count += 1;
            }
        }
    }

    (next_state, play_alert, cpu_usage, display_log)
}
