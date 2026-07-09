use std::path::PathBuf;
use std::{collections::HashSet, fs::File};
use std::io::Write;
use banner::print_banner;
use indicatif::{ProgressBar, ProgressStyle};
use parser::Cli;
use clap::Parser;
use crate::file::MediaFile;
use crate::sorter::{get_file_hashes, get_files, move_file};

mod sorter;
mod banner;
mod parser;
mod file;

fn main() {
    // TODO: Implement AppError
    print_banner();

    // Parser
    let cli = Cli::parse();

    // get the absolute paths
    let abs_source = match cli.source.canonicalize() {
        Ok(p) => p,
        Err(_) => std::env::current_dir().unwrap().join(&cli.source),
    };
    let abs_dest = match cli.destination.as_ref() {
        Some(d) => match d.canonicalize() {
            Ok(p) => Some(p),
            Err(_) => Some(std::env::current_dir().unwrap().join(&d)),
        }
        None => None
    };

    // get files
    println!("[i] Gathering files in {}", cli.source.display());
    let source_files = match cli.remove_duplicates {
        true => get_file_hashes(&abs_source, &cli.skip_dirs, &HashSet::new(), &cli.recursive),
        false => get_files(&abs_source, &cli.recursive, &cli.skip_dirs),
    };
    println!("[i] Found {} media files in {}", source_files.len(), cli.source.display());


    let dest_files = match &abs_dest {
        Some(dest) => {
            println!("[i] Gathering files in {}", dest.display());

            if !cli.remove_duplicates {
                get_files(&dest, &cli.recursive, &cli.skip_dirs)
            } else {
                get_file_hashes(&dest, &cli.skip_dirs, &source_files, &cli.recursive)
            }
        },
        None => HashSet::new()
    };
    if dest_files.len() > 0 {
        println!("[i] Found {} media files in {}", dest_files.len(), abs_dest.as_ref().unwrap().display());
    }

    let move_dest = match abs_dest {
        Some(d) => d,
        None => abs_source
    };

    handler(source_files, move_dest, cli);     
}


fn handler(source_files: HashSet<MediaFile>, move_dest: PathBuf, cli: Cli) {
    if let Some(out) = cli.output {
        println!("[!] Printing all media files in source to output file {}", out);
        let mut output_file = File::create(out).unwrap();
        let mut contents = String::new();
        for f in &source_files {
            contents.push_str(&format!("{}\n", f));
            println!("[i] Media file found: {}", f);
        }
        output_file.write_all(contents.as_bytes()).unwrap();
    }
    
    if cli.list {
        // listing the found files
        source_files.iter().for_each(|file| println!("[i] Media file found: {}", file));
        return;
    }

    // handle files/ move files
    let pb = ProgressBar::new(source_files.len() as u64);

    // customizing the progress bar
    pb.set_style(
        ProgressStyle::with_template(
            "[i] Processing files: [{wide_bar}] {pos}/{len} - ETA: {eta} "
        )
        .unwrap()
        .progress_chars("=> "),
    );

    let mut used_filenames: HashSet<String> = HashSet::new();
    for mf in source_files {
        move_file(&mf, &move_dest, &mut used_filenames, cli.rename, cli.sort).unwrap();
        pb.inc(1);
    }
    pb.finish();
}


