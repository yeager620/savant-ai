# Audio/Video Setup Automation

## 🎯 Overview

Savant AI's audio/video setup automation provides seamless configuration of complex multimodal capture systems. This document outlines the automated setup workflows for different platforms and use cases.

## 🔧 Core Architecture

### Automated Setup Framework

```rust
// audio_video_setup.rs
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioVideoSetup {
    pub platform: Platform,
    pub audio_config: AudioConfig,
    pub video_config: VideoConfig,
    pub automation_level: AutomationLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub input_devices: Vec<AudioDevice>,
    pub output_devices: Vec<AudioDevice>,
    pub virtual_devices: Vec<VirtualAudioDevice>,
    pub sample_rate: u32,
    pub channels: u8,
    pub buffer_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    pub capture_devices: Vec<VideoDevice>,
    pub screen_capture: ScreenCaptureConfig,
    pub resolution: Resolution,
    pub frame_rate: f32,
    pub encoding: VideoEncoding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationLevel {
    Minimal,      // Basic audio/video only
    Standard,     // Standard multimodal setup
    Advanced,     // Full system audio capture
    Professional, // Multi-source, high-quality
}

impl AudioVideoSetup {
    pub async fn auto_configure(platform: Platform, level: AutomationLevel) -> Result<Self, SetupError> {
        let audio_config = AudioConfig::detect_and_configure(platform, level).await?;
        let video_config = VideoConfig::detect_and_configure(platform, level).await?;
        
        Ok(AudioVideoSetup {
            platform,
            audio_config,
            video_config,
            automation_level: level,
        })
    }
    
    pub async fn apply_configuration(&self) -> Result<(), SetupError> {
        // Apply audio configuration
        self.audio_config.apply(&self.platform).await?;
        
        // Apply video configuration
        self.video_config.apply(&self.platform).await?;
        
        // Validate configuration
        self.validate_setup().await?;
        
        Ok(())
    }
}
```

---

## 🎤 Audio Setup Automation

### Cross-Platform Audio Detection

```rust
// audio_detection.rs
impl AudioConfig {
    pub async fn detect_and_configure(platform: Platform, level: AutomationLevel) -> Result<Self, AudioError> {
        let detector = AudioDetector::new(platform);
        
        // Detect available devices
        let input_devices = detector.detect_input_devices().await?;
        let output_devices = detector.detect_output_devices().await?;
        
        // Configure based on automation level
        let config = match level {
            AutomationLevel::Minimal => {
                Self::configure_minimal(input_devices, output_devices).await?
            }
            AutomationLevel::Standard => {
                Self::configure_standard(input_devices, output_devices).await?
            }
            AutomationLevel::Advanced => {
                Self::configure_advanced(input_devices, output_devices, platform).await?
            }
            AutomationLevel::Professional => {
                Self::configure_professional(input_devices, output_devices, platform).await?
            }
        };
        
        Ok(config)
    }
    
    async fn configure_minimal(
        input_devices: Vec<AudioDevice>,
        output_devices: Vec<AudioDevice>
    ) -> Result<Self, AudioError> {
        // Select default microphone
        let default_input = input_devices.into_iter()
            .find(|d| d.is_default)
            .ok_or(AudioError::NoDefaultDevice)?;
        
        // Select default speakers
        let default_output = output_devices.into_iter()
            .find(|d| d.is_default)
            .ok_or(AudioError::NoDefaultDevice)?;
        
        Ok(AudioConfig {
            input_devices: vec![default_input],
            output_devices: vec![default_output],
            virtual_devices: vec![],
            sample_rate: 44100,
            channels: 2,
            buffer_size: 1024,
        })
    }
    
    async fn configure_advanced(
        input_devices: Vec<AudioDevice>,
        output_devices: Vec<AudioDevice>,
        platform: Platform
    ) -> Result<Self, AudioError> {
        // Install and configure virtual audio devices for system audio capture
        let virtual_devices = match platform {
            Platform::MacOS => {
                vec![
                    VirtualAudioDevice::blackhole_2ch(),
                    VirtualAudioDevice::multi_output_device(),
                ]
            }
            Platform::Windows => {
                vec![
                    VirtualAudioDevice::virtual_audio_cable(),
                    VirtualAudioDevice::voicemeeter(),
                ]
            }
            Platform::Linux => {
                vec![
                    VirtualAudioDevice::pulseaudio_monitor(),
                    VirtualAudioDevice::jack_loopback(),
                ]
            }
        };
        
        // Configure for system audio capture
        let mut config = Self::configure_standard(input_devices, output_devices).await?;
        config.virtual_devices = virtual_devices;
        config.sample_rate = 48000; // Higher quality
        config.buffer_size = 512;   // Lower latency
        
        Ok(config)
    }
}
```

