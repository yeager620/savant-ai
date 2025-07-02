# Savant AI Scripts Directory

This directory contains all utility scripts organized by functionality.

## Directory Structure

```
scripts/
├── setup/                     # Automated setup and installation
│   ├── automated-setup.sh     # Complete automated setup + guided permissions
│   ├── verify-system-permissions.sh # Comprehensive system verification
│   ├── permission-helper.sh   # Advanced permission troubleshooting
│   ├── auto-setup-system-audio.sh
│   ├── fixed-audio-daemon.sh
│   ├── package-distribution.sh
│   └── setup-system-audio.sh
├── daemon-management/          # Integrated daemon management
│   ├── start_all_daemons.sh   # Start all systems
│   ├── stop_all_daemons.sh    # Stop all systems
│   ├── restart_daemons.sh     # Restart all systems
│   ├── monitor_daemons.sh     # Real-time monitoring
│   ├── test_all_systems.sh    # System testing
│   └── README.md              # Daemon management docs
├── audio/                     # Audio-specific scripts
│   ├── audio-devices.sh       # Audio device management
│   ├── capture-system-audio.sh # System audio capture
│   ├── savant-audio-control.sh # Audio daemon control
│   └── savant-audio-daemon.sh  # Audio daemon runner
├── video/                     # Video-specific scripts
│   ├── savant-video-control.sh # Video daemon control
│   └── savant-video-daemon.sh  # Video daemon runner
├── tests/                     # Test scripts
│   ├── test-chatbot-integration.sh
│   ├── test-database-sql.sh
│   └── test-mcp-natural-queries.sh
└── utils/                     # Utility scripts
    └── update-paths.sh
```

## Quick Access

From the project root, use convenience commands:

```bash
# Setup & Verification
./setup             # Automated setup (dependencies + guided permissions)
./verify-permissions # Check system configuration

# Daemon Management  
./start-daemons     # Start all daemons
./stop-daemons      # Stop all daemons  
./monitor-daemons   # Monitor systems
./test-systems      # Test all components
```

## Direct Script Access

```bash
# Setup & Installation
./scripts/setup/automated-setup.sh
./scripts/setup/verify-system-permissions.sh
./scripts/setup/permission-helper.sh

# Daemon management
./scripts/daemon-management/start_all_daemons.sh
./scripts/daemon-management/monitor_daemons.sh

# Audio scripts
./scripts/audio/savant-audio-control.sh

# Video scripts  
./scripts/video/savant-video-control.sh

# Testing
./scripts/tests/test-mcp-natural-queries.sh
```

## Script Categories

### **Setup & Installation** (`setup/`)
Automated installation, system verification, and permission management.

**Key Scripts:**
- `automated-setup.sh` - Complete automated setup with guided permissions
- `verify-system-permissions.sh` - Comprehensive system verification
- `permission-helper.sh` - Advanced permission troubleshooting and reset

### **Daemon Management** (`daemon-management/`)
Complete system lifecycle management with dependency checking, status monitoring, and integrated testing.

### **Audio** (`audio/`)
Audio capture, device management, and daemon control scripts.

### **Video** (`video/`)
Screen capture and video analysis daemon management.

### **Tests** (`tests/`)
Integration tests for MCP, database, and multimodal functionality.

### **Utils** (`utils/`)
General utility scripts for maintenance and development.