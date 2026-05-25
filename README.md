# media-sorter

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
    -d, --destination <DESTINATION>
        Define the destination folder. Defaults to the value of source. Required when using hashes.
    
    -l, --list
        List the files in the source folder. Does not move or rename files.
    
    -o, --output <OUTPUT>
        Output a list of files to a file (requires --list)
    
    -r, --rename
        Renames the files in the current directory without moving them
    
    -q, --quick
        Greatly improves speed, by sorting without checking hashes for duplicates
    
    --skip-dirs <SKIP_DIRS>
        Skip the specified directories (comma-separated). 
        Example: --skip-dirs "Backups,Archive,Temp"
    
    --dont-create-subdirs
        Do not automatically create subdirectories for every year
    
    -h, --help
        Print help information
    
    -V, --version
        Print version information
```

### Common Examples

**Sort files into a different directory and detecting duplicates by calculating file hashes:**
```sh
media-sorter /path/to/unsorted --destination /path/to/destination
```

**Rename files in place (no moving, no subdirectories):**
```sh
media-sorter /path/to/media --rename
```

**List all media files without making changes:**
```sh
media-sorter /path/to/media --list
```

**Save file list to a file:**
```sh
media-sorter /path/to/media --list --output media_files.txt
```

**Fast mode (skip duplicate detection):**
```sh
media-sorter /path/to/media --quick
```

**Skip specific folders:**
```sh
media-sorter /path/to/media --skip-dirs "Archive,Backups,Trash" -d /path/to/destination
```

**Organize without year subdirectories:**
```sh
media-sorter /path/to/media --dont-create-subdirs
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

### Default Mode (with Duplicate Detection)

1. **Gather Hashes**: Scans source folder and computes XXH3 hashes for all media files
2. **Check Destination**: Computes hashes of files already in destination
3. **Compare**: Identifies which files are new or different
4. **Process**: Renames and moves only new/different files

### Quick Mode (Fast, No Duplicate Detection)

1. **Scan**: Identifies all media files in source
2. **Process**: Immediately renames and moves without hash comparison
3. **Speed**: Significantly faster for large collections

### List Mode

- Scans and displays all supported media files
- Optionally saves list to a text file
- No files are modified

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
- **exiftool** (external): EXIF metadata extraction

### Performance

- Parallel processing of large file sets
- Efficient XXH3 hashing (8192-byte buffer chunks)
- Progress feedback to avoid blocking perception
- Optional quick mode sacrifices accuracy for speed

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

### Slow performance
Use `--quick` mode if you don't need duplicate detection. The default mode is thorough but slower for large collections.

### Handling special cases
Use `--skip-dirs` to exclude backup or temporary folders from processing.

## Version

Current version: **1.0.1**