### macOS Audio Setup

```rust
// macos_audio_setup.rs
use core_audio::*;
use std::process::Command;

impl AudioConfig {
    pub async fn setup_macos_system_audio() -> Result<MacOSAudioSetup, AudioError> {
        let setup = MacOSAudioSetup::new();
        
        // 1. Install BlackHole if not present
        if !setup.is_blackhole_installed().await? {
            setup.install_blackhole().await?;
        }
        
        // 2. Create Multi-Output Device
        let multi_output = setup.create_multi_output_device().await?;
        
        // 3. Configure system to use Multi-Output
        setup.set_system_output_device(multi_output.id).await?;
        
        // 4. Set up audio routing
        setup.configure_audio_routing().await?;
        
        Ok(setup)
    }
}

struct MacOSAudioSetup {
    blackhole_device: Option<AudioDevice>,
    multi_output_device: Option<AudioDevice>,
    original_output: Option<AudioDevice>,
}

impl MacOSAudioSetup {
    async fn install_blackhole(&self) -> Result<(), AudioError> {
        // Download and install BlackHole
        let download_url = "https://github.com/ExistentialAudio/BlackHole/releases/latest/download/BlackHole.2ch.pkg";
        let temp_file = "/tmp/BlackHole.2ch.pkg";
        
        // Download
        let mut cmd = Command::new("curl");
        cmd.args(&["-L", download_url, "-o", temp_file]);
        let output = cmd.output().await?;
        
        if !output.status.success() {
            return Err(AudioError::DownloadFailed("BlackHole".to_string()));
        }
        
        // Install
        let mut cmd = Command::new("sudo");
        cmd.args(&["installer", "-pkg", temp_file, "-target", "/"]);
        let output = cmd.output().await?;
        
        if !output.status.success() {
            return Err(AudioError::InstallationFailed("BlackHole".to_string()));
        }
        
        // Clean up
        tokio::fs::remove_file(temp_file).await?;
        
        Ok(())
    }
    
    async fn create_multi_output_device(&self) -> Result<AudioDevice, AudioError> {
        // Use Core Audio to create Multi-Output Device
        let script = r#"
        tell application "Audio MIDI Setup"
            activate
            delay 1
            
            -- Create Multi-Output Device
            set multiOutput to make new aggregate device
            set name of multiOutput to "Savant AI Multi-Output"
            
            -- Add devices
            set devices to audio devices
            repeat with device in devices
                if name of device is "Built-in Output" then
                    set subdevices of multiOutput to {device}
                end if
                if name of device is "BlackHole 2ch" then
                    set subdevices of multiOutput to (subdevices of multiOutput) & {device}
                end if
            end repeat
            
            -- Configure as system output
            set default output device to multiOutput
            
            return id of multiOutput
        end tell
        "#;
        
        let mut cmd = Command::new("osascript");
        cmd.arg("-e").arg(script);
        let output = cmd.output().await?;
        
        if !output.status.success() {
            return Err(AudioError::ConfigurationFailed("Multi-Output Device".to_string()));
        }
        
        let device_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        Ok(AudioDevice {
            id: device_id,
            name: "Savant AI Multi-Output".to_string(),
            is_input: false,
            is_output: true,
            is_default: true,
            channels: 2,
            sample_rate: 44100,
        })
    }
    
    async fn configure_audio_routing(&self) -> Result<(), AudioError> {
        // Set up audio routing for system audio capture
        let routing_config = AudioRoutingConfig {
            system_output: "Savant AI Multi-Output",
            capture_source: "BlackHole 2ch",
            monitor_output: "Built-in Output",
        };
        
        // Apply routing configuration
        self.apply_routing_config(routing_config).await?;
        
        Ok(())
    }
}
```

