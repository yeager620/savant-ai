#!/usr/bin/env bash

# Codebase Size Calculator - Bash Version
# Recursively calculates the total size of source code files in a codebase by file type.

# Define source code file extensions and their languages
declare -A EXTENSIONS=(
    # Popular languages
    [".py"]="Python"
    [".js"]="JavaScript"
    [".ts"]="TypeScript"
    [".jsx"]="React JSX"
    [".tsx"]="React TSX"
    [".java"]="Java"
    [".c"]="C"
    [".cpp"]="C++"
    [".cc"]="C++"
    [".cxx"]="C++"
    [".h"]="C/C++ Header"
    [".hpp"]="C++ Header"
    [".cs"]="C#"
    [".rs"]="Rust"
    [".go"]="Go"
    [".php"]="PHP"
    [".rb"]="Ruby"
    [".swift"]="Swift"
    [".kt"]="Kotlin"
    [".scala"]="Scala"
    [".r"]="R"
    [".m"]="Objective-C"
    [".mm"]="Objective-C++"
    
    # Web technologies
    [".html"]="HTML"
    [".htm"]="HTML"
    [".css"]="CSS"
    [".scss"]="SCSS"
    [".sass"]="Sass"
    [".less"]="Less"
    [".vue"]="Vue"
    [".svelte"]="Svelte"
    
    # Shell and scripting
    [".sh"]="Shell"
    [".bash"]="Bash"
    [".zsh"]="Zsh"
    [".fish"]="Fish"
    [".ps1"]="PowerShell"
    [".bat"]="Batch"
    [".cmd"]="Command"
    
    # Other languages
    [".lua"]="Lua"
    [".pl"]="Perl"
    [".pm"]="Perl Module"
    [".dart"]="Dart"
    [".elm"]="Elm"
    [".ex"]="Elixir"
    [".exs"]="Elixir Script"
    [".erl"]="Erlang"
    [".hrl"]="Erlang Header"
    [".clj"]="Clojure"
    [".cljs"]="ClojureScript"
    [".hs"]="Haskell"
    [".lhs"]="Literate Haskell"
    [".ml"]="OCaml"
    [".mli"]="OCaml Interface"
    [".fs"]="F#"
    [".fsx"]="F# Script"
    [".jl"]="Julia"
    [".nim"]="Nim"
    [".cr"]="Crystal"
    [".zig"]="Zig"
    
    # Configuration and data files
    [".json"]="JSON"
    [".yaml"]="YAML"
    [".yml"]="YAML"
    [".toml"]="TOML"
    [".xml"]="XML"
    [".sql"]="SQL"
    [".dockerfile"]="Dockerfile"
    [".makefile"]="Makefile"
    [".cmake"]="CMake"
    [".gradle"]="Gradle"
    [".groovy"]="Groovy"
)

# Directories to ignore
IGNORE_DIRS=".git .svn .hg .bzr node_modules __pycache__ .pytest_cache target build dist out bin obj .vscode .idea .eclipse vendor deps third_party"

# Global counters
declare -A LANG_FILES
declare -A LANG_SIZES
TOTAL_FILES=0
TOTAL_SIZE=0

# Function to convert bytes to human readable format
format_size() {
    local bytes="$1"
    local precision="${2:-1}"
    
    if [[ $bytes -lt 1024 ]]; then
        printf "%d B" "$bytes"
    elif [[ $bytes -lt $((1024 * 1024)) ]]; then
        printf "%.${precision}f KB" "$(awk "BEGIN {printf \"%.${precision}f\", $bytes/1024}")"
    elif [[ $bytes -lt $((1024 * 1024 * 1024)) ]]; then
        printf "%.${precision}f MB" "$(awk "BEGIN {printf \"%.${precision}f\", $bytes/(1024*1024)}")"
    elif [[ $bytes -lt $((1024 * 1024 * 1024 * 1024)) ]]; then
        printf "%.${precision}f GB" "$(awk "BEGIN {printf \"%.${precision}f\", $bytes/(1024*1024*1024)}")"
    else
        printf "%.${precision}f TB" "$(awk "BEGIN {printf \"%.${precision}f\", $bytes/(1024*1024*1024*1024)}")"
    fi
}

# Function to find git project root
find_git_root() {
    local dir="$1"
    [[ -z "$dir" ]] && dir="$PWD"
    
    # Convert to absolute path
    dir=$(realpath "$dir" 2>/dev/null || echo "$dir")
    
    # Search upward for .git directory
    while [[ "$dir" != "/" ]]; do
        if [[ -d "$dir/.git" ]]; then
            echo "$dir"
            return 0
        fi
        dir=$(dirname "$dir")
    done
    
    # No git root found
    return 1
}

# Function to check if a directory should be ignored
should_ignore_dir() {
    local dir="$1"
    for ignore_dir in $IGNORE_DIRS; do
        if [[ "$dir" == *"$ignore_dir"* ]]; then
            return 0
        fi
    done
    return 1
}

