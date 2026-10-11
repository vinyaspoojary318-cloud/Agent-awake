use log::warn;

/// Returns a human-readable battery status string.
pub fn get_battery_info() -> String {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

        let mut status = SYSTEM_POWER_STATUS::default();

        // windows-rs 0.58: GetSystemPowerStatus returns Result<()>
        let result = unsafe { GetSystemPowerStatus(&mut status) };

        if result.is_ok() {
            let percent = status.BatteryLifePercent;

            // 255 means "unknown"
            if percent == 255 {
                return "Battery: Unknown".to_string();
            }

            let ac_line = match status.ACLineStatus {
                0 => "On Battery",
                1 => "Plugged In",
                _ => "Unknown Power Source",
            };

            let charging = if status.BatteryFlag & 8 != 0 {
                " (Charging)"
            } else {
                ""
            };

            let remaining = if status.BatteryLifeTime != u32::MAX && status.BatteryLifeTime > 0 {
                let mins = status.BatteryLifeTime / 60;
                format!(" • ~{}m remaining", mins)
            } else {
                String::new()
            };

            return format!(
                "Battery: {}% {}{}{}",
                percent, ac_line, charging, remaining
            );
        } else {
            warn!("GetSystemPowerStatus failed");
            return "Battery: Unable to read".to_string();
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        "Battery: Only supported on Windows".to_string()
    }
}

/// Returns battery percentage (0-100) or None if unknown.
pub fn get_battery_percent() -> Option<u8> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

        let mut status = SYSTEM_POWER_STATUS::default();
        let result = unsafe { GetSystemPowerStatus(&mut status) };

        if result.is_ok() {
            let percent = status.BatteryLifePercent;
            if percent <= 100 {
                return Some(percent);
            }
        }
    }
    None
}
