use log::{info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder, TrayIconEvent, TrayIconEventReceiver,
};

use crate::lid;
use crate::power;

static TRAY_RUNNING: AtomicBool = AtomicBool::new(false);

/// Simple 32x32 RGBA icon (solid color).
fn make_icon(r: u8, g: u8, b: u8) -> tray_icon::Icon {
    let size = 32;
    let mut rgba = vec![0u8; size * size * 4];
    for pixel in rgba.chunks_exact_mut(4) {
        pixel[0] = r;
        pixel[1] = g;
        pixel[2] = b;
        pixel[3] = 255;
    }
    tray_icon::Icon::from_rgba(rgba, size as u32, size as u32).expect("Failed to create icon")
}

fn icon_for_state(awake: bool) -> tray_icon::Icon {
    if awake {
        make_icon(40, 200, 80) // green = awake
    } else {
        make_icon(140, 140, 140) // gray = normal
    }
}

/// Start the system tray on a background thread.
/// Returns immediately. Call this once at startup.
pub fn start(override_lid: bool, restore_lid: bool) {
    if TRAY_RUNNING.swap(true, Ordering::SeqCst) {
        return; // already running
    }

    thread::spawn(move || {
        if let Err(e) = run_tray(override_lid, restore_lid) {
            warn!("Tray exited with error: {}", e);
        }
        TRAY_RUNNING.store(false, Ordering::SeqCst);
    });
}

fn run_tray(override_lid: bool, restore_lid: bool) -> Result<(), Box<dyn std::error::Error>> {
    // On Windows, tray-icon needs a message pump.
    // We create the tray and then pump Win32 messages + poll tray events.

    let awake_item = MenuItem::new("Keep Awake", true, None);
    let sleep_item = MenuItem::new("Allow Sleep", true, None);
    let status_item = MenuItem::new("Refresh Status", true, None);
    let quit_item = MenuItem::new("Quit Agent-awake", true, None);

    let menu = Menu::new();
    menu.append(&awake_item)?;
    menu.append(&sleep_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&status_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let mut tray: TrayIcon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Agent-awake")
        .with_icon(icon_for_state(power::is_awake()))
        .build()?;

    info!("System tray icon started");

    let menu_channel = MenuEvent::receiver();
    let tray_channel: &TrayIconEventReceiver = TrayIconEvent::receiver();

    loop {
        // Update icon + tooltip periodically
        let awake = power::is_awake();
        let tip = if awake {
            "Agent-awake — ON (keeping system awake)"
        } else {
            "Agent-awake — OFF"
        };
        let _ = tray.set_tooltip(Some(tip));
        let _ = tray.set_icon(Some(icon_for_state(awake)));

        // Handle menu clicks
        while let Ok(event) = menu_channel.try_recv() {
            if event.id == awake_item.id() {
                if power::keep_awake(true).is_ok() {
                    if override_lid {
                        let _ = lid::set_do_nothing();
                    }
                    info!("Tray: Keep Awake");
                }
            } else if event.id == sleep_item.id() {
                let _ = power::keep_awake(false);
                if restore_lid {
                    let _ = lid::restore_default();
                }
                info!("Tray: Allow Sleep");
            } else if event.id == status_item.id() {
                info!(
                    "Tray status — awake={}, lid_override={}",
                    power::is_awake(),
                    lid::is_overridden()
                );
            } else if event.id == quit_item.id() {
                info!("Tray: Quit requested");
                if restore_lid {
                    let _ = lid::restore_default();
                }
                let _ = power::keep_awake(false);
                // Drop tray then exit process
                drop(tray);
                std::process::exit(0);
            }
        }

        // Drain tray icon events (clicks etc.)
        while let Ok(_e) = tray_channel.try_recv() {
            // optional: left-click could toggle awake
        }

        // Simple message pump for Windows so the tray stays responsive
        #[cfg(target_os = "windows")]
        {
            pump_windows_messages();
        }

        thread::sleep(Duration::from_millis(200));
    }
}

#[cfg(target_os = "windows")]
fn pump_windows_messages() {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
    };

    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn pump_windows_messages() {}
