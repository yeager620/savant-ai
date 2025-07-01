# Cleanup Candidates - Files for Potential Removal

This document identifies files and code that may be extraneous and could potentially be removed to simplify the codebase.

## 🗂️ Files Identified for Review

### Potentially Redundant Scripts

#### Setup Scripts (multiple versions)
- `scripts/setup/auto-setup-system-audio.sh` - **KEEP** (main automated setup)
- `scripts/setup/setup-system-audio.sh` - **REVIEW** (may be duplicate functionality)
- `scripts/setup/fixed-audio-daemon.sh` - **REVIEW** (may be obsolete after daemon fixes)

**Recommendation**: Compare functionality and consolidate if duplicated.

#### Example Scripts
- `examples/unix_examples.sh` - **KEEP** (demonstrates UNIX philosophy)

### Documentation Files

#### Development Documentation
- `docs/development/UNIX_PHILOSOPHY_DEMO.md` - **KEEP** (important architectural reference)
- `docs/development/UNIX_REFACTOR_PLAN.md` - **REVIEW** (may be completed work)

**Recommendation**: Archive completed refactor plans or move to historical docs.

### Source Code Components

#### Removed UI Components (confirmed removed)
- ✅ `src/components/dashboard.rs` - **ALREADY REMOVED**
- ✅ `src/components/overlay.rs` - **ALREADY REMOVED** 
- ✅ `src/components/browser_overlay.rs` - **ALREADY REMOVED**
- ✅ `src/commands/ocr.rs` - **ALREADY REMOVED**

#### Unused Functions (compiler warnings indicate)
- `crates/savant-audio/src/macos.rs:110` - `check_macos_version()` function
- `crates/savant-audio/src/macos.rs:117` - `check_audio_permissions()` function
- `crates/savant-stt/src/whisper.rs:225` - Unused struct fields `timestamp` and `channels`

**Recommendation**: Remove unused functions or implement their usage.

### Build/Distribution Files

#### Target Directory
- `target/` - **KEEP** (Rust build artifacts, in .gitignore)

#### Distribution Directory
- `dist/` - **KEEP** (Frontend build output)

### Configuration Files

#### Multiple Cargo.toml Files
- Root `Cargo.toml` - **KEEP** (workspace root)
- `src-tauri/Cargo.toml` - **KEEP** (backend app)
- Individual crate `Cargo.toml` files - **KEEP** (required for workspace)

**Status**: All necessary for workspace architecture.

## 🧹 Cleanup Actions

### Immediate Actions (Safe)
1. **Remove compiler warning functions**:
   ```bash
   # Remove unused functions in savant-audio/src/macos.rs
   # Remove unused struct fields in savant-stt/src/whisper.rs
   ```

2. **Consolidate setup scripts**:
   ```bash
   # Compare and merge duplicate setup functionality
   # Keep one authoritative setup script
   ```

### Review Required
1. **Compare setup scripts functionality**
2. **Archive completed refactor documentation**
3. **Evaluate if any development documentation is outdated**

### NOT to Remove
- Core Rust workspace crates
- Essential build configurations
- Active documentation
- Current UI components
- Working scripts and utilities

## 📊 Cleanup Impact Assessment

### Low Risk Removals
- Unused functions (compiler warnings)
- Duplicate setup scripts (after verification)
- Completed planning documents

### Medium Risk Removals
- Development documentation that may still be referenced
- Alternative script versions that might have unique features

### High Risk Removals
- Any core Rust crates
- Active UI components
- Working configuration files

## 🎯 Recommended Cleanup Order

1. **Phase 1**: Remove unused functions causing compiler warnings
2. **Phase 2**: Compare and consolidate duplicate scripts
3. **Phase 3**: Archive completed documentation
4. **Phase 4**: Review and clean development artifacts

## ⚠️ Important Notes

- **DO NOT** remove any files without testing functionality first
- **ALWAYS** backup or commit changes before cleanup
- **VERIFY** that removed functions are truly unused
- **TEST** application after each cleanup phase

This document should be updated as cleanup progresses and new candidates are identified.