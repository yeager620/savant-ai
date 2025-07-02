# Daemon Management

Manage Savant AI audio and video capture daemons.

## Quick Commands

```bash
# Start all daemons
./start-daemons

# Stop all daemons  
./stop-daemons

# Monitor in real-time
./monitor-daemons

# Test all systems
./test-systems
```

## Individual Daemon Control

```bash
# Audio daemon
./sav start|stop|status|logs|test

# Video daemon (high-frequency monitoring)
./sav-video start --interval 500 --enable-ocr --enable-vision
./sav-video stop|status|logs|test
```

## Configuration

Edit `~/.config/savant-ai/config.toml`:

```toml
[video_capture]
interval_milliseconds = 500
enable_full_text_extraction = true
enable_real_time_analysis = true

[privacy]
blocked_applications = ["Zoom", "Teams"]
recording_schedule = "09:00-17:00"
```

## Troubleshooting

- Check permissions: `./verify-permissions`
- View logs: `./monitor-daemons`
- Test components: `./test-systems`

For complete documentation, see [CLAUDE.md](../../CLAUDE.md).