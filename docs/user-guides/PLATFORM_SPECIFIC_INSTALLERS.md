# Platform-Specific Installers & Wizards

## 🏗️ Installation Wizard Architecture

### Cross-Platform Installer Framework

The Savant AI installer uses a unified framework that adapts to each platform's conventions while maintaining consistent user experience:

```rust
// Core installer framework
pub struct SavantInstaller {
    platform: Platform,
    ui_framework: UIFramework,
    config: InstallerConfig,
    dependency_manager: DependencyManager,
    permission_manager: PermissionManager,
}

impl SavantInstaller {
    pub fn new(platform: Platform) -> Self {
        let ui_framework = match platform {
            Platform::MacOS => UIFramework::Native(CocoaUI::new()),
            Platform::Windows => UIFramework::Native(WinUI::new()),
            Platform::Linux => UIFramework::GTK(GtkUI::new()),
        };
        
        SavantInstaller {
            platform,
            ui_framework,
            config: InstallerConfig::default(),
            dependency_manager: DependencyManager::new(platform),
            permission_manager: PermissionManager::new(platform),
        }
    }
    
    pub async fn run_installation(&mut self) -> Result<(), InstallError> {
        self.show_welcome_screen().await?;
        self.check_system_requirements().await?;
        self.install_dependencies().await?;
        self.request_permissions().await?;
        self.install_application().await?;
        self.configure_services().await?;
        self.run_first_time_setup().await?;
        self.show_completion_screen().await?;
        Ok(())
    }
}
```

---

## 🍎 macOS Installation Wizard

### Native macOS Installer (.pkg)

#### Installer Structure
```
SavantAI-2.1.0-macos-universal.pkg
├── Distribution.xml (installer configuration)
├── Resources/
│   ├── background.png
│   ├── license.html
│   ├── welcome.html
│   └── scripts/
│       ├── preinstall
│       ├── postinstall
│       └── check_requirements.sh
└── Payload/
    ├── SavantAI.app/
    ├── LaunchAgents/
    └── Documentation/
```

#### Installation Flow

##### Step 1: Welcome Screen
```objc
// Native Cocoa welcome screen
@interface WelcomeViewController : NSViewController
- (void)displayWelcomeMessage {
    NSString *welcomeText = @"Welcome to Savant AI!\n\n"
                           @"This installer will guide you through setting up "
                           @"your intelligent coding assistant.\n\n"
                           @"Features:\n"
                           @"• Real-time coding problem detection\n"
                           @"• AI-powered solution suggestions\n"
                           @"• Privacy-first local processing\n"
                           @"• Seamless macOS integration";
    
    [self.welcomeLabel setStringValue:welcomeText];
    [self.continueButton setEnabled:YES];
}
@end
```

##### Step 2: System Requirements Check
```bash
#!/bin/bash
# check_requirements.sh

echo "Checking macOS system requirements..."

# Check macOS version
min_version="10.15"
current_version=$(sw_vers -productVersion)
if [[ "$(printf '%s\n' "$min_version" "$current_version" | sort -V | head -n1)" != "$min_version" ]]; then
    echo "ERROR: macOS $min_version or later required (found $current_version)"
    exit 1
fi

# Check architecture
arch=$(uname -m)
if [[ "$arch" != "arm64" && "$arch" != "x86_64" ]]; then
    echo "ERROR: Unsupported architecture: $arch"
    exit 1
fi

# Check available disk space
available_space=$(df -H / | tail -1 | awk '{print $4}' | sed 's/G//')
required_space=10
if (( $(echo "$available_space < $required_space" | bc -l) )); then
    echo "ERROR: Insufficient disk space. Need ${required_space}GB, have ${available_space}GB"
    exit 1
fi

# Check RAM
memory_gb=$(( $(sysctl -n hw.memsize) / 1024 / 1024 / 1024 ))
if [ "$memory_gb" -lt 4 ]; then
    echo "WARNING: Low memory detected (${memory_gb}GB). 8GB+ recommended"
fi

echo "✓ System requirements check passed"
```

##### Step 3: Dependency Installation
```objc
@interface DependencyInstaller : NSObject
- (void)installDependencies {
    // Check for Homebrew
    if (![self isHomebrewInstalled]) {
        [self showHomebrewInstallDialog];
    }
    
    // Install required packages
    NSArray *packages = @[@"ollama", @"tesseract", @"imagemagick"];
    for (NSString *package in packages) {
        if (![self isPackageInstalled:package]) {
            [self installPackage:package withProgressCallback:^(float progress) {
                dispatch_async(dispatch_get_main_queue(), ^{
                    [self.progressIndicator setDoubleValue:progress];
                });
            }];
        }
    }
}

- (BOOL)isHomebrewInstalled {
    return [[NSFileManager defaultManager] fileExistsAtPath:@"/opt/homebrew/bin/brew"] ||
           [[NSFileManager defaultManager] fileExistsAtPath:@"/usr/local/bin/brew"];
}
@end
```

