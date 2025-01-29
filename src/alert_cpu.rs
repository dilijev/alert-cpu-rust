use sysinfo::{System, CpuRefreshKind};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CpuState {
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
    current_state: CpuState,
    threshold: f32,
    above_threshold_count: &mut i32,
    below_threshold_count: &mut i32,
    alert_repeat_count: &mut i32,
) -> (CpuState, bool, f32, bool) {
    let cpu_usage = sys.get_cpu_usage();
    let mut next_state = current_state;
    let mut play_alert = false;
    let mut display_log = false;

    match current_state {
        CpuState::Initial => {
            if cpu_usage > threshold {
                *above_threshold_count += 1;
                if *above_threshold_count >= DEBOUNCE_COUNT {
                    next_state = CpuState::OverThreshold;
                } else {
                    next_state = CpuState::RisingEdge;
                }
            } else {
                next_state = CpuState::BelowThreshold;
            }
            display_log = true;
        }
        CpuState::RisingEdge => {
            if cpu_usage > threshold {
                *above_threshold_count += 1;
                if *above_threshold_count >= DEBOUNCE_COUNT {
                    next_state = CpuState::OverThreshold;
                } else {
                    next_state = CpuState::RisingEdge;
                }
            } else {
                *above_threshold_count = 0;
                next_state = CpuState::BelowThreshold;
            }
            display_log = true;
        }
        CpuState::OverThreshold => {
            if cpu_usage <= threshold {
                *below_threshold_count += 1;
                if *below_threshold_count >= DEBOUNCE_COUNT {
                    next_state = CpuState::BelowThreshold;
                } else {
                    next_state = CpuState::FallingEdge;
                }
            }
        }
        CpuState::FallingEdge => {
            if cpu_usage <= threshold {
                *below_threshold_count += 1;
                if *below_threshold_count >= DEBOUNCE_COUNT {
                    next_state = CpuState::BelowThreshold;
                    play_alert = true;
                }
            } else {
                *below_threshold_count = 0;
                next_state = CpuState::OverThreshold;
            }
            display_log = true;
        }
        CpuState::BelowThreshold => {
            if cpu_usage > threshold {
                *alert_repeat_count = 0;
                *above_threshold_count += 1;
                if *above_threshold_count >= DEBOUNCE_COUNT {
                    next_state = CpuState::OverThreshold;
                } else {
                    next_state = CpuState::RisingEdge;
                }
                display_log = true;
            } else {
                next_state = CpuState::BelowThreshold;
                if (*alert_repeat_count) < ALERT_REPEAT_COUNT {
                    play_alert = true;
                }
                *alert_repeat_count += 1;
            }
        }
    }

    (next_state, play_alert, cpu_usage, display_log)
}