### Windows Audio Setup

```rust
// windows_audio_setup.rs
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;

impl AudioConfig {
    pub async fn setup_windows_system_audio() -> Result<WindowsAudioSetup, AudioError> {
        let setup = WindowsAudioSetup::new();
        
        // 1. Install Virtual Audio Cable or VoiceMeeter
        if !setup.is_virtual_audio_installed().await? {
            setup.install_virtual_audio().await?;
        }
        
        // 2. Configure audio routing
        setup.configure_audio_routing().await?;
        
        // 3. Set up WASAPI loopback
        setup.configure_wasapi_loopback().await?;
        
        Ok(setup)
    }
}

struct WindowsAudioSetup {
    virtual_audio_device: Option<AudioDevice>,
    loopback_device: Option<AudioDevice>,
}

impl WindowsAudioSetup {
    async fn install_virtual_audio(&self) -> Result<(), AudioError> {
        // Try VoiceMeeter first (free and reliable)
        if let Ok(_) = self.install_voicemeeter().await {
            return Ok(());
        }
        
        // Fall back to Virtual Audio Cable
        self.install_virtual_audio_cable().await
    }
    
    async fn install_voicemeeter(&self) -> Result<(), AudioError> {
        let download_url = "https://download.vb-audio.com/Download_CABLE/VoicemeeterSetup.exe";
        let temp_file = "C:\\temp\\VoicemeeterSetup.exe";
        
        // Create temp directory
        tokio::fs::create_dir_all("C:\\temp").await?;
        
        // Download
        let mut cmd = Command::new("powershell");
        cmd.args(&[
            "-Command",
            &format!("Invoke-WebRequest -Uri '{}' -OutFile '{}'", download_url, temp_file)
        ]);
        let output = cmd.output().await?;
        
        if !output.status.success() {
            return Err(AudioError::DownloadFailed("VoiceMeeter".to_string()));
        }
        
        // Install
        let mut cmd = Command::new(temp_file);
        cmd.args(&["/S"]); // Silent install
        let output = cmd.output().await?;
        
        if !output.status.success() {
            return Err(AudioError::InstallationFailed("VoiceMeeter".to_string()));
        }
        
        // Clean up
        tokio::fs::remove_file(temp_file).await?;
        
        Ok(())
    }
    
    async fn configure_wasapi_loopback(&self) -> Result<(), AudioError> {
        // Configure WASAPI loopback for system audio capture
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)?;
            
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(
                &MMDeviceEnumerator,
                None,
                CLSCTX_ALL,
            )?;
            
            // Get default audio endpoint
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
            
            // Activate audio client
            let audio_client: IAudioClient = device.Activate(
                CLSCTX_ALL,
                None,
            )?;
            
            // Get mix format
            let mix_format = audio_client.GetMixFormat()?;
            
            // Initialize for loopback
            audio_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_LOOPBACK,
                10_000_000, // 1 second buffer
                0,
                mix_format,
                None,
            )?;
            
            // Start capture
            audio_client.Start()?;
            
            CoUninitialize();
        }
        
        Ok(())
    }
}
```

### Linux Audio Setup