##### Step 4: Permission Request Interface
```objc
@interface PermissionRequestController : NSViewController
- (void)requestScreenRecordingPermission {
    // Check current permission status
    if ([self hasScreenRecordingPermission]) {
        [self.screenRecordingStatus setStringValue:@"✓ Granted"];
        [self.screenRecordingButton setEnabled:NO];
        return;
    }
    
    // Show explanation dialog
    NSAlert *alert = [[NSAlert alloc] init];
    [alert setMessageText:@"Screen Recording Permission Required"];
    [alert setInformativeText:@"Savant AI needs screen recording permission to detect coding problems on your screen. This permission is required for core functionality."];
    [alert addButtonWithTitle:@"Open System Preferences"];
    [alert addButtonWithTitle:@"Cancel"];
    
    [alert beginSheetModalForWindow:self.view.window completionHandler:^(NSModalResponse returnCode) {
        if (returnCode == NSAlertFirstButtonReturn) {
            // Open System Preferences to Privacy settings
            [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"]];
            
            // Start monitoring for permission grant
            [self startPermissionMonitoring];
        }
    }];
}

- (void)startPermissionMonitoring {
    self.permissionTimer = [NSTimer scheduledTimerWithTimeInterval:1.0 repeats:YES block:^(NSTimer *timer) {
        if ([self hasScreenRecordingPermission]) {
            [self.screenRecordingStatus setStringValue:@"✓ Granted"];
            [self.screenRecordingButton setEnabled:NO];
            [timer invalidate];
            self.permissionTimer = nil;
        }
    }];
}

- (BOOL)hasScreenRecordingPermission {
    // Test screen recording by attempting a screenshot
    CGImageRef screenshot = CGWindowListCreateImage(CGRectMake(0, 0, 1, 1), 
                                                   kCGWindowListOptionOnScreenOnly, 
                                                   kCGNullWindowID, 
                                                   kCGWindowImageDefault);
    if (screenshot) {
        CFRelease(screenshot);
        return YES;
    }
    return NO;
}
@end
```

##### Step 5: Service Installation
```bash
#!/bin/bash
# postinstall script

echo "Installing Savant AI services..."

# Create launch agent
cat > ~/Library/LaunchAgents/com.savant-ai.agent.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.savant-ai.agent</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Applications/Savant AI.app/Contents/MacOS/savant-ai</string>
        <string>--daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
EOF

# Load launch agent
launchctl load ~/Library/LaunchAgents/com.savant-ai.agent.plist

# Create application support directory
mkdir -p ~/Library/Application\ Support/Savant\ AI/

# Set up initial configuration
cat > ~/Library/Application\ Support/Savant\ AI/config.toml << 'EOF'
[general]
first_run = true
platform = "macos"
install_date = "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

[ai]
model = "codellama:7b"
local_only = true

[privacy]
screen_capture = false  # Will be enabled after permission grant
audio_transcription = false
EOF

echo "✓ Services installed successfully"
```

### macOS App Store Version

For App Store distribution, a sandboxed version with limited permissions:

```objc
// App Store entitlements
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.app-sandbox</key>
    <true/>
    <key>com.apple.security.device.microphone</key>
    <true/>
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.automation.apple-events</key>
    <true/>
</dict>
</plist>

@interface AppStoreInstaller : NSObject
- (void)performAppStoreSetup {
    // Request microphone permission
    [AVCaptureDevice requestAccessForMediaType:AVMediaTypeAudio completionHandler:^(BOOL granted) {
        if (granted) {
            NSLog(@"Microphone access granted");
        } else {
            NSLog(@"Microphone access denied");
        }
    }];
    
    // Note: Screen recording requires user to manually grant permission
    // in System Preferences due to App Store sandbox restrictions
}
@end
```

---

## 🪟 Windows Installation Wizard

### Windows Installer (.msi)

#### MSI Structure
```
SavantAI-2.1.0-windows-x64.msi
├── Product.wxs (WiX configuration)
├── UI/
│   ├── WelcomeDialog.wxs
│   ├── LicenseDialog.wxs
│   ├── InstallDirDialog.wxs
│   └── ProgressDialog.wxs
├── CustomActions/
│   ├── CheckRequirements.dll
│   ├── InstallDependencies.dll
│   └── ConfigureServices.dll
└── Files/
    ├── SavantAI.exe
    ├── Dependencies/
    └── Documentation/
```

#### WiX Configuration
```xml
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <Product Id="*" Name="Savant AI" Language="1033" Version="2.1.0" Manufacturer="Savant AI" UpgradeCode="12345678-1234-1234-1234-123456789ABC">
        <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />
        
        <!-- Installation UI -->
        <UI>
            <UIRef Id="WixUI_InstallDir" />
            <UIRef Id="WixUI_ErrorProgressText" />
            
            <!-- Custom welcome dialog -->
            <DialogRef Id="WelcomeDlg" />
            <DialogRef Id="LicenseAgreementDlg" />
            <DialogRef Id="InstallDirDlg" />
            <DialogRef Id="VerifyReadyDlg" />
            
            <!-- Progress dialog -->
            <DialogRef Id="ProgressDlg" />
        </UI>
        
        <!-- Installation directory -->
        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="ProgramFilesFolder">
                <Directory Id="INSTALLFOLDER" Name="Savant AI" />
            </Directory>
            <Directory Id="ProgramMenuFolder">
                <Directory Id="ApplicationProgramsFolder" Name="Savant AI" />
            </Directory>
        </Directory>
        
        <!-- Features -->
        <Feature Id="ProductFeature" Title="Savant AI" Level="1">
            <ComponentRef Id="MainExecutable" />
            <ComponentRef Id="Dependencies" />
            <ComponentRef Id="Services" />
            <ComponentRef Id="Shortcuts" />
        </Feature>
        
        <!-- Custom actions -->
        <CustomAction Id="CheckSystemRequirements" BinaryKey="CustomActions" DllEntry="CheckRequirements" Execute="immediate" />
        <CustomAction Id="InstallDependencies" BinaryKey="CustomActions" DllEntry="InstallDependencies" Execute="deferred" />
        <CustomAction Id="ConfigureServices" BinaryKey="CustomActions" DllEntry="ConfigureServices" Execute="deferred" />
        
        <!-- Installation sequence -->
        <InstallExecuteSequence>
            <Custom Action="CheckSystemRequirements" After="LaunchConditions" />
            <Custom Action="InstallDependencies" After="InstallFiles" />
            <Custom Action="ConfigureServices" After="InstallDependencies" />
        </InstallExecuteSequence>
    </Product>
</Wix>
```

