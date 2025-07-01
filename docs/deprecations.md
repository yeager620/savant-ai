# Savant AI - Deprecations & Cleanup Guide

This document tracks deprecated features, extraneous code, and planned removals to maintain a clean, efficient codebase aligned with the project's UNIX philosophy.

## 📋 **Table of Contents**
- [Audio Transcription System](#audio-transcription-system)
- [Frontend Components](#frontend-components)  
- [Backend Commands](#backend-commands)
- [Database & Storage](#database--storage)
- [Build & Development](#build--development)
- [Cleanup Actions](#cleanup-actions)

---

## 🎙️ **Audio Transcription System**

### **✅ Implemented & Working**
- `crates/savant-transcribe/` - CLI transcription tool with JSON output
- `crates/savant-stt/` - Speech-to-text with Whisper integration  
- `crates/savant-audio/` - Audio capture from microphone and system
- Post-processing logic for silence handling ("you" → "[no signal]")
- Session metadata tracking with speaker identification

### **🗑️ Deprecated / To Remove**
- **Markdown-only output**: The original markdown-first approach should be phased out in favor of JSON-first with optional markdown export
- **Hardcoded 5-minute intervals**: The current audio daemon writes new files every 5 minutes - this should be replaced with continuous streaming to database
- **Manual file organization**: Individual .md files per session should be replaced with database storage

### **📁 Files Requiring Cleanup**
```bash
# Remove or consolidate these once database is fully implemented:
src-tauri/src/commands/config.rs     # May have audio-specific configs that should move to transcribe CLI
styles.css                           # Check for unused audio-related CSS rules
```

---

## 🖥️ **Frontend Components**

### **✅ Active Components**
```
src/
├── main.rs                     # ✅ Frontend entry point
├── taskbar_app.rs              # ✅ Main UI component
├── components/
│   └── minimal_chat.rs         # ✅ Chat interface
└── utils/
    ├── llm.rs                  # ✅ Frontend LLM utilities
    └── shared_types.rs         # ✅ Shared type definitions
```

### **🗑️ Deprecated Components**
```
src/components/
├── invisible_overlay.rs        # ⚠️  PARTIALLY DEPRECATED
│                              # - Heavy compilation errors
│                              # - Complex overlay system may be overkill
│                              # - Consider simplifying or removing
│
├── simple_invisible_overlay.rs # ❌ REMOVE CANDIDATE
│                              # - Appears to be test/unused code
│                              # - Name suggests it's a simplified version
│
└── dashboard.rs               # ❌ REMOVED (mentioned in CLAUDE.md)
    browser_overlay.rs         # ❌ REMOVED (mentioned in CLAUDE.md)
    overlay.rs                 # ❌ REMOVED (mentioned in CLAUDE.md)
```

### **🔧 Frontend Cleanup Actions**
1. **Resolve invisible_overlay.rs compilation issues or remove entirely**
   - Multiple compilation errors with wasm-bindgen types
   - Complex state management that may not align with minimalistic UI goals
   - Consider replacing with simpler stealth overlay if needed

2. **Remove simple_invisible_overlay.rs if unused**
   - Verify if this file is referenced anywhere
   - Remove if it's orphaned code

3. **Audit styles.css for unused rules**
   - Remove CSS for deleted overlay components
   - Consolidate chat-related styles

---

## ⚙️ **Backend Commands**

### **✅ Active Backend Commands**
```
src-tauri/src/commands/
├── llm.rs                      # ✅ Multi-provider AI integration
├── chat_history.rs             # ✅ Persistent conversation storage  
├── browser.rs                  # ✅ Browser monitoring via Accessibility APIs
├── system.rs                   # ✅ Stealth window management
├── hotkey.rs                   # ✅ Global keyboard shortcuts
└── config.rs                   # ✅ Configuration management
```

### **⚠️ Potentially Deprecated**
```
src-tauri/src/commands/
└── ocr.rs                      # ❌ REMOVED (mentioned in CLAUDE.md)
                               # - OCR/screenshot functionality removed
                               # - Dependencies on image/screenshots crates removed
```

### **🔧 Backend Cleanup Actions**
1. **Verify config.rs doesn't have orphaned audio settings**
   - Audio configuration should move to transcribe CLI
   - Keep only UI/stealth/browser settings in main config

2. **Audit dependencies in src-tauri/Cargo.toml**
   - Remove any image processing dependencies if OCR is fully removed
   - Remove screenshots crate dependencies mentioned in CLAUDE.md

---

## 🗄️ **Database & Storage**

### **✅ New Database Architecture**
```
crates/savant-db/               # 🔄 IN PROGRESS
├── src/lib.rs                  # ✅ Database schema & query interface
├── src/main.rs                 # ✅ CLI tool for database operations
└── migrations/001_initial.sql  # ✅ Database schema
```

### **🗑️ Legacy Storage (To Deprecate)**
- **Individual .md files**: Currently created every 5 minutes by audio daemon
- **Manual file organization**: Users manually organizing transcript files
- **Markdown-first approach**: Primary output being markdown instead of structured data

### **📁 Files Requiring Migration**
```bash
# These patterns should be migrated to database:
~/.config/savant-ai/chat_history.json    # Chat history should integrate with transcription DB
~/Documents/transcripts/*.md             # Existing markdown transcripts need migration tool
```

### **🔧 Database Implementation Status**
- ✅ **Schema Design**: Conversations + Segments tables with FTS
- ✅ **CLI Interface**: Full CRUD operations planned
- ❌ **SQLite Connection**: Currently has connection issues (needs debugging)
- 🔄 **Migration Tools**: Need scripts to import existing .md files

### **📋 Database Cleanup Actions**
1. **Resolve SQLite connection issues in savant-db**
   - Debug file permissions and path resolution
   - Test database creation and table setup

2. **Create migration scripts**
   - Tool to parse existing .md transcript files
   - Convert to JSON and import to database
   - Preserve timestamps and metadata where possible

3. **Phase out markdown-first approach**
   - Make JSON the primary output format
   - Keep markdown as optional export format
   - Update documentation to reflect this change

---

## 🔨 **Build & Development**

### **✅ Active Build Configuration**
```
Cargo.toml                      # ✅ Workspace root with all crates
Trunk.toml                      # ✅ Frontend build configuration
index.html                      # ✅ Frontend entry point  
tauri.conf.json                 # ✅ Desktop app configuration
```

### **⚠️ Potentially Extraneous**
```
.gemini/                        # ❓ UNKNOWN - Added recently, purpose unclear
                               # - May be AI-related tooling
                               # - Should be documented or removed
```

### **🔧 Build Cleanup Actions**
1. **Investigate .gemini/ directory**
   - Document its purpose or add to .gitignore
   - Ensure it's not accidentally committed configuration

2. **Audit workspace dependencies**
   - Remove unused dependencies from workspace Cargo.toml
   - Consolidate version numbers where possible

---

## 📊 **Cleanup Priority Matrix**

### **🔴 High Priority (Blocking Development)**
1. **Fix savant-db SQLite connection** - Blocks database integration
2. **Resolve invisible_overlay.rs compilation** - Blocks frontend builds
3. **Remove simple_invisible_overlay.rs** - Code cleanliness

### **🟡 Medium Priority (Technical Debt)**
1. **Create .md to database migration tool** - User data preservation
2. **Audit and consolidate CSS** - Performance and maintainability  
3. **Remove orphaned audio configs** - Configuration cleanliness

### **🟢 Low Priority (Nice to Have)**
1. **Document or remove .gemini/** - Project organization
2. **Consolidate dependency versions** - Build optimization
3. **Update CLAUDE.md with latest architecture** - Documentation accuracy

---

## 🚀 **Migration Plan**

### **Phase 1: Immediate Fixes (Week 1)**
```bash
# Fix blocking issues
1. Debug savant-db SQLite connection
2. Remove or fix invisible_overlay.rs compilation errors  
3. Remove simple_invisible_overlay.rs if unused
4. Document .gemini/ directory purpose
```

### **Phase 2: Database Integration (Week 2-3)**
```bash
# Complete database transition
1. Implement working database storage
2. Create markdown → JSON → database migration script
3. Update transcribe CLI to optionally pipe to database
4. Test full pipeline: audio → transcribe → store
```

### **Phase 3: Legacy Cleanup (Week 4)**
```bash
# Remove deprecated systems
1. Phase out individual .md file creation
2. Make JSON primary output format (markdown optional)
3. Remove unused CSS and frontend code
4. Update all documentation
```

---

## 📝 **Specific Cleanup Commands**

### **Safe to Remove (After Verification)**
```bash
# Remove these files if they're confirmed unused:
rm src/components/simple_invisible_overlay.rs

# Audit these for unused code:
# - src/components/invisible_overlay.rs (fix or remove)
# - styles.css (remove unused overlay rules)
# - .gemini/ (document or gitignore)
```

### **Requires Migration First**
```bash
# Don't remove until database migration is complete:
# - Individual .md transcript files
# - markdown::format_transcription_markdown function
# - CLI --format markdown option
```

### **Dependencies to Audit**
```toml
# Check if these are still needed in src-tauri/Cargo.toml:
# - Any image processing crates (if OCR removed)
# - screenshot-related dependencies
# - Unused UI framework dependencies
```

---

## ✅ **Implementation Verification Checklist**

Before removing any deprecated features, verify:

- [ ] **Database Integration Works**: `savant-db` can store and query transcriptions
- [ ] **Migration Tool Exists**: Can convert existing .md files to database
- [ ] **Pipeline Functions**: `savant-transcribe | savant-db store` works end-to-end
- [ ] **Frontend Compiles**: All compilation errors resolved
- [ ] **Tests Pass**: `cargo test --workspace` succeeds
- [ ] **Documentation Updated**: README reflects current architecture

---

**Last Updated**: 2025-07-01  
**Status**: Database integration in progress, frontend cleanup needed  
**Next Review**: After database connection issues resolved