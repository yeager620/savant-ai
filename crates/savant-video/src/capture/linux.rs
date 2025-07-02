use super::{ApplicationInfo, DisplayInfo, ScreenCapture, VideoCapture};
use anyhow::Result;
use async_trait::async_trait;

pub struct LinuxCapture {
    stealth_mode: bool,
}

impl LinuxCapture {
    pub fn new() -> Result<Self> {
        Ok(Self {
            stealth_mode: true,
        })
    }
}

#[async_trait]
impl VideoCapture for LinuxCapture {
    async fn capture_screen(&self) -> Result<ScreenCapture> {
        // TODO: Implement Linux screen capture with X11/Wayland
        anyhow::bail!("Linux capture not yet implemented")
    }

    async fn get_displays(&self) -> Result<Vec<DisplayInfo>> {
        // TODO: Enumerate Linux displays
        Ok(vec![])
    }

    async fn set_stealth_mode(&self, _enabled: bool) -> Result<()> {
        // TODO: Implement stealth mode for Linux
        Ok(())
    }

    async fn get_active_application(&self) -> Result<Option<ApplicationInfo>> {
        // TODO: Get active window info on Linux
        Ok(None)
    }
}