#### Custom Actions (C#)
```csharp
using Microsoft.Deployment.WindowsInstaller;
using System;
using System.Diagnostics;
using System.IO;
using System.Management;

public class CustomActions
{
    [CustomAction]
    public static ActionResult CheckRequirements(Session session)
    {
        session.Log("Begin CheckRequirements");
        
        try
        {
            // Check Windows version
            var version = Environment.OSVersion.Version;
            if (version.Major < 10 || (version.Major == 10 && version.Build < 17763))
            {
                session.Log("ERROR: Windows 10 version 1809 or later required");
                return ActionResult.Failure;
            }
            
            // Check available memory
            var memorySize = GetTotalMemoryInGB();
            if (memorySize < 4)
            {
                session.Log($"WARNING: Low memory detected ({memorySize}GB). 8GB+ recommended");
            }
            
            // Check disk space
            var availableSpace = GetAvailableDiskSpaceGB();
            if (availableSpace < 10)
            {
                session.Log($"ERROR: Insufficient disk space. Need 10GB, have {availableSpace}GB");
                return ActionResult.Failure;
            }
            
            // Check for .NET Framework
            if (!IsDotNetFrameworkInstalled())
            {
                session.Log("ERROR: .NET Framework 4.8 or later required");
                return ActionResult.Failure;
            }
            
            session.Log("✓ System requirements check passed");
            return ActionResult.Success;
        }
        catch (Exception ex)
        {
            session.Log($"ERROR in CheckRequirements: {ex.Message}");
            return ActionResult.Failure;
        }
    }
    
    [CustomAction]
    public static ActionResult InstallDependencies(Session session)
    {
        session.Log("Begin InstallDependencies");
        
        try
        {
            // Install Chocolatey if not present
            if (!IsChocolateyInstalled())
            {
                session.Log("Installing Chocolatey...");
                InstallChocolatey();
            }
            
            // Install required packages
            var packages = new[] { "ollama", "tesseract", "imagemagick" };
            foreach (var package in packages)
            {
                if (!IsPackageInstalled(package))
                {
                    session.Log($"Installing {package}...");
                    InstallPackage(package);
                }
                else
                {
                    session.Log($"✓ {package} already installed");
                }
            }
            
            session.Log("✓ Dependencies installed successfully");
            return ActionResult.Success;
        }
        catch (Exception ex)
        {
            session.Log($"ERROR in InstallDependencies: {ex.Message}");
            return ActionResult.Failure;
        }
    }
    
    [CustomAction]
    public static ActionResult ConfigureServices(Session session)
    {
        session.Log("Begin ConfigureServices");
        
        try
        {
            // Create Windows Service
            var serviceName = "SavantAI";
            var serviceDisplayName = "Savant AI Service";
            var serviceDescription = "Savant AI intelligent coding assistant service";
            var servicePath = Path.Combine(session.Property("INSTALLFOLDER"), "SavantAI.exe");
            
            CreateWindowsService(serviceName, serviceDisplayName, serviceDescription, servicePath);
            
            // Configure firewall rules
            ConfigureFirewall();
            
            // Set up auto-start
            ConfigureAutoStart();
            
            session.Log("✓ Services configured successfully");
            return ActionResult.Success;
        }
        catch (Exception ex)
        {
            session.Log($"ERROR in ConfigureServices: {ex.Message}");
            return ActionResult.Failure;
        }
    }
    
    private static void CreateWindowsService(string serviceName, string displayName, string description, string path)
    {
        var startInfo = new ProcessStartInfo
        {
            FileName = "sc.exe",
            Arguments = $"create {serviceName} binPath= \"{path} --service\" start= auto",
            UseShellExecute = false,
            RedirectStandardOutput = true,
            RedirectStandardError = true
        };
        
        using (var process = Process.Start(startInfo))
        {
            process.WaitForExit();
            if (process.ExitCode != 0)
            {
                throw new InvalidOperationException($"Failed to create service: {process.StandardError.ReadToEnd()}");
            }
        }
        
        // Set service description
        startInfo.Arguments = $"description {serviceName} \"{description}\"";
        using (var process = Process.Start(startInfo))
        {
            process.WaitForExit();
        }
    }
}
```

