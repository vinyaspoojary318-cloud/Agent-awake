use sysinfo::System;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalState {
    Normal,
    Elevated,
    Critical,
}

impl ThermalState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThermalState::Normal => "Normal",
            ThermalState::Elevated => "Elevated",
            ThermalState::Critical => "Critical",
        }
    }
}

/// Estimate thermal pressure on Windows using CPU load as a proxy.
/// (Direct thermal zone APIs are limited / OEM-specific.)
pub fn current_state(cpu_threshold: f32) -> ThermalState {
    let mut sys = System::new();
    sys.refresh_cpu_usage();
    // Need a short settle for accurate reading
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_usage();

    let usage = sys.global_cpu_info().cpu_usage();

    if usage >= cpu_threshold {
        ThermalState::Critical
    } else if usage >= cpu_threshold * 0.75 {
        ThermalState::Elevated
    } else {
        ThermalState::Normal
    }
}

pub fn cpu_usage() -> f32 {
    let mut sys = System::new();
    sys.refresh_cpu_usage();
    std::thread::sleep(std::time::Duration::from_millis(150));
    sys.refresh_cpu_usage();
    sys.global_cpu_info().cpu_usage()
}
