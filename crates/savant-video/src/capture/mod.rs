use anyhow::Result;
use async_trait::async_trait;
use image::DynamicImage;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

#[async_trait]
pub trait VideoCapture: Send + Sync {
    /// Capture a screenshot of the entire screen
    async fn capture_screen(&self) -> Result<ScreenCapture>;
    
    /// Get information about available displays
    async fn get_displays(&self) -> Result<Vec<DisplayInfo>>;
    
    /// Set stealth mode for invisibility to external capture
    async fn set_stealth_mode(&self, enabled: bool) -> Result<()>;
    
    /// Get the currently active application
    async fn get_active_application(&self) -> Result<Option<ApplicationInfo>>;
}

#[derive(Debug, Clone)]
pub struct ScreenCapture {
    pub image: DynamicImage,
    pub display_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub resolution: (u32, u32),
    pub is_primary: bool,
}

#[derive(Debug, Clone)]
pub struct ApplicationInfo {
    pub name: String,
    pub window_title: Option<String>,
    pub bundle_id: Option<String>,
}

/// Create platform-specific capture implementation
pub fn create_platform_capture() -> Result<Box<dyn VideoCapture>> {
    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(macos::MacOSCapture::new()?))
    }
    
    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(windows::WindowsCapture::new()?))
    }
    
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::LinuxCapture::new()?))
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        anyhow::bail!("Unsupported platform")
    }
}