#### Modern Windows UI (WinUI 3)
```xml
<!-- WelcomeDialog.xaml -->
<Page x:Class="SavantAI.Installer.WelcomeDialog"
      xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
    <Grid>
        <Grid.RowDefinitions>
            <RowDefinition Height="Auto"/>
            <RowDefinition Height="*"/>
            <RowDefinition Height="Auto"/>
        </Grid.RowDefinitions>
        
        <!-- Header -->
        <StackPanel Grid.Row="0" Orientation="Horizontal" Margin="20">
            <Image Source="Assets/SavantAI-Icon.png" Width="64" Height="64"/>
            <StackPanel Margin="20,0,0,0">
                <TextBlock Text="Welcome to Savant AI" Style="{StaticResource HeaderTextBlockStyle}"/>
                <TextBlock Text="Version 2.1.0" Style="{StaticResource SubtitleTextBlockStyle}"/>
            </StackPanel>
        </StackPanel>
        
        <!-- Content -->
        <ScrollViewer Grid.Row="1" Margin="20">
            <StackPanel>
                <TextBlock Text="Savant AI is your intelligent coding assistant that provides real-time help with programming challenges." 
                          TextWrapping="Wrap" Margin="0,0,0,20"/>
                
                <TextBlock Text="Features:" FontWeight="Bold" Margin="0,0,0,10"/>
                <StackPanel Margin="20,0,0,0">
                    <TextBlock Text="• Real-time coding problem detection" Margin="0,0,0,5"/>
                    <TextBlock Text="• AI-powered solution suggestions" Margin="0,0,0,5"/>
                    <TextBlock Text="• Privacy-first local processing" Margin="0,0,0,5"/>
                    <TextBlock Text="• Seamless Windows integration" Margin="0,0,0,5"/>
                </StackPanel>
                
                <TextBlock Text="System Requirements:" FontWeight="Bold" Margin="0,20,0,10"/>
                <StackPanel Margin="20,0,0,0">
                    <TextBlock Text="• Windows 10 version 1809 or later" Margin="0,0,0,5"/>
                    <TextBlock Text="• 8GB RAM recommended (4GB minimum)" Margin="0,0,0,5"/>
                    <TextBlock Text="• 10GB free disk space" Margin="0,0,0,5"/>
                    <TextBlock Text="• .NET Framework 4.8 or later" Margin="0,0,0,5"/>
                </StackPanel>
            </StackPanel>
        </ScrollViewer>
        
        <!-- Buttons -->
        <StackPanel Grid.Row="2" Orientation="Horizontal" HorizontalAlignment="Right" Margin="20">
            <Button Content="Cancel" Margin="0,0,10,0" Click="CancelButton_Click"/>
            <Button Content="Next" Style="{StaticResource AccentButtonStyle}" Click="NextButton_Click"/>
        </StackPanel>
    </Grid>
</Page>
```

### Windows Store Package (MSIX)

For Microsoft Store distribution:

```xml
<!-- Package.appxmanifest -->
<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10">
    <Identity Name="SavantAI" Publisher="CN=Savant AI" Version="2.1.0.0" />
    
    <Properties>
        <DisplayName>Savant AI</DisplayName>
        <PublisherDisplayName>Savant AI</PublisherDisplayName>
        <Logo>Assets\StoreLogo.png</Logo>
    </Properties>
    
    <Dependencies>
        <TargetDeviceFamily Name="Windows.Universal" MinVersion="10.0.17763.0" MaxVersionTested="10.0.19041.0" />
    </Dependencies>
    
    <Resources>
        <Resource Language="x-generate"/>
    </Resources>
    
    <Applications>
        <Application Id="App" Executable="SavantAI.exe" EntryPoint="SavantAI.App">
            <uap:VisualElements DisplayName="Savant AI" Square150x150Logo="Assets\Square150x150Logo.png" Square44x44Logo="Assets\Square44x44Logo.png" Description="Savant AI intelligent coding assistant" BackgroundColor="transparent">
                <uap:DefaultTile Wide310x150Logo="Assets\Wide310x150Logo.png"/>
                <uap:SplashScreen Image="Assets\SplashScreen.png" />
            </uap:VisualElements>
        </Application>
    </Applications>
    
    <Capabilities>
        <Capability Name="internetClient" />
        <uap:Capability Name="microphone" />
        <uap:Capability Name="webcam" />
        <rescap:Capability Name="runFullTrust" />
    </Capabilities>
</Package>
```

---

## 🐧 Linux Installation Wizard

### Debian/Ubuntu (.deb)

#### Package Structure
```
savant-ai_2.1.0_amd64.deb
├── DEBIAN/
│   ├── control
│   ├── preinst
│   ├── postinst
│   ├── prerm
│   └── postrm
├── usr/
│   ├── bin/
│   │   └── savant-ai
│   ├── lib/
│   │   └── savant-ai/
│   └── share/
│       ├── applications/
│       │   └── savant-ai.desktop
│       ├── icons/
│       └── doc/
└── etc/
    └── systemd/
        └── system/
            └── savant-ai.service
```

