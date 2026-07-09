# media-sorter

![](.img/media-sorter.png)

A high-performance Rust CLI tool to automatically sort and rename your images and videos by their creation date.

## Overview

**media-sorter** scans your media library and intelligently organizes it by automatically:
- **Renaming** files with consistent naming: `IMG_YYYYMMDD_HHMMSS.ext` or `VID_YYYYMMDD_HHMMSS.ext`
- **Moving** files to organized year-based directories (e.g., `2024/`, `2023/`)
- **Detecting duplicates** using xxh3 hashing to prevent duplicate copies
- **Extracting metadata** from EXIF data and file timestamps

Perfect for organizing large photo and video collections from cameras, phones, and other sources.

## Installation

### Quick Install (Recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/KVN1701/media-sorter/main/install.sh | sh
```

### Manual Installation

Ensure you have [Rust](https://www.rust-lang.org/tools/install) and [exiftool](https://exiftool.org/) installed, then:

```sh
git clone https://github.com/KVN1701/media-sorter.git
cd media-sorter
cargo build --release
sudo cp target/release/media-sorter /usr/local/bin/
```

### Prerequisites

- **exiftool**: Required for reading media metadata
  - macOS: `brew install exiftool`
  - Linux: `sudo apt-get install libimage-exiftool-perl` or `sudo yum install perl-Image-ExifTool`
  - Windows: Download from [exiftool.org](https://exiftool.org/)

### Verify Installation

```sh
media-sorter --version
```

## Usage

### Basic Usage

Sort all media in a folder by creation date (default behavior):

```sh
media-sorter /path/to/media/folder -d /path/to/destination/folder
```

This will:
1. Scan all supported media files and collect their hashes
2. Extract creation dates from metadata
3. Rename files to `IMG_YYYYMMDD_HHMMSS.ext` or `VID_YYYYMMDD_HHMMSS.ext`
4. Organize into year subdirectories (2024/, 2023/, etc.)
5. Skip any existing files (won't overwrite)

### Available Options

```
USAGE:
    media-sorter [OPTIONS] <SOURCE>

ARGUMENTS:
    <SOURCE>    Define the source folder

OPTIONS:
    -d, --dest <FOLDER>
        Define the destination folder. Defaults to the source folder.
    
    -r, --recursive
        Recursively gathers files in the source and destination folders.
    
    -D, --remove-duplicates
        Removes duplicates of files by comparing hashes. Causes a great increase in runtime.
    
    -l, --list
        List the files in the source folder. Does not move or rename files.
    
    -o, --output <OUTPUT>
        Output a list of files to a file (requires --list)
    
    -n, --rename
        Renames the files during the operation.
    
    --skip-dirs <SKIP_DIRS>
        Skip the specified directories (comma-separated). 
        Example: --skip-dirs "Backups,Archive,Temp"
    
    -s, --sort
        Create subdirectories for every year (e.g., 2000/, 2001/, ...)
    
    -h, --help
        Print help information
    
    -V, --version
        Print version information
```

### Common Examples

**Sort files into a different directory with duplicate detection:**
```sh
media-sorter /path/to/unsorted --dest /path/to/destination --remove-duplicates
```

**Rename and organize files with year subdirectories:**
```sh
media-sorter /path/to/media --rename --sort --dest /path/to/destination
```

**List all media files without making changes:**
```sh
media-sorter /path/to/media --list
```

**Save file list to a file:**
```sh
media-sorter /path/to/media --list --output media_files.txt
```

**Recursively process nested folders:**
```sh
media-sorter /path/to/media --recursive --dest /path/to/destination
```

**Skip specific folders:**
```sh
media-sorter /path/to/media --skip-dirs "Archive,Backups,Trash" --dest /path/to/destination
```

**Organize files without year subdirectories:**
```sh
media-sorter /path/to/media --rename --dest /path/to/destination
```

## Supported Formats

### Images
- JPG / JPEG
- PNG
- GIF
- BMP

### Videos
- MP4
- AVI
- MKV
- MOV
- FLV

## How It Works

### Default Mode (Fast, Single-Pass Processing)

1. **Scan**: Identifies all media files in source folder
2. **Extract Metadata**: Reads creation dates from EXIF data or file timestamps
3. **Rename**: Generates standardized filenames with timestamps
4. **Organize**: Creates year-based subdirectories and moves files
5. **Deduplicate**: Removes duplicate files found in the source folder

### Duplicate Detection Mode (with `--remove-duplicates`)

1. **Gather Source Hashes**: Computes XXH3 hashes for all media files in source
2. **Check Destination**: Computes hashes of files already in destination
3. **Compare**: Identifies which files are new or different
4. **Process**: Renames and moves only new/different files
5. **Speed Impact**: Significantly slower but thorough for large collections

### List Mode

- Scans and displays all supported media files
- Optionally saves list to a text file
- No files are modified

### Recursive Processing (with `--recursive`)

- Scans all subdirectories within the source folder
- Processes nested media files recursively
- Organizes all files into a unified destination structure

## Naming Convention

Files are renamed according to this pattern:

```
[TYPE]_YYYYMMDD_HHMMSS[_COUNTER].[ext]
```

- **TYPE**: `IMG` for images, `VID` for videos
- **YYYYMMDD**: Creation date (Year-Month-Day)
- **HHMMSS**: Creation time (Hour-Minute-Second)
- **_COUNTER**: Optional suffix for collisions (e.g., `_0001`, `_0002`)
- **ext**: Original file extension (lowercase)

### Examples
- `IMG_20240515_143022.jpg` - Photo taken May 15, 2024 at 2:30:22 PM
- `VID_20231225_081500_0001.mp4` - Video with duplicate timestamp

## Technical Details

### Dependencies

- **chrono**: Date/time parsing and manipulation
- **clap**: Command-line argument parsing
- **rayon**: Data parallelism for fast processing
- **walkdir**: Recursive directory traversal
- **xxhash-rust**: Fast XXH3 hashing for duplicates
- **indicatif**: Progress bars
- **colored**: Terminal colors
- **anyhow**: Error handling
- **exiftool** (external): EXIF metadata extraction

### Performance

- Parallel processing of large file sets
- Efficient XXH3 hashing (8192-byte buffer chunks)
- Progress feedback to avoid blocking perception
- Optional recursive and duplicate detection modes for advanced use cases

### Code Architecture

The codebase has been refactored into modular components:

- **`file.rs`**: Core `MediaFile` struct and file operations
- **`sorter.rs`**: File discovery, hashing, and movement logic
- **`parser.rs`**: CLI argument parsing with flexible modes
- **`main.rs`**: Application orchestration and mode handling
- **`banner.rs`**: Welcome message display

## Contributing

Contributions are welcome! Feel free to:
- Report issues
- Suggest features
- Submit pull requests

## License

This project is open source. Check the repository for license details.

## Troubleshooting

### "exiftool not found"
Ensure exiftool is installed and in your PATH. See [Prerequisites](#prerequisites) section.

### Files not being renamed
Some files may lack EXIF metadata. These will be moved without renaming and logged with a warning.

### Slow performance with duplicate detection
The `--remove-duplicates` flag requires hashing all files, which is slower for large collections. Use without this flag for faster processing if you don't need duplicate detection.

### Handling special cases
Use `--skip-dirs` to exclude backup or temporary folders from processing.

### Processing nested directories
Use `--recursive` to scan and process all subdirectories within your source folder.