```rust
// linux_audio_setup.rs
use pulse::*;
use std::process::Command;

impl AudioConfig {
    pub async fn setup_linux_system_audio() -> Result<LinuxAudioSetup, AudioError> {
        let setup = LinuxAudioSetup::new();
        
        // 1. Configure PulseAudio
        setup.configure_pulseaudio().await?;
        
        // 2. Set up monitor sources
        setup.setup_monitor_sources().await?;
        
        // 3. Configure JACK if available
        if setup.is_jack_available().await? {
            setup.configure_jack().await?;
        }
        
        Ok(setup)
    }
}

struct LinuxAudioSetup {
    pulse_context: Option<pulse::context::Context>,
    jack_client: Option<String>,
}

impl LinuxAudioSetup {
    async fn configure_pulseaudio(&self) -> Result<(), AudioError> {
        // Load necessary PulseAudio modules
        let modules = vec![
            "module-null-sink sink_name=savant_ai_sink",
            "module-loopback source=savant_ai_sink.monitor sink=@DEFAULT_SINK@",
            "module-loopback source=@DEFAULT_SOURCE@ sink=savant_ai_sink",
        ];
        
        for module in modules {
            let mut cmd = Command::new("pactl");
            cmd.args(&["load-module", module]);
            let output = cmd.output().await?;
            
            if !output.status.success() {
                eprintln!("Warning: Failed to load module: {}", module);
            }
        }
        
        Ok(())
    }
    
    async fn setup_monitor_sources(&self) -> Result<(), AudioError> {
        // Create monitor sources for system audio capture
        let script = r#"
        #!/bin/bash
        
        # Create null sink for routing
        pactl load-module module-null-sink sink_name=savant_ai_monitor sink_properties=device.description="Savant AI Monitor"
        
        # Create loopback from default sink to monitor
        pactl load-module module-loopback source=@DEFAULT_SINK@.monitor sink=savant_ai_monitor
        
        # Create loopback from monitor to default sink (so you can still hear audio)
        pactl load-module module-loopback source=savant_ai_monitor.monitor sink=@DEFAULT_SINK@
        
        # Set default source to monitor
        pactl set-default-source savant_ai_monitor.monitor
        "#;
        
        let mut cmd = Command::new("bash");
        cmd.arg("-c").arg(script);
        let output = cmd.output().await?;
        
        if !output.status.success() {
            return Err(AudioError::ConfigurationFailed("Monitor sources".to_string()));
        }
        
        Ok(())
    }
    
    async fn configure_jack(&self) -> Result<(), AudioError> {
        // Configure JACK for professional audio
        let jack_config = r#"
        {
            "driver": "alsa",
            "device": "hw:0",
            "rate": 48000,
            "period": 256,
            "nperiods": 2,
            "duplex": true,
            "capture": "savant_ai_capture",
            "playback": "savant_ai_playback"
        }
        "#;
        
        // Write JACK configuration
        let config_path = "/home/user/.jackdrc";
        tokio::fs::write(config_path, jack_config).await?;
        
        // Start JACK daemon
        let mut cmd = Command::new("jackd");
        cmd.args(&["-d", "alsa", "-d", "hw:0", "-r", "48000", "-p", "256"]);
        cmd.spawn()?;
        
        // Wait for JACK to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Connect ports
        let connections = vec![
            ("system:capture_1", "savant_ai_capture:input_1"),
            ("system:capture_2", "savant_ai_capture:input_2"),
            ("savant_ai_playback:output_1", "system:playback_1"),
            ("savant_ai_playback:output_2", "system:playback_2"),
        ];
        
        for (source, dest) in connections {
            let mut cmd = Command::new("jack_connect");
            cmd.args(&[source, dest]);
            cmd.output().await?;
        }
        
        Ok(())
    }
}
```

---

## 📹 Video Setup Automation

### Screen Capture Configuration

```rust
// video_setup.rs
impl VideoConfig {
    pub async fn detect_and_configure(platform: Platform, level: AutomationLevel) -> Result<Self, VideoError> {
        let detector = VideoDetector::new(platform);
        
        // Detect displays and capabilities
        let displays = detector.detect_displays().await?;
        let capture_capabilities = detector.get_capture_capabilities().await?;
        
        // Configure based on automation level
        let config = match level {
            AutomationLevel::Minimal => {
                Self::configure_minimal(displays, capture_capabilities).await?
            }
            AutomationLevel::Standard => {
                Self::configure_standard(displays, capture_capabilities).await?
            }
            AutomationLevel::Advanced => {
                Self::configure_advanced(displays, capture_capabilities, platform).await?
            }
            AutomationLevel::Professional => {
                Self::configure_professional(displays, capture_capabilities, platform).await?
            }
        };
        
        Ok(config)
    }
    
    async fn configure_advanced(
        displays: Vec<Display>,
        capabilities: CaptureCapabilities,
        platform: Platform
    ) -> Result<Self, VideoError> {
        // High-frequency capture configuration
        let screen_capture = ScreenCaptureConfig {
            capture_method: match platform {
                Platform::MacOS => CaptureMethod::CoreGraphics,
                Platform::Windows => CaptureMethod::DXGI,
                Platform::Linux => CaptureMethod::X11,
            },
            frame_rate: 2.0, // 500ms intervals
            resolution: Resolution::Adaptive, // Adjust based on content
            compression: CompressionLevel::Fast,
            change_detection: true,
            ocr_enabled: true,
            vision_analysis: true,
        };
        
        Ok(VideoConfig {
            capture_devices: vec![],
            screen_capture,
            resolution: Resolution::Full,
            frame_rate: 2.0,
            encoding: VideoEncoding::H264Fast,
        })
    }
}
```

