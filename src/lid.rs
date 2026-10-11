use anyhow::{Context, Result};
use log::{info, warn};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

static LID_OVERRIDDEN: AtomicBool = AtomicBool::new(false);

/// Set lid-close action to "Do nothing" (0) for both AC and battery.
/// Returns true if the change was applied.
pub fn set_do_nothing() -> Result<()> {
    // powercfg /setacvalueindex SCHEME_CURRENT SUB_BUTTONS LIDACTION 0
    // powercfg /setdcvalueindex SCHEME_CURRENT SUB_BUTTONS LIDACTION 0
    // powercfg /setactive SCHEME_CURRENT

    run_powercfg(&["/setacvalueindex", "SCHEME_CURRENT", "SUB_BUTTONS", "LIDACTION", "0"])?;
    run_powercfg(&["/setdcvalueindex", "SCHEME_CURRENT", "SUB_BUTTONS", "LIDACTION", "0"])?;
    run_powercfg(&["/setactive", "SCHEME_CURRENT"])?;

    LID_OVERRIDDEN.store(true, Ordering::SeqCst);
    info!("Lid-close action set to 'Do nothing'");
    Ok(())
}

/// Restore lid-close action to Sleep (1) — the common default.
/// Note: We currently restore to Sleep. A future version can save/restore the exact previous value.
pub fn restore_default() -> Result<()> {
    if !LID_OVERRIDDEN.load(Ordering::SeqCst) {
        return Ok(());
    }

    run_powercfg(&["/setacvalueindex", "SCHEME_CURRENT", "SUB_BUTTONS", "LIDACTION", "1"])?;
    run_powercfg(&["/setdcvalueindex", "SCHEME_CURRENT", "SUB_BUTTONS", "LIDACTION", "1"])?;
    run_powercfg(&["/setactive", "SCHEME_CURRENT"])?;

    LID_OVERRIDDEN.store(false, Ordering::SeqCst);
    info!("Lid-close action restored to 'Sleep'");
    Ok(())
}

pub fn is_overridden() -> bool {
    LID_OVERRIDDEN.load(Ordering::SeqCst)
}

fn run_powercfg(args: &[&str]) -> Result<()> {
    let output = Command::new("powercfg")
        .args(args)
        .output()
        .context("Failed to run powercfg. Is it available on PATH?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("powercfg {:?} failed: {}", args, stderr);
        // Don't hard-fail — some systems restrict this without admin
    }

    Ok(())
}
