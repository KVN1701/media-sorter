use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "media-sorter")]
#[command(version = "1.1.0")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Define the source folder
    pub source:PathBuf,

    #[arg(short, long = "dest", conflicts_with = "list", value_name = "FOLDER")]
    /// Define the destination folder. Defaults to the source folder.
    pub destination: Option<PathBuf>,

    #[arg(short, long)]
    /// Recursively gathers files/ file hashes in the destination folder and source folder.
    pub recursive:bool,
    
    #[arg(short = 'D', long, conflicts_with = "list")]
    /// Removes duplicates of files. Causes a great increase in runtime.
    pub remove_duplicates:bool,
    
    #[arg(short, long, conflicts_with = "destination")]
    /// List the files in the source folder. Does not move or rename files.
    pub list:bool,
    
    #[arg(short, long, requires = "list")]
    /// output a list of files to a file
    pub output:Option<String>,
    
    #[arg(short = 'n', long, conflicts_with = "list")]
    /// Renames the files during the operation.
    pub rename:bool,
    
    #[arg(long, num_args = 0.., value_delimiter = ',')]
    /// Skips the directories, allows multiple entries separated by ','
    pub skip_dirs:Vec<String>,
    
    #[arg(short, long, conflicts_with = "list")]
    /// Does automatically create subdirectories for every year (2000, 2001, ...)
    pub sort:bool,
}