### macOS Screen Capture

```rust
// macos_screen_capture.rs
use core_graphics::*;
use core_foundation::*;

pub struct MacOSScreenCapture {
    display_id: CGDirectDisplayID,
    capture_session: Option<CGDisplayStreamRef>,
}

impl MacOSScreenCapture {
    pub async fn new() -> Result<Self, VideoError> {
        // Get main display
        let display_id = unsafe { CGMainDisplayID() };
        
        // Check screen recording permission
        if !Self::has_screen_recording_permission().await? {
            return Err(VideoError::PermissionDenied("Screen Recording".to_string()));
        }
        
        Ok(MacOSScreenCapture {
            display_id,
            capture_session: None,
        })
    }
    
    pub async fn start_capture(&mut self, callback: impl Fn(CGImageRef) + Send + 'static) -> Result<(), VideoError> {
        unsafe {
            // Create display stream
            let props = CFDictionaryCreate(
                kCFAllocatorDefault,
                std::ptr::null(),
                std::ptr::null(),
                0,
                &kCFTypeDictionaryKeyCallBacks,
                &kCFTypeDictionaryValueCallBacks,
            );
            
            let queue = dispatch_queue_create(
                "com.savant-ai.screen-capture".as_ptr() as *const i8,
                DISPATCH_QUEUE_SERIAL,
            );
            
            let stream = CGDisplayStreamCreateWithDispatchQueue(
                self.display_id,
                1920, // width
                1080, // height
                kCVPixelFormatType_32BGRA,
                props,
                queue,
                Box::into_raw(Box::new(callback)) as *mut c_void,
            );
            
            if stream.is_null() {
                return Err(VideoError::CaptureInitializationFailed);
            }
            
            // Start the stream
            let result = CGDisplayStreamStart(stream);
            if result != kCGErrorSuccess {
                return Err(VideoError::CaptureStartFailed);
            }
            
            self.capture_session = Some(stream);
        }
        
        Ok(())
    }
    
    async fn has_screen_recording_permission() -> Result<bool, VideoError> {
        // Test screen recording by attempting a small capture
        unsafe {
            let image = CGWindowListCreateImage(
                CGRectMake(0.0, 0.0, 1.0, 1.0),
                kCGWindowListOptionOnScreenOnly,
                kCGNullWindowID,
                kCGWindowImageDefault,
            );
            
            if image.is_null() {
                Ok(false)
            } else {
                CFRelease(image as CFTypeRef);
                Ok(true)
            }
        }
    }
}
```

### Windows Screen Capture

```rust
// windows_screen_capture.rs
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Direct3D11::*;

pub struct WindowsScreenCapture {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    output_duplication: IDXGIOutputDuplication,
}

impl WindowsScreenCapture {
    pub async fn new() -> Result<Self, VideoError> {
        unsafe {
            // Create D3D11 device
            let mut device = None;
            let mut context = None;
            
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_FLAG(0),
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )?;
            
            let device = device.unwrap();
            let context = context.unwrap();
            
            // Get DXGI adapter
            let dxgi_device: IDXGIDevice = device.cast()?;
            let adapter = dxgi_device.GetAdapter()?;
            
            // Get primary output
            let output = adapter.EnumOutputs(0)?;
            let output1: IDXGIOutput1 = output.cast()?;
            
            // Create output duplication
            let output_duplication = output1.DuplicateOutput(&device)?;
            
            Ok(WindowsScreenCapture {
                device,
                context,
                output_duplication,
            })
        }
    }
    
    pub async fn capture_frame(&self) -> Result<Vec<u8>, VideoError> {
        unsafe {
            let mut resource = None;
            let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
            
            // Acquire next frame
            let result = self.output_duplication.AcquireNextFrame(
                1000, // 1 second timeout
                &mut frame_info,
                &mut resource,
            );
            
            if result.is_err() {
                return Err(VideoError::FrameAcquisitionFailed);
            }
            
            let resource = resource.unwrap();
            let texture: ID3D11Texture2D = resource.cast()?;
            
            // Map texture to read pixels
            let mut mapped_resource = D3D11_MAPPED_SUBRESOURCE::default();
            self.context.Map(
                &texture,
                0,
                D3D11_MAP_READ,
                0,
                Some(&mut mapped_resource),
            )?;
            
            // Copy pixel data
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            texture.GetDesc(&mut desc);
            
            let pixel_data = std::slice::from_raw_parts(
                mapped_resource.pData as *const u8,
                (desc.Width * desc.Height * 4) as usize,
            );
            
            let result = pixel_data.to_vec();
            
            // Unmap and release
            self.context.Unmap(&texture, 0);
            self.output_duplication.ReleaseFrame()?;
            
            Ok(result)
        }
    }
}
```

