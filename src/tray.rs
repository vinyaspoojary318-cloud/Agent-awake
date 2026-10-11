use log::{info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder, TrayIconEvent, TrayIconEventReceiver,
};

use crate::audit;
use crate::bagmode;
use crate::lid;
use crate::power;
use crate::thermal;

static TRAY_RUNNING: AtomicBool = AtomicBool::new(false);

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

fn icon_for_state(awake: bool, bag: bool) -> tray_icon::Icon {
    if awake && bag {
        make_icon(30, 144, 255) // blue = awake + bag mode
    } else if awake {
        make_icon(40, 200, 80) // green = awake
    } else {
        make_icon(140, 140, 140) // gray
    }
}

fn tooltip_text() -> String {
    let awake = power::is_awake();
    let bag = bagmode::is_active();
    let cpu = thermal::cpu_usage();
    format!(
        "Agent-awake | {} | {} | CPU {:.0}%",
        if awake { "AWAKE" } else { "sleep ok" },
        if bag { "BAG" } else { "normal" },
        cpu
    )
}

/// Start the system tray on a background thread.
pub fn start(override_lid: bool, restore_lid: bool) {
    if TRAY_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(move || {
        if let Err(e) = run_tray(override_lid, restore_lid) {
            warn!("Tray exited with error: {}", e);
        }
        TRAY_RUNNING.store(false, Ordering::SeqCst);
    });
}

fn run_tray(override_lid: bool, restore_lid: bool) -> Result<(), Box<dyn std::error::Error>> {
    let awake_item = MenuItem::new("Keep Awake", true, None);
    let sleep_item = MenuItem::new("Allow Sleep", true, None);
    let bag_item = MenuItem::new("Toggle Bag Mode", true, None);
    let status_item = MenuItem::new("Log Status", true, None);
    let quit_item = MenuItem::new("Quit Agent-awake", true, None);

    let menu = Menu::new();
    menu.append(&awake_item)?;
    menu.append(&sleep_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&bag_item)?;
    menu.append(&status_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let mut tray: TrayIcon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(&tooltip_text())
        .with_icon(icon_for_state(power::is_awake(), bagmode::is_active()))
        .build()?;

    info!("System tray icon started");

    let menu_channel = MenuEvent::receiver();
    let tray_channel: &TrayIconEventReceiver = TrayIconEvent::receiver();

    loop {
        let awake = power::is_awake();
        let bag = bagmode::is_active();
        let _ = tray.set_tooltip(Some(tooltip_text()));
        let _ = tray.set_icon(Some(icon_for_state(awake, bag)));

        while let Ok(event) = menu_channel.try_recv() {
            if event.id == awake_item.id() {
                if power::keep_awake(true).is_ok() {
                    if override_lid {
                        let _ = lid::set_do_nothing();
                    }
                    bagmode::set_active(true);
                    audit::log_event(true, "tray", "Keep Awake");
                    info!("Tray: Keep Awake");
                }
            } else if event.id == sleep_item.id() {
                let _ = power::keep_awake(false);
                if restore_lid {
                    let _ = lid::restore_default();
                }
                bagmode::set_active(false);
                audit::log_event(false, "tray", "Allow Sleep");
                info!("Tray: Allow Sleep");
            } else if event.id == bag_item.id() {
                let next = !bagmode::is_active();
                bagmode::set_active(next);
                info!("Tray: Bag mode -> {}", next);
            } else if event.id == status_item.id() {
                info!(
                    "Tray status — awake={}, bag={}, lid={}, cpu={:.0}%",
                    power::is_awake(),
                    bagmode::is_active(),
                    lid::is_overridden(),
                    thermal::cpu_usage()
                );
            } else if event.id == quit_item.id() {
                info!("Tray: Quit");
                if restore_lid {
                    let _ = lid::restore_default();
                }
                let _ = power::keep_awake(false);
                bagmode::set_active(false);
                audit::log_event(false, "tray", "Quit");
                drop(tray);
                std::process::exit(0);
            }
        }

        while let Ok(_e) = tray_channel.try_recv() {}

        #[cfg(target_os = "windows")]
        pump_windows_messages();

        thread::sleep(Duration::from_millis(250));
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
