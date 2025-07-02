#!/bin/bash

# Database Cleanup Script
# Removes old temporary databases and provides cleanup options

set -euo pipefail

DATABASES_DIR="$(dirname "$0")/../data/databases"
TEMP_DIR="$DATABASES_DIR/tmp"
DEV_DIR="$DATABASES_DIR/dev"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    cat << EOF
Database Cleanup Script

Usage: $0 [OPTIONS]

Options:
    --dry-run       Show what would be deleted without actually deleting
    --temp-only     Only clean temporary databases (default)
    --dev-clean     Clean development databases (interactive)
    --all           Clean all non-essential databases (interactive)
    --older-than N  Clean files older than N days (default: 7)
    --help          Show this help message

Examples:
    $0                          # Clean temp databases older than 7 days
    $0 --dry-run                # Preview cleanup without deleting
    $0 --older-than 3           # Clean temp databases older than 3 days
    $0 --dev-clean              # Interactively clean dev databases
EOF
}

# Parse command line arguments
DRY_RUN=false
TEMP_ONLY=true
DEV_CLEAN=false
ALL_CLEAN=false
OLDER_THAN_DAYS=7

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --temp-only)
            TEMP_ONLY=true
            shift
            ;;
        --dev-clean)
            DEV_CLEAN=true
            TEMP_ONLY=false
            shift
            ;;
        --all)
            ALL_CLEAN=true
            TEMP_ONLY=false
            shift
            ;;
        --older-than)
            OLDER_THAN_DAYS="$2"
            shift 2
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Create directories if they don't exist
mkdir -p "$TEMP_DIR" "$DEV_DIR"

# Function to clean temporary databases
clean_temp_databases() {
    log_info "Cleaning temporary databases older than $OLDER_THAN_DAYS days..."
    
    if [ ! -d "$TEMP_DIR" ]; then
        log_warn "Temporary database directory does not exist: $TEMP_DIR"
        return 0
    fi
    
    local count=0
    while IFS= read -r -d '' file; do
        if [ "$DRY_RUN" = true ]; then
            log_info "Would delete: $file"
        else
            log_info "Deleting: $file"
            rm -f "$file"
        fi
        ((count++))
    done < <(find "$TEMP_DIR" -name "*.db" -type f -mtime +"$OLDER_THAN_DAYS" -print0 2>/dev/null || true)
    
    if [ $count -eq 0 ]; then
        log_info "No temporary databases found older than $OLDER_THAN_DAYS days"
    else
        log_info "Processed $count temporary database(s)"
    fi
}

# Function to clean development databases (interactive)
clean_dev_databases() {
    log_info "Development database cleanup (interactive)..."
    
    if [ ! -d "$DEV_DIR" ]; then
        log_warn "Development database directory does not exist: $DEV_DIR"
        return 0
    fi
    
    local dev_files
    dev_files=$(find "$DEV_DIR" -name "*.db" -type f 2>/dev/null || true)
    
    if [ -z "$dev_files" ]; then
        log_info "No development databases found"
        return 0
    fi
    
    echo "$dev_files" | while read -r file; do
        if [ -n "$file" ]; then
            local size
            size=$(du -h "$file" | cut -f1)
            local modified
            modified=$(stat -f "%Sm" -t "%Y-%m-%d %H:%M" "$file" 2>/dev/null || stat -c "%y" "$file" 2>/dev/null | cut -d' ' -f1,2 | cut -d'.' -f1)
            
            echo
            log_info "Found development database:"
            echo "  File: $(basename "$file")"
            echo "  Size: $size"
            echo "  Modified: $modified"
            
            if [ "$DRY_RUN" = true ]; then
                log_info "Would prompt for deletion (dry run)"
            else
                read -p "Delete this database? [y/N]: " -r
                if [[ $REPLY =~ ^[Yy]$ ]]; then
                    rm -f "$file"
                    log_info "Deleted: $file"
                else
                    log_info "Kept: $file"
                fi
            fi
        fi
    done
}

# Function to show database statistics
show_stats() {
    log_info "Database Statistics:"
    
    local temp_count dev_count test_count
    temp_count=$(find "$TEMP_DIR" -name "*.db" -type f 2>/dev/null | wc -l | tr -d ' ')
    dev_count=$(find "$DEV_DIR" -name "*.db" -type f 2>/dev/null | wc -l | tr -d ' ')
    test_count=$(find "$DATABASES_DIR/test" -name "*.db" -type f 2>/dev/null | wc -l | tr -d ' ')
    
    echo "  Temporary databases: $temp_count"
    echo "  Development databases: $dev_count"
    echo "  Test databases: $test_count"
    
    if [ -d "$TEMP_DIR" ] && [ $temp_count -gt 0 ]; then
        local temp_size
        temp_size=$(du -sh "$TEMP_DIR" 2>/dev/null | cut -f1 || echo "unknown")
        echo "  Temporary database size: $temp_size"
    fi
}

# Main execution
echo "Savant AI Database Cleanup"
echo "=========================="

show_stats
echo

if [ "$DRY_RUN" = true ]; then
    log_warn "DRY RUN MODE - No files will be deleted"
    echo
fi

if [ "$TEMP_ONLY" = true ]; then
    clean_temp_databases
elif [ "$DEV_CLEAN" = true ]; then
    clean_temp_databases
    clean_dev_databases
elif [ "$ALL_CLEAN" = true ]; then
    if [ "$DRY_RUN" = false ]; then
        log_warn "This will clean ALL non-essential databases!"
        read -p "Are you sure? [y/N]: " -r
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Cleanup cancelled"
            exit 0
        fi
    fi
    clean_temp_databases
    clean_dev_databases
fi

echo
log_info "Database cleanup completed"