use crate::battery;

/// Battery health estimate + adaptive cutoff suggestion.
#[derive(Debug, Clone)]
pub struct BatteryHealth {
    pub percent: Option<u8>,
    pub on_ac: bool,
    /// 0.0–1.0 rough health score (1.0 = healthy). Unknown => 1.0
    pub health_score: f32,
    pub note: String,
}

pub fn assess() -> BatteryHealth {
    let percent = battery::get_battery_percent();
    let info = battery::get_battery_info();
    let on_ac = info.to_lowercase().contains("plugged");

    // Windows does not expose design vs full-charge capacity easily without WMI.
    // We use a conservative heuristic: very low % while plugged may indicate issues,
    // but default to healthy.
    let health_score = 1.0_f32;
    let note = if on_ac {
        "On AC — good for long agent runs".into()
    } else {
        match percent {
            Some(p) if p < 20 => "On battery and low — prefer shorter runs".into(),
            Some(p) if p < 40 => "On battery — monitor temperature and duration".into(),
            _ => "Battery level OK for agent work".into(),
        }
    };

    BatteryHealth {
        percent,
        on_ac,
        health_score,
        note,
    }
}

/// Raise the effective cutoff when health is poor or on battery for longevity.
pub fn effective_cutoff(base_cutoff: u8, adaptive: bool) -> u8 {
    if !adaptive {
        return base_cutoff;
    }
    let h = assess();
    // Prefer not draining below a higher floor when on battery
    if !h.on_ac {
        base_cutoff.saturating_add(5).min(40)
    } else {
        base_cutoff
    }
}

pub fn summary() -> String {
    let h = assess();
    format!(
        "{}\nHealth note: {}\nAdaptive context: {}",
        battery::get_battery_info(),
        h.note,
        if h.on_ac { "plugged in" } else { "on battery" }
    )
}
