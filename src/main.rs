use std::collections::HashSet;
use banner::print_banner;
use tool::*;
use parser::Cli;
use clap::Parser;

use crate::file::MediaFile;

mod sorter;
mod banner;
mod tool;
mod parser;
mod file;

fn main() {
    print_banner();

    let mut renamed_files: HashSet<MediaFile> = HashSet::new();

    // Parser
    let cli = Cli::parse();
    let source_dir = cli.source.clone();
    let destination_dir = cli.destination.clone().unwrap();

    // get the absolute paths
    let abs_source = match source_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => std::env::current_dir().unwrap().join(&source_dir),
    };
    let abs_dest = match destination_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => std::env::current_dir().unwrap().join(&source_dir),
    };

    // list, hashing, rename, quick
    if cli.list {
        list_files(&abs_source, &cli.skip_dirs, cli.output);
    }
    else if cli.rename {
        rename_in_place(&abs_source, &cli.skip_dirs, &mut renamed_files);
    }
    else if cli.quick {
        quick_mode(&abs_source, &abs_dest, &cli.skip_dirs, &mut renamed_files, cli.dont_create_subdirs);
    }
    // no option set defaulting to moving with hashing
    else {
        move_with_hashing(&abs_source, &abs_dest, &cli.skip_dirs, &mut renamed_files, cli.dont_create_subdirs);
    }         
}


