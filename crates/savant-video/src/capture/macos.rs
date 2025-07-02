use super::{ApplicationInfo, DisplayInfo, ScreenCapture, VideoCapture};
use anyhow::Result;
use async_trait::async_trait;
use cocoa::appkit::NSScreen;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSString};
use core_graphics::display::{CGDisplayBounds, CGMainDisplayID};
use core_graphics::image::CGImage;
use core_graphics::window::{
    kCGNullWindowID, kCGWindowImageDefault, kCGWindowListOptionOnScreenOnly,
    CGWindowListCreateImage,
};
use foreign_types::ForeignType;
use image::{DynamicImage, RgbaImage};
use objc::runtime::{Class, Object};
use objc::{msg_send, sel, sel_impl};
use std::sync::Arc;
use std::sync::Mutex;

pub struct MacOSCapture {
    stealth_mode: Arc<Mutex<bool>>,
    stealth_window: Arc<Mutex<Option<*mut Object>>>,
}

unsafe impl Send for MacOSCapture {}
unsafe impl Sync for MacOSCapture {}

impl MacOSCapture {
    pub fn new() -> Result<Self> {
        Ok(Self {
            stealth_mode: Arc::new(Mutex::new(true)), // Default to stealth
            stealth_window: Arc::new(Mutex::new(None)),
        })
    }

    fn create_stealth_window(&self) -> Result<*mut Object> {
        unsafe {
            // Create an invisible window that excludes our capture from other apps
            let window_class = Class::get("NSWindow").unwrap();
            let rect = cocoa::foundation::NSRect {
                origin: cocoa::foundation::NSPoint { x: 0.0, y: 0.0 },
                size: cocoa::foundation::NSSize {
                    width: 1.0,
                    height: 1.0,
                },
            };

            let style_mask = 0; // No decorations
            let backing_store = 2; // Buffered
            let defer = 0; // NO

            let window: *mut Object = msg_send![window_class, alloc];
            let window: *mut Object = msg_send![
                window,
                initWithContentRect:rect
                styleMask:style_mask
                backing:backing_store
                defer:defer
            ];

            // Make window invisible and exclude from capture
            let _: () = msg_send![window, setOpaque: false];
            let _: () = msg_send![window, setAlphaValue: 0.0];
            let _: () = msg_send![window, setLevel: i64::MAX]; // Above all windows
            let _: () = msg_send![window, setSharingType: 0]; // None - excludes from capture
            let _: () = msg_send![window, setCollectionBehavior: 1 << 7]; // Stationary
            
            // This is the key for stealth mode - exclude from window list
            let _: () = msg_send![window, setExcludedFromWindowsMenu: true];
            let _: () = msg_send![window, setHidesOnDeactivate: false];

            Ok(window)
        }
    }

    fn capture_display_image(&self, display_id: u32) -> Result<Vec<u8>> {
        unsafe {
            let rect = CGDisplayBounds(display_id);
            
            // Create image with specific options for stealth
            let options = if *self.stealth_mode.lock().unwrap() {
                kCGWindowListOptionOnScreenOnly | (1 << 5) // Exclude desktop elements
            } else {
                kCGWindowListOptionOnScreenOnly
            };

            let image = CGWindowListCreateImage(
                rect,
                options,
                kCGNullWindowID,
                kCGWindowImageDefault,
            );

            if image.is_null() {
                anyhow::bail!("Failed to capture screen");
            }

            // Convert to bytes immediately
            let cgimage = CGImage::from_ptr(image);
            let width = cgimage.width() as u32;
            let height = cgimage.height() as u32;
            let bytes_per_row = cgimage.bytes_per_row();
            let data = cgimage.data();

            let mut rgba_image = RgbaImage::new(width, height);

            for y in 0..height as usize {
                for x in 0..width as usize {
                    let offset = y * bytes_per_row + x * 4;
                    if offset + 3 < data.len() as usize {
                        let pixel = image::Rgba([
                            data[offset + 2], // B -> R
                            data[offset + 1], // G
                            data[offset],     // R -> B
                            data[offset + 3], // A
                        ]);
                        rgba_image.put_pixel(x as u32, y as u32, pixel);
                    }
                }
            }

            // Convert to PNG bytes
            let mut png_bytes = Vec::new();
            {
                use image::ImageEncoder;
                let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
                encoder.write_image(
                    rgba_image.as_raw(),
                    width,
                    height,
                    image::ExtendedColorType::Rgba8
                )?;
            }

            Ok(png_bytes)
        }
    }