# Function to check if a file should be ignored
should_ignore_file() {
    local file="$1"
    local basename=$(basename "$file")
    
    # Ignore hidden files (except some common ones)
    if [[ "$basename" == .* ]] && [[ "$basename" != ".gitignore" ]] && [[ "$basename" != ".dockerignore" ]] && [[ "$basename" != ".editorconfig" ]]; then
        return 0
    fi
    
    # Ignore backup and temporary files
    if [[ "$basename" == *.bak ]] || [[ "$basename" == *.tmp ]] || [[ "$basename" == *.temp ]] || [[ "$basename" == *.swp ]] || [[ "$basename" == *.swo ]] || [[ "$basename" == *~ ]]; then
        return 0
    fi
    
    # Ignore lock files
    if [[ "$basename" == "package-lock.json" ]] || [[ "$basename" == "yarn.lock" ]] || [[ "$basename" == "cargo.lock" ]] || [[ "$basename" == "pipfile.lock" ]] || [[ "$basename" == "poetry.lock" ]]; then
        return 0
    fi
    
    return 1
}

# Function to get file extension
get_extension() {
    local file="$1"
    local basename=$(basename "$file")
    
    # Handle special files without extensions
    case "$basename" in
        [Mm]akefile|[Gg]emfile|[Rr]akefile)
            echo ".makefile"
            return
            ;;
        [Dd]ockerfile)
            echo ".dockerfile"
            return
            ;;
    esac
    
    # Get the extension (everything after the last dot)
    if [[ "$basename" == *.* ]]; then
        echo "${basename##*.}" | tr '[:upper:]' '[:lower:]' | sed 's/^/./'
    else
        echo ""
    fi
}

# Function to get file size
get_file_size() {
    local file="$1"
    if [[ -r "$file" ]] && [[ -f "$file" ]]; then
        # Use stat to get file size (works on both macOS and Linux)
        if stat -c%s "$file" 2>/dev/null; then
            return
        elif stat -f%z "$file" 2>/dev/null; then
            return
        else
            # Fallback using wc (less accurate for binary files)
            wc -c < "$file" 2>/dev/null || echo 0
        fi
    else
        echo 0
    fi
}

# Function to process a single file
process_file() {
    local file="$1"
    local specific_extensions="$2"
    
    # Skip if file should be ignored
    if should_ignore_file "$file"; then
        return
    fi
    
    local ext=$(get_extension "$file")
    
    # Skip if extension is empty
    if [[ -z "$ext" ]]; then
        return
    fi
    
    # If specific extensions are provided, check if this file matches
    if [[ -n "$specific_extensions" ]]; then
        local found=false
        for spec_ext in $specific_extensions; do
            if [[ "$ext" == "$spec_ext" ]]; then
                found=true
                break
            fi
        done
        if [[ "$found" == false ]]; then
            return
        fi
    fi
    
    # Check if extension is in our known list
    if [[ -n "${EXTENSIONS[$ext]}" ]]; then
        local language="${EXTENSIONS[$ext]}"
        local size=$(get_file_size "$file")
        
        if [[ $size -gt 0 ]]; then
            LANG_FILES["$language"]=$((${LANG_FILES["$language"]:-0} + 1))
            LANG_SIZES["$language"]=$((${LANG_SIZES["$language"]:-0} + size))
            TOTAL_FILES=$((TOTAL_FILES + 1))
            TOTAL_SIZE=$((TOTAL_SIZE + size))
        fi
    fi
}

# Function to recursively process directory
process_directory() {
    local dir="$1"
    local specific_extensions="$2"
    
    # Check if directory should be ignored
    if should_ignore_dir "$dir"; then
        return
    fi
    
    # Process all files in current directory
    while IFS= read -r -d '' file; do
        if [[ -f "$file" ]]; then
            process_file "$file" "$specific_extensions"
        fi
    done < <(find "$dir" -maxdepth 1 -type f -print0 2>/dev/null)
    
    # Recursively process subdirectories
    while IFS= read -r -d '' subdir; do
        if [[ -d "$subdir" ]]; then
            process_directory "$subdir" "$specific_extensions"
        fi
    done < <(find "$dir" -maxdepth 1 -type d ! -path "$dir" -print0 2>/dev/null)
}

