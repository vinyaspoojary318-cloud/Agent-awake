use anyhow::{Result, anyhow};
use log::info;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_AWAKE: AtomicBool = AtomicBool::new(false);

/// Keep the system awake or restore normal behavior.
/// Uses SetThreadExecutionState which is simple and reliable on Windows.
pub fn keep_awake(enable: bool) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Threading::{
            SetThreadExecutionState, ES_CONTINUOUS, ES_SYSTEM_REQUIRED,
        };

        if enable {
            // Keep system awake (display is allowed to turn off to save power)
            let flags = ES_CONTINUOUS | ES_SYSTEM_REQUIRED;
            let result = unsafe { SetThreadExecutionState(flags) };

            if result.0 == 0 {
                return Err(anyhow!("SetThreadExecutionState failed"));
            }

            IS_AWAKE.store(true, Ordering::SeqCst);
            info!("Keep-awake ENABLED");
        } else {
            // Clear the request → normal sleep behavior returns
            let result = unsafe { SetThreadExecutionState(ES_CONTINUOUS) };
            if result.0 == 0 {
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
