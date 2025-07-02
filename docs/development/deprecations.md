# Deprecations and Migration Guide

This document tracks deprecated features and provides migration paths following the project's UNIX philosophy and design principles.

## Current Deprecations

### Frontend Components

#### ❌ Confirmed Deprecated
- **`src/components/simple_invisible_overlay.rs`**
  - Status: Unused, not in mod.rs
  - Reason: Superseded by minimalistic UI approach
  - Action: Remove safely

- **CSS Classes in `styles.css`**
  - `.dashboard` (lines 60, 69, 322) - Component removed
  - `.overlay-container`, `.overlay-toggle` (lines 190-311) - Overlay components removed
  - Action: Clean up unused styles

#### ⚠️ Compilation Issues
- **`src/components/invisible_overlay.rs`**
  - Status: Has wasm-bindgen compilation errors
  - Reason: Complex overlay system conflicts with minimalistic design
  - Action: Fix compilation or remove entirely

### Backend Systems

#### ✅ Already Removed
- **OCR/Screenshot System**
  - `src-tauri/src/commands/ocr.rs` - Removed
  - Image processing dependencies - Audit remaining
  - Reason: Focused on audio-first approach

#### 🔄 In Transition
- **Individual Markdown Transcript Files**
  - Legacy: 5-minute interval .md files
  - New: SQLite database with MCP server
  - Migration: Complete database transition before removing .md generation
  - Timeline: Phase out after v1.0

### UNIX Tools Migration

#### ✅ Completed Refactoring
- **CLI Tools Implementation**
  - `savant-llm` - LLM inference CLI ✅
  - `savant-transcribe` - Audio transcription CLI ✅  
  - `savant-db` - Database management CLI ✅
  - `savant-mcp` - Model Context Protocol server ✅

#### 📄 Documentation Updates Needed
- **`docs/development/UNIX_REFACTOR_PLAN.md`**
  - Status: Refactoring complete
  - Action: Archive or update to reflect current state

### Code Quality Issues

#### Unused Functions (Compiler Warnings)
```rust
// crates/savant-audio/src/macos.rs
fn check_macos_version() // Line 110 - unused
fn check_audio_permissions() // Line 117 - unused

// crates/savant-stt/src/whisper.rs  
struct WhisperConfig { /* unused fields */ } // Line 225
```

#### Scripts Consolidation Needed
- **Setup Scripts**
  - `scripts/setup/setup-system-audio.sh` vs `scripts/setup/auto-setup-system-audio.sh`
  - `scripts/setup/fixed-audio-daemon.sh` - May be obsolete
  - Action: Compare functionality and consolidate

## Migration Paths

### For Users

#### Audio Transcription
```bash
# Old: Individual markdown files
# New: Database-first with CLI tools
savant-transcribe --speaker "user" --duration 10 | savant-db store --title "Meeting"
```

#### Data Access
```bash
# Old: File-based search
# New: Database queries + MCP server
savant-db query --speaker "john" --text "meeting"
echo '{"method":"tools/call","params":{"name":"query_conversations"}}' | savant-mcp
```

### For Developers

#### UI Components
```rust
// Old: Complex overlay system
// New: Minimalistic taskbar approach
// Use: src/taskbar_app.rs as primary UI
```

#### Database Integration
```rust
// Old: Direct file I/O
// New: MCP server + structured queries
use savant_db::{MCPServer, TranscriptDatabase};
```

## Cleanup Timeline

### Immediate (Blocking Development)
1. Fix or remove `invisible_overlay.rs` compilation errors
2. Remove `simple_invisible_overlay.rs` 
3. Clean up unused CSS classes

### Short-term (Technical Debt)
1. Remove unused functions in audio/stt crates
2. Consolidate duplicate setup scripts
3. Document or remove `.gemini/` directory
4. Audit image processing dependencies

### Long-term (Post v1.0)
1. Phase out markdown-first transcript approach
2. Remove individual .md file generation  
3. Archive completed refactoring documentation
4. Migrate remaining hardcoded configurations to TOML

## Design Principle Alignment

### UNIX Philosophy Compliance
- ✅ Single-purpose CLI tools implemented
- ✅ JSON I/O standardization complete
- ✅ Composable workflow patterns established
- 🔄 Legacy components being phased out

### Security & Privacy
- ✅ Query validation implemented
- ✅ Local processing maintained
- ✅ Rate limiting added to MCP server
- 🔄 Encryption at rest planned

### Performance & Scalability  
- ✅ Async operations throughout
- ✅ Connection pooling implemented
- ✅ Time-series database optimizations
- 🔄 Vector search integration planned

## Breaking Changes

None currently planned. All deprecations follow graceful migration patterns with backwards compatibility during transition periods.

## Support

For migration assistance or questions about deprecated features:
1. Check relevant documentation in `docs/`
2. Run test scripts: `./test-mcp-natural-queries.sh`, `./test-database-sql.sh`
3. Review `CLAUDE.md` for current architecture