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

pub struct Settings {
    pub threshold: f32,
    pub debounce_count: i32,
    pub alert_repeat_count: i32,
}

pub struct CpuMonitorArgs {
    pub above_threshold_count: i32,
    pub below_threshold_count: i32,
    pub alert_repeat_count: i32,
}

pub struct CpuMonitorOutput {
    pub next_state: CpuMonitorState,
    pub cpu_usage: f32,
    pub play_alert: bool,
    pub display_log: bool,
}

pub fn evolve_cpu_state<T: CpuMonitor>(
    sys: &mut T,
    current_state: CpuMonitorState,
    settings: &Settings,
    args: &mut CpuMonitorArgs,
) -> CpuMonitorOutput {
    let cpu_usage = sys.get_cpu_usage();
    let next_state;
    let mut play_alert = false;
    let mut display_log = false;

    // Make these updates regardless of current state
    if cpu_usage > settings.threshold {
        // above
        args.below_threshold_count = 0;
        args.above_threshold_count += 1;
    } else {
        // below
        if args.below_threshold_count == 0 {
            args.alert_repeat_count = 0;
        }
        args.above_threshold_count = 0;
        args.below_threshold_count += 1;
    }

    match current_state {
        CpuMonitorState::Initial => {
            if cpu_usage > settings.threshold {
                if args.above_threshold_count >= settings.debounce_count {
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
            if cpu_usage > settings.threshold {
                if args.above_threshold_count >= settings.debounce_count {
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
            if cpu_usage <= settings.threshold {
                if args.below_threshold_count >= settings.debounce_count {
                    next_state = CpuMonitorState::BelowThreshold;
                } else {
                    next_state = CpuMonitorState::FallingEdge;
                }
            } else {
                next_state = CpuMonitorState::OverThreshold;
            }
        }
        CpuMonitorState::FallingEdge => {
            if cpu_usage <= settings.threshold {
                if args.below_threshold_count >= settings.debounce_count {
                    next_state = CpuMonitorState::BelowThreshold;
                } else {
                    next_state = CpuMonitorState::FallingEdge;
                }
            } else {
                next_state = CpuMonitorState::OverThreshold;
            }
        }
        CpuMonitorState::BelowThreshold => {
            if cpu_usage > settings.threshold {
                if args.above_threshold_count >= settings.debounce_count {
                    next_state = CpuMonitorState::OverThreshold;
                } else {
                    next_state = CpuMonitorState::RisingEdge;
                }
            } else {
                next_state = CpuMonitorState::BelowThreshold;

                // Play alerts if applicable.
                if args.alert_repeat_count < settings.alert_repeat_count {
                    play_alert = true;
                }
                args.alert_repeat_count += 1;
            }
        }
    }

    CpuMonitorOutput {
        next_state,
        cpu_usage,
        play_alert,
        display_log,
    }
}
