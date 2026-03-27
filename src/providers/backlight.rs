use std::fs;
use std::process::Command;
use udev::MonitorBuilder;
use tokio::io::unix::AsyncFd;
use tokio::sync::mpsc::Sender;
use super::{BacklightEvent, ShellEvent};
use crate::paths;

#[derive(Debug, thiserror::Error)]
pub enum BacklightError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
    #[error("failed to set brightness")]
    SetBrightness,
    #[error("channel closed")]
    ChannelClosed,
}

pub async fn watch(tx: Sender<ShellEvent>) {
    match watch_inner(tx).await {
        Ok(_) => {}
        Err(BacklightError::ChannelClosed) => {},
        Err(e) => eprintln!("Backlight watcher stopped: {e}"),
    }
}

async fn watch_inner(tx: Sender<ShellEvent>) -> Result<(), BacklightError> {
    let monitor = MonitorBuilder::new()
        .and_then(|m| m.match_subsystem("backlight"))
        .and_then(|m| m.listen())?;

    let async_fd = AsyncFd::new(monitor)?;

    let devices = get_devices()?;
    for device in devices {
        tx.send(ShellEvent::Backlight(BacklightEvent::DeviceAdded(device.clone()))).await
            .map_err(|_| BacklightError::ChannelClosed)?;
        match get_brightness(&device) {
            Ok(value) => {
                tx.send(ShellEvent::Backlight(BacklightEvent::Brightness { device, value })).await
                    .map_err(|_| BacklightError::ChannelClosed)?;
            }
            Err(e) => eprintln!("Brightness error: {e}"),
        }
    }

    loop {
        let mut guard = async_fd.readable().await?;
        guard.clear_ready();

        for event in async_fd.get_ref().iter() {
            let action = event.action().unwrap_or_default();
            let name = match event.sysname().to_str() {
                Some(n) => n.to_string(),
                None => continue,
            };

            match action.to_str() {
                Some("change") => {
                    match get_brightness(&name) {
                        Ok(value) => {
                            tx.send(ShellEvent::Backlight(BacklightEvent::Brightness { device: name, value })).await
                                .map_err(|_| BacklightError::ChannelClosed)?;
                        }
                        Err(e) => eprintln!("Brightness error: {e}"),
                    }
                }
                Some("add") => {
                    tx.send(ShellEvent::Backlight(BacklightEvent::DeviceAdded(name))).await
                        .map_err(|_| BacklightError::ChannelClosed)?;
                }
                Some("remove") => {
                    tx.send(ShellEvent::Backlight(BacklightEvent::DeviceRemoved(name))).await
                        .map_err(|_| BacklightError::ChannelClosed)?;
                }
                _ => {}
            }
        }
    }
}

fn get_devices() -> Result<Vec<String>, BacklightError> {
    fs::read_dir(paths::BACKLIGHT_PATH)?
        .map(|e| Ok(e?.file_name().to_string_lossy().into_owned()))
        .collect()
}

fn get_brightness(device: &str) -> Result<u32, BacklightError> {
    let max = fs::read_to_string(sysfs_path(device, "max_brightness"))?
        .trim()
        .parse::<u32>()?;
    let current = fs::read_to_string(sysfs_path(device, "brightness"))?
        .trim()
        .parse::<u32>()?;

    let fmt_value = (current as f32 / max as f32 * 100.0).round() as u32;
    Ok(fmt_value)
}

fn sysfs_path(device: &str, file: &str) -> std::path::PathBuf {
    std::path::Path::new(paths::BACKLIGHT_PATH).join(device).join(file)
}

pub fn set_brightness(brightness: u32) -> Result<(), BacklightError> {
    let level = format!("{}%", brightness);

    let output = Command::new("brightnessctl")
        .args(["set", &level])
        .output()
        .map_err(|_| BacklightError::SetBrightness)?;

    if !output.status.success() {
        return Err(BacklightError::SetBrightness);
    }

    Ok(())}