### Linux Screen Capture

```rust
// linux_screen_capture.rs
use x11::xlib::*;
use x11::xrandr::*;

pub struct LinuxScreenCapture {
    display: *mut Display,
    root: Window,
    screen: i32,
}

impl LinuxScreenCapture {
    pub async fn new() -> Result<Self, VideoError> {
        unsafe {
            let display = XOpenDisplay(std::ptr::null());
            if display.is_null() {
                return Err(VideoError::DisplayConnectionFailed);
            }
            
            let screen = XDefaultScreen(display);
            let root = XRootWindow(display, screen);
            
            Ok(LinuxScreenCapture {
                display,
                root,
                screen,
            })
        }
    }
    
    pub async fn capture_frame(&self) -> Result<Vec<u8>, VideoError> {
        unsafe {
            // Get screen dimensions
            let mut root_return = 0;
            let mut x_return = 0;
            let mut y_return = 0;
            let mut width_return = 0;
            let mut height_return = 0;
            let mut border_width_return = 0;
            let mut depth_return = 0;
            
            XGetGeometry(
                self.display,
                self.root,
                &mut root_return,
                &mut x_return,
                &mut y_return,
                &mut width_return,
                &mut height_return,
                &mut border_width_return,
                &mut depth_return,
            );
            
            // Capture screen
            let image = XGetImage(
                self.display,
                self.root,
                0,
                0,
                width_return,
                height_return,
                AllPlanes,
                ZPixmap,
            );
            
            if image.is_null() {
                return Err(VideoError::CaptureImageFailed);
            }
            
            // Convert to RGB data
            let pixel_data = std::slice::from_raw_parts(
                (*image).data as *const u8,
                (width_return * height_return * 4) as usize,
            );
            
            let result = pixel_data.to_vec();
            
            XDestroyImage(image);
            
            Ok(result)
        }
    }
}
```

---

## 🔧 Automated Configuration Management

### Configuration Wizard

