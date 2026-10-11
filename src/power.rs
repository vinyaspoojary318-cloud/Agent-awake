use anyhow::{Result, anyhow};
use log::info;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_AWAKE: AtomicBool = AtomicBool::new(false);

// Use raw FFI for SetThreadExecutionState — more reliable across windows-rs versions.
#[cfg(target_os = "windows")]
mod win_power {
    pub const ES_SYSTEM_REQUIRED: u32 = 0x0000_0001;
    pub const ES_CONTINUOUS: u32 = 0x8000_0000;

    #[link(name = "kernel32")]
    extern "system" {
        pub fn SetThreadExecutionState(es_flags: u32) -> u32;
    }
}

pub fn keep_awake(enable: bool) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use win_power::*;

        if enable {
            let flags = ES_CONTINUOUS | ES_SYSTEM_REQUIRED;
            let result = unsafe { SetThreadExecutionState(flags) };
            if result == 0 {
                return Err(anyhow!("SetThreadExecutionState failed"));
            }
            IS_AWAKE.store(true, Ordering::SeqCst);
            info!("Keep-awake ENABLED");
        } else {
            let result = unsafe { SetThreadExecutionState(ES_CONTINUOUS) };
            if result == 0 {
                return Err(anyhow!("Failed to clear keep-awake"));
            }
            IS_AWAKE.store(false, Ordering::SeqCst);
            info!("Keep-awake DISABLED");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = enable;
        return Err(anyhow!("Keep-awake is currently only implemented for Windows"));
    }

    Ok(())
}

pub fn is_awake() -> bool {
    IS_AWAKE.load(Ordering::SeqCst)
}
