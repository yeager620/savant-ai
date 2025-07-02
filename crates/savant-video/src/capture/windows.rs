use super::{ApplicationInfo, DisplayInfo, ScreenCapture, VideoCapture};
use anyhow::Result;
use async_trait::async_trait;

pub struct WindowsCapture {
    stealth_mode: bool,
}

impl WindowsCapture {
    pub fn new() -> Result<Self> {
        Ok(Self {
            stealth_mode: true,
        })
    }
}

#[async_trait]
impl VideoCapture for WindowsCapture {
    async fn capture_screen(&self) -> Result<ScreenCapture> {
        // TODO: Implement Windows screen capture with stealth mode
        anyhow::bail!("Windows capture not yet implemented")
    }

    async fn get_displays(&self) -> Result<Vec<DisplayInfo>> {
        // TODO: Enumerate Windows displays
        Ok(vec![])
    }

    async fn set_stealth_mode(&self, _enabled: bool) -> Result<()> {
        // TODO: Implement stealth mode for Windows
        Ok(())
    }

    async fn get_active_application(&self) -> Result<Option<ApplicationInfo>> {
        // TODO: Get active window info on Windows
        Ok(None)
    }
}