```rust
// config_wizard.rs
pub struct AudioVideoWizard {
    platform: Platform,
    current_step: usize,
    total_steps: usize,
    configuration: AudioVideoSetup,
}

impl AudioVideoWizard {
    pub fn new(platform: Platform) -> Self {
        AudioVideoWizard {
            platform,
            current_step: 0,
            total_steps: 8,
            configuration: AudioVideoSetup::default(),
        }
    }
    
    pub async fn run_wizard(&mut self) -> Result<AudioVideoSetup, WizardError> {
        // Step 1: Welcome and overview
        self.show_welcome_screen().await?;
        
        // Step 2: Detect capabilities
        self.detect_system_capabilities().await?;
        
        // Step 3: Choose automation level
        self.choose_automation_level().await?;
        
        // Step 4: Audio device selection
        self.configure_audio_devices().await?;
        
        // Step 5: Video capture setup
        self.configure_video_capture().await?;
        
        // Step 6: Permission requests
        self.request_permissions().await?;
        
        // Step 7: Apply configuration
        self.apply_configuration().await?;
        
        // Step 8: Test and validate
        self.test_configuration().await?;
        
        Ok(self.configuration.clone())
    }
    
    async fn detect_system_capabilities(&mut self) -> Result<(), WizardError> {
        self.update_progress(0.125, "Detecting system capabilities...").await;
        
        // Detect audio capabilities
        let audio_detector = AudioDetector::new(self.platform);
        let audio_capabilities = audio_detector.detect_all().await?;
        
        // Detect video capabilities
        let video_detector = VideoDetector::new(self.platform);
        let video_capabilities = video_detector.detect_all().await?;
        
        // Store capabilities
        self.configuration.audio_capabilities = Some(audio_capabilities);
        self.configuration.video_capabilities = Some(video_capabilities);
        
        self.update_progress(0.25, "✓ System capabilities detected").await;
        Ok(())
    }
    
    async fn choose_automation_level(&mut self) -> Result<(), WizardError> {
        self.update_progress(0.375, "Selecting automation level...").await;
        
        // Show automation level selection UI
        let level = self.show_automation_level_dialog().await?;
        self.configuration.automation_level = level;
        
        self.update_progress(0.5, "✓ Automation level selected").await;
        Ok(())
    }
    
    async fn configure_audio_devices(&mut self) -> Result<(), WizardError> {
        self.update_progress(0.625, "Configuring audio devices...").await;
        
        match self.configuration.automation_level {
            AutomationLevel::Minimal => {
                // Just use default devices
                self.configuration.audio_config = AudioConfig::default();
            }
            AutomationLevel::Standard => {
                // Configure microphone and speakers
                self.configuration.audio_config = self.configure_standard_audio().await?;
            }
            AutomationLevel::Advanced => {
                // Set up system audio capture
                self.configuration.audio_config = self.configure_advanced_audio().await?;
            }
            AutomationLevel::Professional => {
                // Full professional setup
                self.configuration.audio_config = self.configure_professional_audio().await?;
            }
        }
        
        self.update_progress(0.75, "✓ Audio devices configured").await;
        Ok(())
    }
    
    async fn configure_advanced_audio(&mut self) -> Result<AudioConfig, WizardError> {
        match self.platform {
            Platform::MacOS => {
                // Install and configure BlackHole
                self.install_blackhole().await?;
                self.setup_macos_multi_output().await?;
            }
            Platform::Windows => {
                // Install and configure VoiceMeeter
                self.install_voicemeeter().await?;
                self.setup_windows_routing().await?;
            }
            Platform::Linux => {
                // Configure PulseAudio
                self.setup_pulseaudio_routing().await?;
            }
        }
        
        Ok(AudioConfig::advanced_default(self.platform))
    }
    
    async fn install_blackhole(&mut self) -> Result<(), WizardError> {
        self.update_status("Installing BlackHole virtual audio device...").await;
        
        // Check if already installed
        if self.is_blackhole_installed().await? {
            self.update_status("✓ BlackHole already installed").await;
            return Ok(());
        }
        
        // Download and install
        let installer = BlackHoleInstaller::new();
        installer.download().await?;
        installer.install().await?;
        
        // Verify installation
        if !self.is_blackhole_installed().await? {
            return Err(WizardError::InstallationFailed("BlackHole".to_string()));
        }
        
        self.update_status("✓ BlackHole installed successfully").await;
        Ok(())
    }
    
    async fn setup_macos_multi_output(&mut self) -> Result<(), WizardError> {
        self.update_status("Creating Multi-Output Device...").await;
        
        let script = r#"
        tell application "Audio MIDI Setup"
            try
                set multiOutput to make new aggregate device
                set name of multiOutput to "Savant AI Multi-Output"
                set master of multiOutput to true
                
                -- Add Built-in Output
                set devices to audio devices
                repeat with device in devices
                    if name of device contains "Built-in Output" then
                        set subdevices of multiOutput to {device}
                        exit repeat
                    end if
                end repeat
                
                -- Add BlackHole
                repeat with device in devices
                    if name of device contains "BlackHole" then
                        set subdevices of multiOutput to (subdevices of multiOutput) & {device}
                        exit repeat
                    end if
                end repeat
                
                -- Set as default output
                set default output device to multiOutput
                
                return "success"
            on error errMsg
                return "error: " & errMsg
            end try
        end tell
        "#;
        
        let result = self.run_applescript(script).await?;
        if result.contains("error") {
            return Err(WizardError::ConfigurationFailed(result));
        }
        
        self.update_status("✓ Multi-Output Device created").await;
        Ok(())
    }
}
```

