use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::io::{BufReader, Read};
use std::fs;
use std::process::Command;
use anyhow::Ok;
use chrono::{DateTime, NaiveDateTime, Timelike, Datelike};
use xxhash_rust::xxh3::Xxh3;


const IMAGE_EXTENSIONS: [&str; 5] = ["jpg", "jpeg", "png", "gif", "bmp"];
const VIDEO_EXTENSIONS: [&str; 5] = ["mp4", "avi", "mkv", "mov", "flv"];

#[derive(Clone)]
pub struct MediaFile {
    pub file_loc: PathBuf,
    pub hash: Option<u64>
}

impl MediaFile {
    pub fn new(file_location: &Path) -> anyhow::Result<Self> {
        if !file_location.is_file(){
            return Err(anyhow::anyhow!("not a file: {}", file_location.display()));
        };
        let mf = MediaFile {
            file_loc: file_location.into(),
            hash: None
        };

        if mf.is_media_file() {
            return Ok(mf);
        }
        Err(anyhow::anyhow!("not a media file: {}", file_location.display()))
    }

    pub fn load_hash(&mut self) -> anyhow::Result<()> {
        let file = fs::File::open(&self.file_loc)?;
        let mut reader = BufReader::new(file);
        let mut buffer = [0; 8192];
        let mut hasher = Xxh3::new();
        
        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 { break; }
            hasher.update(&buffer[..count]);
        }
        
        let hash = hasher.digest();
        self.hash = Some(hash);
        Ok(())
    }

    pub fn new_filename(&self, dest_folder: &Path, used_filenames: &mut HashSet<String>, rename: bool, create_subfolders:bool) -> Option<PathBuf> {
        let filename = self.file_name()?;
        let mut dest_path = dest_folder.to_path_buf();

        if let Some(dt) = self.get_date_taken() {
            if create_subfolders {
                dest_path.push(dt.year().to_string());
            }
            
            if rename {
                let base_filename = format!(
                    "{}_{}{:02}{:02}_{:02}{:02}{:02}.{}",
                    if self.is_video_file() { "VID" } else { "IMG" },
                    dt.year(), dt.month(), dt.day(),
                    dt.hour(), dt.minute(), dt.second(),
                    self.file_loc.extension()?.to_str()?
                );

                let mut new_filename = base_filename.clone();
                let mut counter: u32 = 1;
                while used_filenames.contains(&new_filename) {
                    let ext = self.file_loc.extension()?.to_str()?;
                    let stem_end = base_filename.len() - ext.len() - 1;
                    new_filename = format!("{}_{:04}.{}", &base_filename[..stem_end], counter, ext);
                    counter += 1;
                }

                used_filenames.insert(new_filename.to_string());
                dest_path.push(new_filename);
            } 
            else {
                dest_path.push(filename);
            }       
        } 
        else {
            // Fallback: Leave in source dir 
            eprintln!("[!] No EXIF date found: {}", self);
            dest_path.push(filename);
        }
        Some(dest_path)
    }

    pub fn get_date_taken(&self) -> Option<NaiveDateTime> {
        let path_str = self.file_loc.to_str()?;

        let output = Command::new("exiftool")
            .args([
                "-DateTimeOriginal",
                "-CreateDate",
                "-MediaCreateDate",
                "-TrackCreateDate",
                "-CreationTime",
                "-FileModifyDate",  // last resort fallback
                "-s3",
                "-f",
                path_str,
            ])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        stdout
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && *l != "-")
            .find_map(|l| {
                // "%Y:%m:%d %H:%M:%S" — standard EXIF
                NaiveDateTime::parse_from_str(l, "%Y:%m:%d %H:%M:%S")
                    // "%Y:%m:%d %H:%M:%S%z" — with timezone offset (e.g. FileModifyDate)
                    .or_else(|_| {
                        DateTime::parse_from_str(l, "%Y:%m:%d %H:%M:%S%z")
                            .map(|dt| dt.naive_local())
                    })
                    // "Sat 02 May 2026 16:23:10 CEST" — CreationTime format
                    .or_else(|_| {
                        DateTime::parse_from_str(l, "%a %d %b %Y %H:%M:%S %Z")
                            .map(|dt| dt.naive_local())
                    })
                    .ok()
            })
    }

    pub fn is_image_file(&self) -> bool {
        IMAGE_EXTENSIONS.iter().any(|ext| self.file_loc.extension().unwrap_or_default() == *ext)
    }

    pub fn is_video_file(&self) -> bool {
        VIDEO_EXTENSIONS.iter().any(|ext| self.file_loc.extension().unwrap_or_default() == *ext)
    }

    pub fn is_media_file(&self) -> bool {
        self.is_image_file() || self.is_video_file()
    }

    pub fn file_name(&self) -> Option<&std::ffi::OsStr>{
        self.file_loc.file_name()
    }

    pub fn parent(&self) -> &Path {
        self.file_loc.parent().unwrap_or(Path::new("."))
    }
}

impl PartialEq for MediaFile {
    fn eq(&self, other: &Self) -> bool {
        self.file_loc == other.file_loc
    }
}

impl Eq for MediaFile {}

impl Hash for MediaFile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.file_loc.hash(state);
    }
}

impl Display for MediaFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_loc.to_str().unwrap_or("File could not be identified"))
    }
}
