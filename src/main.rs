use std::collections::HashSet;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use rayon::prelude::*;

use clap::Parser;

mod file_operations;
use file_operations::*;




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

    // list option
    if cli.list {
        let files = get_files(&abs_source, &cli.skip_dirs);
        println!("[i] Found {} files in {}", files.len(), abs_source.display());

        // put the filenames into a file if the output option is set
        if cli.output.is_some() {
            let mut output_file = File::create(cli.output.unwrap()).unwrap();
            let mut contents = String::new();
            for f in &files {
                contents.push_str(&format!("{}\n", f));
                println!("[i] File found: {}", f);
            }
            output_file.write_all(contents.as_bytes()).unwrap();
            return;
        }
        files.iter().for_each(|file| println!("[i] File found: {}", file));
        return;
    }

    // quick-mode
    if cli.quick {
        let source_files = get_files(&abs_source, &cli.skip_dirs);
        let pb = ProgressBar::new(source_files.len() as u64);

        // customizing the progress bar
        pb.set_style(
            ProgressStyle::with_template(
                "[i] Processing files: [{wide_bar}] {pos}/{len} - ETA: {eta} "
            )
            .unwrap()
            .progress_chars("=> "),
        );

        // renaming and moving files
        for file in source_files {
            rename_file(&file, &abs_dest, &mut renamed_files, !cli.dont_create_subdirs).unwrap();
            pb.inc(1);
        }
        pb.finish();
        return;
    }

    // rename-mode
    if cli.rename {
        let source_files = get_files(&abs_source, &cli.skip_dirs);
        let pb = ProgressBar::new(source_files.len() as u64);

        // customizing the progress bar
        pb.set_style(
            ProgressStyle::with_template(
                "[i] Processing files: [{wide_bar}] {pos}/{len} - ETA: {eta} "
            )
            .unwrap()
            .progress_chars("=> "),
        );

        for file in &source_files {
            rename_file(file, &abs_source, &mut renamed_files, false).unwrap();
            pb.inc(1);
        }
        pb.finish();
        return;
    }

    // base case
    if abs_dest == abs_source {
        println!("[!] The source and destination folder are equal.");
        println!("     If you want to sort your images in this folder run media-sorter with the '--quick' option.");
        return;
    }

    println!("[i] Gathering file hashes in source folder {:?}", abs_source);
    let source_files = get_file_hashes(&abs_source, &cli.skip_dirs, HashSet::new());

    println!("[i] Gathering file hashes in destination folder {:?}", abs_dest);
    let dest_files = get_file_hashes(&abs_dest, &cli.skip_dirs, source_files.values().cloned().collect());

    // setting ub the pregress bar
    let pb = ProgressBar::new(source_files.len() as u64);

    // customizing the progress bar
    pb.set_style(
        ProgressStyle::with_template(
            "[i] Processing files: [{wide_bar}] {pos}/{len} - ETA: {eta} "
        )
        .unwrap()
        .progress_chars("=> "),
    );

    for (hash, filepath) in &source_files {
        if !dest_files.contains_key(hash) || ( dest_files.contains_key(hash) && filepath == &dest_files[hash] ){
            rename_file(filepath, &abs_dest, &mut renamed_files, !cli.dont_create_subdirs).unwrap();
        }
        pb.inc(1);
    }
    pb.finish();
    return;        
}




fn print_banner() {
    let banner = r#"
                                $$\ $$\                                                  $$\                         
                                $$ |\__|                                                 $$ |                        
    $$$$$$\$$$$\   $$$$$$\   $$$$$$$ |$$\  $$$$$$\           $$$$$$$\  $$$$$$\   $$$$$$\ $$$$$$\    $$$$$$\   $$$$$$\  
    $$  _$$  _$$\ $$  __$$\ $$  __$$ |$$ | \____$$\ $$$$$$\ $$  _____|$$  __$$\ $$  __$$\\_$$  _|  $$  __$$\ $$  __$$\ 
    $$ / $$ / $$ |$$$$$$$$ |$$ /  $$ |$$ | $$$$$$$ |\______|\$$$$$$\  $$ /  $$ |$$ |  \__| $$ |    $$$$$$$$ |$$ |  \__|
    $$ | $$ | $$ |$$   ____|$$ |  $$ |$$ |$$  __$$ |         \____$$\ $$ |  $$ |$$ |       $$ |$$\ $$   ____|$$ |      
    $$ | $$ | $$ |\$$$$$$$\ \$$$$$$$ |$$ |\$$$$$$$ |        $$$$$$$  |\$$$$$$  |$$ |       \$$$$  |\$$$$$$$\ $$ |      
    \__| \__| \__| \_______| \_______|\__| \_______|        \_______/  \______/ \__|        \____/  \_______|\__|      
    "#;
    println!("{}", banner);
    println!(" {}\n", "─".repeat(120));
}
