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

pub fn evolve_cpu_state<T: CpuMonitor>(
  sys: &mut T,
  current_state: CpuState,
  threshold: f32,
  _above_threshold_count: &mut i32,
  _below_threshold_count: &mut i32,
) -> (CpuState, bool, f32, bool) {
  let cpu_usage = sys.get_cpu_usage();
  let mut next_state = current_state;
  let mut play_alert = false;
  let mut display_log = false;

  match current_state {
      CpuState::Initial => {
          if cpu_usage > threshold {
              next_state = CpuState::RisingEdge;
          } else {
              next_state = CpuState::BelowThreshold;
          }
          display_log = true;
      }
      CpuState::RisingEdge => {
          if cpu_usage > threshold {
              next_state = CpuState::OverThreshold;
          } else {
              next_state = CpuState::FallingEdge;
          }
          display_log = true;
      }
      CpuState::OverThreshold => {
          if cpu_usage <= threshold {
              next_state = CpuState::FallingEdge;
              display_log = true;
          }
      }
      CpuState::FallingEdge => {
          if cpu_usage <= threshold {
              next_state = CpuState::BelowThreshold;
              play_alert = true;
              display_log = true;
          } else {
              next_state = CpuState::OverThreshold;
          }
      }
      CpuState::BelowThreshold => {
          if cpu_usage > threshold {
              next_state = CpuState::RisingEdge;
              display_log = true;
          }
      }
  }

  (next_state, play_alert, cpu_usage, display_log)
}