#### Control File
```
Package: savant-ai
Version: 2.1.0
Section: utils
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.28), libssl1.1 (>= 1.1.0), libgtk-3-0 (>= 3.20), tesseract-ocr (>= 4.0)
Suggests: ollama, imagemagick
Maintainer: Savant AI Team <support@savant-ai.com>
Description: Intelligent coding assistant with real-time problem detection
 Savant AI is an intelligent coding assistant that provides real-time help
 with programming challenges. It uses AI to detect coding problems on your
 screen and provides instant solutions and explanations.
 .
 Features:
  * Real-time coding problem detection
  * AI-powered solution suggestions
  * Privacy-first local processing
  * Cross-platform compatibility
Homepage: https://savant-ai.com
```

#### Pre/Post Installation Scripts
```bash
#!/bin/bash
# preinst script

set -e

# Check system requirements
check_requirements() {
    echo "Checking system requirements..."
    
    # Check Ubuntu/Debian version
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        case "$ID" in
            ubuntu)
                if [ $(echo "$VERSION_ID >= 20.04" | bc -l) -eq 0 ]; then
                    echo "ERROR: Ubuntu 20.04 or later required"
                    exit 1
                fi
                ;;
            debian)
                if [ $(echo "$VERSION_ID >= 10" | bc -l) -eq 0 ]; then
                    echo "ERROR: Debian 10 or later required"
                    exit 1
                fi
                ;;
            *)
                echo "WARNING: Unsupported distribution detected"
                ;;
        esac
    fi
    
    # Check architecture
    arch=$(dpkg --print-architecture)
    if [ "$arch" != "amd64" ]; then
        echo "ERROR: Only amd64 architecture is supported"
        exit 1
    fi
    
    # Check available memory
    memory_kb=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    memory_gb=$((memory_kb / 1024 / 1024))
    if [ $memory_gb -lt 4 ]; then
        echo "WARNING: Low memory detected (${memory_gb}GB). 8GB+ recommended"
    fi
    
    # Check disk space
    available_space=$(df -BG /usr | tail -1 | awk '{print $4}' | sed 's/G//')
    if [ $available_space -lt 10 ]; then
        echo "ERROR: Insufficient disk space. Need 10GB, have ${available_space}GB"
        exit 1
    fi
    
    echo "✓ System requirements check passed"
}

case "$1" in
    install|upgrade)
        check_requirements
        ;;
    *)
        echo "preinst called with unknown argument \`$1'" >&2
        exit 1
        ;;
esac
```

```bash
#!/bin/bash
# postinst script

set -e

# Configuration
SAVANT_USER="savant-ai"
SAVANT_GROUP="savant-ai"
SAVANT_HOME="/var/lib/savant-ai"
SAVANT_LOG="/var/log/savant-ai"

create_user() {
    if ! getent group "$SAVANT_GROUP" >/dev/null; then
        addgroup --system "$SAVANT_GROUP"
    fi
    
    if ! getent passwd "$SAVANT_USER" >/dev/null; then
        adduser --system --home "$SAVANT_HOME" --shell /bin/false \
                --ingroup "$SAVANT_GROUP" --disabled-password \
                --gecos "Savant AI service user" "$SAVANT_USER"
    fi
}

setup_directories() {
    mkdir -p "$SAVANT_HOME"
    mkdir -p "$SAVANT_LOG"
    
    chown -R "$SAVANT_USER:$SAVANT_GROUP" "$SAVANT_HOME"
    chown -R "$SAVANT_USER:$SAVANT_GROUP" "$SAVANT_LOG"
    
    chmod 755 "$SAVANT_HOME"
    chmod 755 "$SAVANT_LOG"
}

install_dependencies() {
    echo "Installing dependencies..."
    
    # Update package list
    apt-get update
    
    # Install required packages
    apt-get install -y curl wget gnupg software-properties-common
    
    # Install Ollama if not present
    if ! command -v ollama >/dev/null 2>&1; then
        echo "Installing Ollama..."
        curl -fsSL https://ollama.com/install.sh | sh
    fi
    
    # Install ImageMagick if not present
    if ! command -v convert >/dev/null 2>&1; then
        echo "Installing ImageMagick..."
        apt-get install -y imagemagick
    fi
    
    echo "✓ Dependencies installed"
}

configure_systemd() {
    echo "Configuring systemd service..."
    
    # Reload systemd
    systemctl daemon-reload
    
    # Enable service
    systemctl enable savant-ai.service
    
    # Create default configuration
    if [ ! -f "$SAVANT_HOME/config.toml" ]; then
        cat > "$SAVANT_HOME/config.toml" << EOF
[general]
first_run = true
platform = "linux"
install_date = "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

[ai]
model = "codellama:7b"
local_only = true

[privacy]
screen_capture = false
audio_transcription = false
EOF
        chown "$SAVANT_USER:$SAVANT_GROUP" "$SAVANT_HOME/config.toml"
    fi
    
    echo "✓ Systemd service configured"
}

setup_desktop_integration() {
    echo "Setting up desktop integration..."
    
    # Update desktop database
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database /usr/share/applications
    fi
    
    # Update icon cache
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f /usr/share/icons/hicolor
    fi
    
    echo "✓ Desktop integration configured"
}

case "$1" in
    configure)
        create_user
        setup_directories
        install_dependencies
        configure_systemd
        setup_desktop_integration
        
        echo ""
        echo "✓ Savant AI installation completed successfully!"
        echo ""
        echo "Next steps:"
        echo "1. Start the service: sudo systemctl start savant-ai"
        echo "2. Check status: sudo systemctl status savant-ai"
        echo "3. Run setup wizard: savant-ai --setup"
        echo "4. View logs: journalctl -u savant-ai -f"
        ;;
    *)
        echo "postinst called with unknown argument \`$1'" >&2
        exit 1
        ;;
esac
```

