use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "media_sorter")]
#[command(version = "1.0.3")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Define the source folder
    pub source:PathBuf,

    #[arg(short, long, conflicts_with = "list")]
    /// Define the destination folder. Defaults to the value of source
    pub destination: Option<PathBuf>,
    
    #[arg(short, long, conflicts_with = "destination")]
    /// List the files in the source folder. Does not move or rename files.
    pub list:bool,

    #[arg(short, long, requires = "list")]
    /// output a list of files to a file
    pub output:Option<String>,

    #[arg(short, long, conflicts_with = "destination", conflicts_with = "list", conflicts_with = "quick")]
    /// Renames the files in the current directory without moving them.
    pub rename:bool,

    #[arg(short, long, conflicts_with = "list")]
    /// Greately improves speed, but does not check for duplicates. Does not override!
    pub quick:bool,

    #[arg(long, num_args = 0.., value_delimiter = ',')]
    /// Skips the directories, allows multiple entries separated by ','
    pub skip_dirs:Vec<String>,

    #[arg(long, conflicts_with = "list")]
    /// Does not automatically create subdirectories for every year (2000, 2001, ...)
    pub dont_create_subdirs:bool,
}