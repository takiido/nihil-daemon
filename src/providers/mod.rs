pub mod dummy;
pub mod backlight;

pub enum ShellEvent {
    Dummy(u32),
    Backlight(BacklightEvent),
}

pub enum BacklightEvent {
    DeviceAdded(String),
    DeviceRemoved(String),
    Brightness { device: String, value: u32 },
}