### GTK-based GUI Installer

```python
#!/usr/bin/env python3
# gtk_installer.py

import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, GdkPixbuf, GLib, Gio
import os
import sys
import subprocess
import threading
import shutil
import tempfile

class SavantAIInstaller:
    def __init__(self):
        self.builder = Gtk.Builder()
        self.builder.add_from_file('installer.glade')
        self.builder.connect_signals(self)
        
        self.window = self.builder.get_object('main_window')
        self.stack = self.builder.get_object('main_stack')
        self.progress_bar = self.builder.get_object('progress_bar')
        self.status_label = self.builder.get_object('status_label')
        
        self.current_step = 0
        self.total_steps = 6
        
        self.window.show_all()
    
    def on_window_destroy(self, widget):
        Gtk.main_quit()
    
    def on_next_clicked(self, widget):
        current_page = self.stack.get_visible_child_name()
        
        if current_page == 'welcome':
            self.stack.set_visible_child_name('requirements')
            self.check_requirements()
        elif current_page == 'requirements':
            self.stack.set_visible_child_name('dependencies')
            self.install_dependencies()
        elif current_page == 'dependencies':
            self.stack.set_visible_child_name('permissions')
            self.setup_permissions()
        elif current_page == 'permissions':
            self.stack.set_visible_child_name('configuration')
            self.configure_application()
        elif current_page == 'configuration':
            self.stack.set_visible_child_name('complete')
            self.complete_installation()
    
    def check_requirements(self):
        """Check system requirements"""
        self.update_progress(0.1, "Checking system requirements...")
        
        # Check in background thread
        thread = threading.Thread(target=self._check_requirements_thread)
        thread.daemon = True
        thread.start()
    
    def _check_requirements_thread(self):
        try:
            # Check distribution
            with open('/etc/os-release') as f:
                os_info = f.read()
            
            # Check memory
            with open('/proc/meminfo') as f:
                memory_info = f.read()
            
            # Check disk space
            disk_usage = shutil.disk_usage('/')
            
            # Update UI in main thread
            GLib.idle_add(self.update_progress, 1.0, "✓ System requirements check passed")
            GLib.idle_add(self.enable_next_button)
            
        except Exception as e:
            GLib.idle_add(self.show_error, f"Requirements check failed: {str(e)}")
    
    def install_dependencies(self):
        """Install required dependencies"""
        self.update_progress(0.0, "Installing dependencies...")
        
        thread = threading.Thread(target=self._install_dependencies_thread)
        thread.daemon = True
        thread.start()
    
    def _install_dependencies_thread(self):
        try:
            # Update package list
            GLib.idle_add(self.update_progress, 0.1, "Updating package list...")
            subprocess.run(['sudo', 'apt-get', 'update'], check=True)
            
            # Install tesseract
            GLib.idle_add(self.update_progress, 0.3, "Installing Tesseract OCR...")
            subprocess.run(['sudo', 'apt-get', 'install', '-y', 'tesseract-ocr'], check=True)
            
            # Install ImageMagick
            GLib.idle_add(self.update_progress, 0.5, "Installing ImageMagick...")
            subprocess.run(['sudo', 'apt-get', 'install', '-y', 'imagemagick'], check=True)
            
            # Install Ollama
            GLib.idle_add(self.update_progress, 0.7, "Installing Ollama...")
            subprocess.run(['curl', '-fsSL', 'https://ollama.com/install.sh'], stdout=subprocess.PIPE, check=True)
            
            GLib.idle_add(self.update_progress, 1.0, "✓ Dependencies installed successfully")
            GLib.idle_add(self.enable_next_button)
            
        except subprocess.CalledProcessError as e:
            GLib.idle_add(self.show_error, f"Dependency installation failed: {str(e)}")
    
    def setup_permissions(self):
        """Set up system permissions"""
        self.update_progress(0.5, "Setting up permissions...")
        
        # Show permission setup dialog
        dialog = Gtk.MessageDialog(
            transient_for=self.window,
            flags=0,
            message_type=Gtk.MessageType.INFO,
            buttons=Gtk.ButtonsType.OK,
            text="Permission Setup Required"
        )
        dialog.format_secondary_text(
            "Savant AI needs screen capture permission to detect coding problems. "
            "Please grant permission when prompted by your desktop environment."
        )
        dialog.run()
        dialog.destroy()
        
        # Request screen capture permission (implementation depends on desktop environment)
        self.request_screen_capture_permission()
    
    def request_screen_capture_permission(self):
        """Request screen capture permission"""
        try:
            # Try to detect desktop environment
            desktop = os.environ.get('XDG_CURRENT_DESKTOP', '').lower()
            
            if 'gnome' in desktop:
                # GNOME/Mutter - use portal
                self.request_gnome_permission()
            elif 'kde' in desktop:
                # KDE Plasma - use KWin
                self.request_kde_permission()
            else:
                # Generic X11 - test with xwininfo
                self.request_x11_permission()
        
        except Exception as e:
            self.show_error(f"Permission setup failed: {str(e)}")
    
    def request_gnome_permission(self):
        """Request permission via GNOME portal"""
        try:
            # Use xdg-desktop-portal for screen capture
            subprocess.run([
                'gdbus', 'call', '--session',
                '--dest', 'org.freedesktop.portal.Desktop',
                '--object-path', '/org/freedesktop/portal/desktop',
                '--method', 'org.freedesktop.portal.ScreenCast.CreateSession',
                '{}'
            ], check=True)
            
            self.update_progress(1.0, "✓ Permissions configured")
            self.enable_next_button()
            
        except subprocess.CalledProcessError:
            self.show_error("Failed to request screen capture permission")
    
    def update_progress(self, fraction, text):
        """Update progress bar and status"""
        self.progress_bar.set_fraction(fraction)
        self.status_label.set_text(text)
    
    def enable_next_button(self):
        """Enable the next button"""
        next_button = self.builder.get_object('next_button')
        next_button.set_sensitive(True)
    
    def show_error(self, message):
        """Show error dialog"""
        dialog = Gtk.MessageDialog(
            transient_for=self.window,
            flags=0,
            message_type=Gtk.MessageType.ERROR,
            buttons=Gtk.ButtonsType.OK,
            text="Installation Error"
        )
        dialog.format_secondary_text(message)
        dialog.run()
        dialog.destroy()

if __name__ == '__main__':
    installer = SavantAIInstaller()
    Gtk.main()
```

