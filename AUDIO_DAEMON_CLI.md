Updated Audio Daemon Command Interface

  Primary Control Script

  # Main control interface
  ./scripts/audio/savant-audio-control.sh <command>

  Available Commands

  | Command | Description                            | Example                                    |
  |---------|----------------------------------------|--------------------------------------------|
  | status  | Show daemon status and recent activity | ./savant-audio-control.sh status           |
  | start   | Start the audio capture daemon         | ./savant-audio-control.sh start            |
  | stop    | Stop the audio capture daemon          | ./savant-audio-control.sh stop             |
  | restart | Restart the audio capture daemon       | ./savant-audio-control.sh restart          |
  | logs    | View live daemon logs                  | ./savant-audio-control.sh logs             |
  | list    | List all captured transcripts          | ./savant-audio-control.sh list             |
  | search  | Search transcripts for text            | ./savant-audio-control.sh search "meeting" |
  | test    | Test multiple instance protection      | ./savant-audio-control.sh test             |
  | help    | Show help information                  | ./savant-audio-control.sh help             |

  Key File Locations

  ~/Documents/savant-ai/
  ├── scripts/audio/
  │   ├── savant-audio-daemon.sh      # Main daemon script
  │   └── savant-audio-control.sh     # Control interface
  └── data/
      ├── audio-captures/             # Transcribed audio files
      └── daemon-logs/
          ├── savant-audio-daemon.log # Daemon activity log
          └── savant-audio-daemon.pid # Process ID file

