use std::collections::HashSet;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use clap::Parser;

mod file_operations;
mod banner;
mod tool;

use banner::print_banner;
use tool::*;


#[derive(Parser)]
#[command(name = "media_sorter")]
#[command(version = "1.0.1")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Define the source folder
    source:PathBuf,

    #[arg(short, long, conflicts_with = "list")]
    /// Define the destination folder. Defaults to the value of source
    destination: Option<PathBuf>,
    
    #[arg(short, long, conflicts_with = "destination")]
    /// List the files in the source folder. Does not move or rename files.
    list:bool,

    #[arg(short, long, requires = "list")]
    /// output a list of files to a file
    output:Option<String>,

    #[arg(short, long, conflicts_with = "destination", conflicts_with = "list", conflicts_with = "quick")]
    /// Renames the files in the current directory without moving them.
    rename:bool,

    #[arg(short, long, conflicts_with = "list")]
    /// Greately improves speed, but does not check for duplicates. Does not override!
    quick:bool,

    #[arg(long, num_args = 0.., value_delimiter = ',')]
    /// Skips the directories, allows multiple entries separated by ','
    skip_dirs:Vec<String>,

    #[arg(long, conflicts_with = "list")]
    /// Does not automatically create subdirectories for every year (2000, 2001, ...)
    dont_create_subdirs:bool,
}


fn main() {
    print_banner();

    let mut renamed_files: HashSet<String> = HashSet::new();

    // Parser
    let cli = Cli::parse();
    let source_dir = cli.source.clone();
    let destination_dir = cli.destination.clone().unwrap_or(source_dir.clone());

    // get the absolute paths
    let abs_source = match source_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => std::env::current_dir().unwrap().join(&source_dir),
    };
    
    let abs_dest = match destination_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => std::env::current_dir().unwrap().join(&source_dir),
    };

          
}


