use log::info;
use std::sync::atomic::{AtomicBool, Ordering};

static BAG_MODE: AtomicBool = AtomicBool::new(false);

/// Bag mode = closed-lid friendly profile:
/// - Prefer local work
/// - Avoid unnecessary keep-awake for low-priority cloud-only agents when possible
/// - Flag for future firewall / data-saver integration
pub fn set_active(active: bool) {
    let prev = BAG_MODE.swap(active, Ordering::SeqCst);
    if prev != active {
        if active {
            info!("Bag mode ENABLED (network-light / thermal-aware profile)");
        } else {
            info!("Bag mode DISABLED");
        }
    }
}

pub fn is_active() -> bool {
    BAG_MODE.load(Ordering::SeqCst)
}

pub fn status_line() -> String {
    if is_active() {
        "Bag mode: ON (prefer local models, reduce non-essential wake)".into()
    } else {
        "Bag mode: OFF".into()
    }
}