# Function to print results
print_results() {
    if [[ $TOTAL_FILES -eq 0 ]]; then
        echo "No source code files found."
        return
    fi
    
    echo
    echo "Codebase Size Summary"
    echo "============================================================"
    printf "%-20s %-8s %-12s %-8s\n" "Language" "Files" "Size" "%"
    echo "------------------------------------------------------------"
    
    # Sort languages by size (descending)
    # Create a temporary file with language:size pairs
    local temp_file=$(mktemp)
    for lang in "${!LANG_SIZES[@]}"; do
        echo "${LANG_SIZES[$lang]}:$lang" >> "$temp_file"
    done
    
    # Sort by size (descending) and print
    while IFS=':' read -r size lang; do
        local files=${LANG_FILES["$lang"]}
        local formatted_size=$(format_size "$size" 1)
        local percentage=$(awk "BEGIN {printf \"%.1f\", ($size / $TOTAL_SIZE) * 100}")
        printf "%-20s %-8d %-12s %-7s%%\n" "$lang" "$files" "$formatted_size" "$percentage"
    done < <(sort -rn "$temp_file")
    
    rm -f "$temp_file"
    
    echo "------------------------------------------------------------"
    local total_formatted=$(format_size "$TOTAL_SIZE" 1)
    printf "%-20s %-8d %-12s %-8s\n" "TOTAL" "$TOTAL_FILES" "$total_formatted" "100.0%"
    echo
    
    # Additional summary
    echo "Summary:"
    echo "  Total source files: $TOTAL_FILES"
    echo "  Total codebase size: $(format_size "$TOTAL_SIZE" 2)"
    echo "  Average file size: $(format_size $((TOTAL_SIZE / TOTAL_FILES)) 2)"
    echo
}

# Function to list supported extensions
list_extensions() {
    echo "Supported file extensions:"
    for ext in $(printf '%s\n' "${!EXTENSIONS[@]}" | sort); do
        printf "  %-12s %s\n" "$ext" "${EXTENSIONS[$ext]}"
    done
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [DIRECTORY] [OPTIONS]"
    echo
    echo "Calculate the total size of source code files in a codebase"
    echo
    echo "Arguments:"
    echo "  DIRECTORY           Directory to search (default: auto-detect git root or current directory)"
    echo
    echo "Options:"
    echo "  -e, --extensions    Specific file extensions to include (e.g., .py .js .rs)"
    echo "  -l, --list          List all supported file extensions"
    echo "  -r, --git-root      Force search for git repository root"
    echo "  --no-git-root       Disable automatic git root detection"
    echo "  -h, --help          Show this help message"
    echo
    echo "Examples:"
    echo "  $0                              # Calculate size of all source code (auto-detects git root)"
    echo "  $0 /path/to/project             # Calculate size in specific directory"
    echo "  $0 -e .rs .py                   # Calculate size of only Rust and Python files"
    echo "  $0 --no-git-root                # Use current directory, don't search for git root"
    echo "  $0 -r                           # Force git root detection from current directory"
}

# Main function
main() {
    local directory=""
    local specific_extensions=""
    local list_mode=false
    local force_git_root=false
    local disable_git_root=false
    local directory_specified=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -e|--extensions)
                shift
                while [[ $# -gt 0 ]] && [[ $1 != -* ]]; do
                    # Add dot if not present
                    if [[ $1 == .* ]]; then
                        specific_extensions="$specific_extensions $1"
                    else
                        specific_extensions="$specific_extensions .$1"
                    fi
                    shift
                done
                continue
                ;;
            -l|--list)
                list_mode=true
                shift
                ;;
            -r|--git-root)
                force_git_root=true
                shift
                ;;
            --no-git-root)
                disable_git_root=true
                shift
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            -*)
                echo "Error: Unknown option $1"
                show_usage
                exit 1
                ;;
            *)
                if [[ -z "$directory" ]]; then
                    directory="$1"
                    directory_specified=true
                else
                    echo "Error: Multiple directories specified"
                    show_usage
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    if [[ "$list_mode" == true ]]; then
        list_extensions
        exit 0
    fi
    
    # Determine the directory to analyze
    if [[ -n "$directory" ]]; then
        # Directory was explicitly specified
        if [[ ! -d "$directory" ]]; then
            echo "Error: Directory '$directory' does not exist."
            exit 1
        fi
    else
        # No directory specified, try to auto-detect git root
        if [[ "$disable_git_root" == false ]]; then
            local git_root
            git_root=$(find_git_root)
            if [[ $? -eq 0 ]]; then
                directory="$git_root"
                echo "Auto-detected git repository root: $directory"
            else
                directory="."
                echo "No git repository found, using current directory"
            fi
        else
            directory="."
        fi
    fi
    
    # Handle force git root option
    if [[ "$force_git_root" == true ]]; then
        local git_root
        if [[ "$directory_specified" == true ]]; then
            git_root=$(find_git_root "$directory")
        else
            git_root=$(find_git_root)
        fi
        
        if [[ $? -eq 0 ]]; then
            directory="$git_root"
            echo "Using git repository root: $directory"
        else
            echo "Error: No git repository found"
            if [[ "$directory_specified" == true ]]; then
                echo "Searched from: $directory"
            else
                echo "Searched from: $PWD"
            fi
            exit 1
        fi
    fi
    
    echo "Calculating codebase size in: $(realpath "$directory")"
    if [[ -n "$specific_extensions" ]]; then
        echo "Including extensions:$specific_extensions"
    fi
    echo "Analyzing..."
    
    # Process the directory
    process_directory "$directory" "$specific_extensions"
    
    # Print results
    print_results
}

# Check if script is being executed (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