    fn get_frontmost_app(&self) -> Result<ApplicationInfo> {
        unsafe {
            let workspace_class = Class::get("NSWorkspace").unwrap();
            let shared_workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
            let frontmost_app: *mut Object = msg_send![shared_workspace, frontmostApplication];

            if frontmost_app.is_null() {
                return Ok(ApplicationInfo {
                    name: "Unknown".to_string(),
                    window_title: None,
                    bundle_id: None,
                });
            }

            // Get app name
            let localized_name: *mut Object = msg_send![frontmost_app, localizedName];
            let name = if !localized_name.is_null() {
                let name_str: *const i8 = msg_send![localized_name, UTF8String];
                std::ffi::CStr::from_ptr(name_str)
                    .to_string_lossy()
                    .to_string()
            } else {
                "Unknown".to_string()
            };

            // Get bundle ID
            let bundle_id_obj: *mut Object = msg_send![frontmost_app, bundleIdentifier];
            let bundle_id = if !bundle_id_obj.is_null() {
                let bundle_str: *const i8 = msg_send![bundle_id_obj, UTF8String];
                Some(
                    std::ffi::CStr::from_ptr(bundle_str)
                        .to_string_lossy()
                        .to_string(),
                )
            } else {
                None
            };

            Ok(ApplicationInfo {
                name,
                window_title: None, // TODO: Get window title via Accessibility API
                bundle_id,
            })
        }
    }
}

#[async_trait]
impl VideoCapture for MacOSCapture {
    async fn capture_screen(&self) -> Result<ScreenCapture> {
        let display_id = unsafe { CGMainDisplayID() };
        let png_bytes = self.capture_display_image(display_id)?;
        
        // Convert PNG bytes back to DynamicImage
        let image = image::load_from_memory(&png_bytes)?;

        Ok(ScreenCapture {
            image,
            display_id: Some(display_id.to_string()),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn get_displays(&self) -> Result<Vec<DisplayInfo>> {
        unsafe {
            let screens = NSScreen::screens(nil);
            let count = NSArray::count(screens);
            let mut displays = Vec::new();

            for i in 0..count {
                let screen = NSArray::objectAtIndex(screens, i);
                let frame = NSScreen::frame(screen);
                
                // Get display ID
                let device_description: id = msg_send![screen, deviceDescription];
                let screen_number_key = NSString::alloc(nil)
                    .init_str("NSScreenNumber");
                let screen_number: id = msg_send![device_description, objectForKey: screen_number_key];
                let display_id: u32 = msg_send![screen_number, unsignedIntValue];

                displays.push(DisplayInfo {
                    id: display_id.to_string(),
                    name: format!("Display {}", i + 1),
                    resolution: (frame.size.width as u32, frame.size.height as u32),
                    is_primary: i == 0,
                });
            }

            Ok(displays)
        }
    }

    async fn set_stealth_mode(&self, enabled: bool) -> Result<()> {
        *self.stealth_mode.lock().unwrap() = enabled;
        
        if enabled {
            // Create stealth window if not exists
            let mut window_guard = self.stealth_window.lock().unwrap();
            if window_guard.is_none() {
                let window = self.create_stealth_window()?;
                *window_guard = Some(window);
                
                // Make it key and order front briefly to activate stealth
                unsafe {
                    let _: () = msg_send![window, makeKeyAndOrderFront: nil];
                    let _: () = msg_send![window, orderOut: nil];
                }
            }
        } else {
            // Remove stealth window
            let mut window_guard = self.stealth_window.lock().unwrap();
            if let Some(window) = window_guard.take() {
                unsafe {
                    let _: () = msg_send![window, close];
                    let _: () = msg_send![window, release];
                }
            }
        }
        
        Ok(())
    }

    async fn get_active_application(&self) -> Result<Option<ApplicationInfo>> {
        Ok(Some(self.get_frontmost_app()?))
    }
}

impl Drop for MacOSCapture {
    fn drop(&mut self) {
        // Clean up stealth window if exists
        if let Some(window) = self.stealth_window.lock().unwrap().take() {
            unsafe {
                let _: () = msg_send![window, close];
                let _: () = msg_send![window, release];
            }
        }
    }
}