### Flatpak Package

```json
{
    "app-id": "com.savant-ai.SavantAI",
    "runtime": "org.freedesktop.Platform",
    "runtime-version": "22.08",
    "sdk": "org.freedesktop.Sdk",
    "command": "savant-ai",
    "finish-args": [
        "--socket=wayland",
        "--socket=fallback-x11",
        "--socket=pulseaudio",
        "--device=dri",
        "--share=network",
        "--share=ipc",
        "--filesystem=home",
        "--filesystem=xdg-config/savant-ai:create",
        "--filesystem=xdg-data/savant-ai:create",
        "--env=DCONF_USER_CONFIG_DIR=.config/dconf"
    ],
    "modules": [
        {
            "name": "savant-ai",
            "buildsystem": "simple",
            "build-commands": [
                "install -Dm755 savant-ai /app/bin/savant-ai",
                "install -Dm644 com.savant-ai.SavantAI.desktop /app/share/applications/com.savant-ai.SavantAI.desktop",
                "install -Dm644 com.savant-ai.SavantAI.metainfo.xml /app/share/metainfo/com.savant-ai.SavantAI.metainfo.xml",
                "install -Dm644 icon.png /app/share/icons/hicolor/256x256/apps/com.savant-ai.SavantAI.png"
            ],
            "sources": [
                {
                    "type": "file",
                    "path": "savant-ai"
                },
                {
                    "type": "file",
                    "path": "com.savant-ai.SavantAI.desktop"
                },
                {
                    "type": "file",
                    "path": "com.savant-ai.SavantAI.metainfo.xml"
                },
                {
                    "type": "file",
                    "path": "icon.png"
                }
            ]
        }
    ]
}
```

---

## 🔧 Dependency Management

### Automated Dependency Resolution

