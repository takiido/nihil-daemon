use std::path::Path;
use zbus::zvariant;
use serde::{Serialize, Deserialize};
use crate::paths;

#[derive(zvariant::Type, Serialize, Deserialize)]
pub struct SystemFeatures {
    pub backlight: bool,
}

impl SystemFeatures {
    /// Detects available system features.
    /// TODO: use in active_features with user choice
    pub fn detect() -> Self {
        Self {
            backlight: Path::new(paths::BACKLIGHT_PATH)
                .read_dir()
                .map(|mut d| d.next().is_some())
                .unwrap_or(false),
        }
    }
}