### Testing and Validation

```rust
// test_validation.rs
pub struct AudioVideoTester {
    config: AudioVideoSetup,
}

impl AudioVideoTester {
    pub async fn test_configuration(&self) -> Result<TestResults, TestError> {
        let mut results = TestResults::new();
        
        // Test audio input
        results.audio_input = self.test_audio_input().await?;
        
        // Test audio output
        results.audio_output = self.test_audio_output().await?;
        
        // Test system audio capture
        if self.config.audio_config.has_system_capture() {
            results.system_audio_capture = self.test_system_audio_capture().await?;
        }
        
        // Test screen capture
        results.screen_capture = self.test_screen_capture().await?;
        
        // Test OCR functionality
        results.ocr_functionality = self.test_ocr().await?;
        
        // Test multimodal sync
        results.multimodal_sync = self.test_multimodal_sync().await?;
        
        Ok(results)
    }
    
    async fn test_audio_input(&self) -> Result<TestResult, TestError> {
        let mut test_result = TestResult::new("Audio Input");
        
        // Record a short audio sample
        let recorder = AudioRecorder::new(self.config.audio_config.clone());
        let sample = recorder.record_sample(Duration::from_secs(2)).await?;
        
        // Analyze the sample
        let analysis = AudioAnalyzer::analyze(&sample)?;
        
        if analysis.has_signal {
            test_result.status = TestStatus::Passed;
            test_result.message = format!("Audio input working (level: {:.1}dB)", analysis.level);
        } else {
            test_result.status = TestStatus::Failed;
            test_result.message = "No audio signal detected".to_string();
        }
        
        Ok(test_result)
    }
    
    async fn test_screen_capture(&self) -> Result<TestResult, TestError> {
        let mut test_result = TestResult::new("Screen Capture");
        
        // Capture a test screenshot
        let capturer = ScreenCapturer::new(self.config.video_config.clone());
        let screenshot = capturer.capture_frame().await?;
        
        // Validate the screenshot
        if screenshot.width > 0 && screenshot.height > 0 && !screenshot.data.is_empty() {
            test_result.status = TestStatus::Passed;
            test_result.message = format!("Screen capture working ({}x{})", screenshot.width, screenshot.height);
        } else {
            test_result.status = TestStatus::Failed;
            test_result.message = "Screen capture failed".to_string();
        }
        
        Ok(test_result)
    }
    
    async fn test_system_audio_capture(&self) -> Result<TestResult, TestError> {
        let mut test_result = TestResult::new("System Audio Capture");
        
        // Play a test tone and try to capture it
        let tone_generator = ToneGenerator::new();
        let capture_task = self.capture_system_audio_sample();
        
        // Play test tone
        tone_generator.play_tone(440.0, Duration::from_secs(2)).await?;
        
        // Wait for capture
        let captured_audio = capture_task.await?;
        
        // Analyze for the test tone
        let analysis = AudioAnalyzer::analyze_for_tone(&captured_audio, 440.0)?;
        
        if analysis.tone_detected {
            test_result.status = TestStatus::Passed;
            test_result.message = "System audio capture working".to_string();
        } else {
            test_result.status = TestStatus::Failed;
            test_result.message = "System audio capture not working".to_string();
        }
        
        Ok(test_result)
    }
}
```

This comprehensive audio/video setup automation provides:

1. **Intelligent Detection**: Automatically detects system capabilities and optimal configurations
2. **Platform-Specific Setup**: Tailored setup procedures for macOS, Windows, and Linux
3. **Automated Installation**: Handles virtual audio devices and dependencies automatically
4. **Permission Management**: Guides users through complex permission requirements
5. **Configuration Validation**: Tests all components to ensure proper functionality
6. **Error Recovery**: Provides fallback options and troubleshooting guidance
7. **User-Friendly Interface**: Wizard-based setup with clear progress indicators
8. **Professional Options**: Advanced configurations for power users and professionals

The system ensures that users can achieve complex multimodal AI setups with minimal technical knowledge while providing the flexibility for advanced users to customize their configurations.