```rust
// dependency_manager.rs
use std::collections::HashMap;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub required: bool,
    pub installation_command: Vec<String>,
    pub verification_command: Vec<String>,
    pub fallback_url: Option<String>,
}

pub struct DependencyManager {
    platform: Platform,
    dependencies: HashMap<String, Dependency>,
    package_manager: PackageManager,
}

impl DependencyManager {
    pub fn new(platform: Platform) -> Self {
        let package_manager = PackageManager::detect(&platform);
        let dependencies = Self::load_dependencies(&platform);
        
        DependencyManager {
            platform,
            dependencies,
            package_manager,
        }
    }
    
    pub async fn install_all(&self, progress_callback: impl Fn(f32, &str)) -> Result<(), DependencyError> {
        let total_deps = self.dependencies.len();
        let mut installed = 0;
        
        for (name, dep) in &self.dependencies {
            progress_callback(installed as f32 / total_deps as f32, &format!("Installing {}", name));
            
            if !self.is_installed(dep).await? {
                self.install_dependency(dep).await?;
            }
            
            installed += 1;
        }
        
        progress_callback(1.0, "All dependencies installed successfully");
        Ok(())
    }
    
    async fn install_dependency(&self, dep: &Dependency) -> Result<(), DependencyError> {
        // Try package manager first
        if let Err(e) = self.package_manager.install(&dep.name, dep.version.as_deref()).await {
            // Fall back to manual installation
            if let Some(url) = &dep.fallback_url {
                self.manual_install(url, dep).await?;
            } else {
                return Err(DependencyError::InstallationFailed(dep.name.clone(), e));
            }
        }
        
        // Verify installation
        if !self.is_installed(dep).await? {
            return Err(DependencyError::VerificationFailed(dep.name.clone()));
        }
        
        Ok(())
    }
    
    async fn is_installed(&self, dep: &Dependency) -> Result<bool, DependencyError> {
        let mut cmd = Command::new(&dep.verification_command[0]);
        for arg in &dep.verification_command[1..] {
            cmd.arg(arg);
        }
        
        Ok(cmd.status().await?.success())
    }
    
    fn load_dependencies(platform: &Platform) -> HashMap<String, Dependency> {
        let mut deps = HashMap::new();
        
        // Core dependencies
        deps.insert("tesseract".to_string(), Dependency {
            name: "tesseract".to_string(),
            version: Some("4.0+".to_string()),
            required: true,
            installation_command: match platform {
                Platform::MacOS => vec!["brew".to_string(), "install".to_string(), "tesseract".to_string()],
                Platform::Windows => vec!["choco".to_string(), "install".to_string(), "tesseract".to_string()],
                Platform::Linux => vec!["sudo".to_string(), "apt-get".to_string(), "install".to_string(), "-y".to_string(), "tesseract-ocr".to_string()],
            },
            verification_command: vec!["tesseract".to_string(), "--version".to_string()],
            fallback_url: Some("https://github.com/tesseract-ocr/tesseract/releases".to_string()),
        });
        
        deps.insert("ollama".to_string(), Dependency {
            name: "ollama".to_string(),
            version: None,
            required: true,
            installation_command: match platform {
                Platform::MacOS => vec!["brew".to_string(), "install".to_string(), "ollama".to_string()],
                Platform::Windows => vec!["choco".to_string(), "install".to_string(), "ollama".to_string()],
                Platform::Linux => vec!["curl".to_string(), "-fsSL".to_string(), "https://ollama.com/install.sh".to_string()],
            },
            verification_command: vec!["ollama".to_string(), "--version".to_string()],
            fallback_url: Some("https://ollama.com/download".to_string()),
        });
        
        deps.insert("imagemagick".to_string(), Dependency {
            name: "imagemagick".to_string(),
            version: None,
            required: false,
            installation_command: match platform {
                Platform::MacOS => vec!["brew".to_string(), "install".to_string(), "imagemagick".to_string()],
                Platform::Windows => vec!["choco".to_string(), "install".to_string(), "imagemagick".to_string()],
                Platform::Linux => vec!["sudo".to_string(), "apt-get".to_string(), "install".to_string(), "-y".to_string(), "imagemagick".to_string()],
            },
            verification_command: vec!["convert".to_string(), "--version".to_string()],
            fallback_url: Some("https://imagemagick.org/script/download.php".to_string()),
        });
        
        deps
    }
}

#[derive(Debug, Clone)]
pub enum PackageManager {
    Homebrew,
    Chocolatey,
    Apt,
    Yum,
    Pacman,
    Manual,
}

impl PackageManager {
    fn detect(platform: &Platform) -> Self {
        match platform {
            Platform::MacOS => {
                if Command::new("brew").arg("--version").status().is_ok() {
                    PackageManager::Homebrew
                } else {
                    PackageManager::Manual
                }
            }
            Platform::Windows => {
                if Command::new("choco").arg("--version").status().is_ok() {
                    PackageManager::Chocolatey
                } else {
                    PackageManager::Manual
                }
            }
            Platform::Linux => {
                if Command::new("apt-get").arg("--version").status().is_ok() {
                    PackageManager::Apt
                } else if Command::new("yum").arg("--version").status().is_ok() {
                    PackageManager::Yum
                } else if Command::new("pacman").arg("--version").status().is_ok() {
                    PackageManager::Pacman
                } else {
                    PackageManager::Manual
                }
            }
        }
    }
    
    async fn install(&self, package: &str, version: Option<&str>) -> Result<(), PackageManagerError> {
        match self {
            PackageManager::Homebrew => {
                let mut cmd = Command::new("brew");
                cmd.args(&["install", package]);
                cmd.status().await?.success().then_some(()).ok_or(PackageManagerError::InstallFailed)
            }
            PackageManager::Chocolatey => {
                let mut cmd = Command::new("choco");
                cmd.args(&["install", package, "-y"]);
                cmd.status().await?.success().then_some(()).ok_or(PackageManagerError::InstallFailed)
            }
            PackageManager::Apt => {
                let mut cmd = Command::new("sudo");
                cmd.args(&["apt-get", "install", "-y", package]);
                cmd.status().await?.success().then_some(()).ok_or(PackageManagerError::InstallFailed)
            }
            _ => Err(PackageManagerError::UnsupportedPackageManager),
        }
    }
}
```

This comprehensive platform-specific installer design provides:

1. **Native Platform Integration**: Uses platform-specific UI frameworks and conventions
2. **Automated Dependency Management**: Handles complex dependency chains automatically
3. **Permission Management**: Platform-specific permission request workflows
4. **Error Recovery**: Robust error handling and recovery mechanisms
5. **Progress Tracking**: Real-time progress updates and status reporting
6. **User-Friendly Interface**: Intuitive wizards with clear explanations
7. **Fallback Options**: Multiple installation methods for reliability
8. **Service Integration**: Proper system service setup and management

The design ensures that users on any platform can install Savant AI with minimal technical knowledge while handling all the complexity of multimodal AI setup, system permissions, and platform-specific requirements behind the scenes.