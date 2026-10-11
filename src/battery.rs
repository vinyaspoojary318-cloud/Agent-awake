use sysinfo::System;

pub fn get_battery_info() -> String {
    // Note: sysinfo battery support is limited on some systems.
    // We will improve this later with windows-specific APIs.

    let mut sys = System::new();
    sys.refresh_all();

    // For now return basic system info + placeholder for battery
    // Full battery percentage will be added next using Windows API.

    format!(
        "Battery: (detailed reading coming soon)\n\
CPU Usage: {:.1}%\n\
Memory: {:.1} GB used",
        sys.global_cpu_info().cpu_usage(),
        sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0
    )
}
