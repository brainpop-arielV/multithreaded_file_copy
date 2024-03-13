use std::fs;
use std::io;
use std::path::Path;
use std::{thread, time};
use fcp::{SafeQueue, FileMover, Cli};

fn main() {
    let now = time::Instant::now();
    let cli = Cli::new();

    let source = &cli.source;
    let destination = &cli.destination;
    let n_workers = match cli.workers {
        Some(i) => i,
        None => {match thread::available_parallelism() {
            Ok(i) => i.get() as i32,
            Err(e) => {
                println!("Couldn't match get number of available cpus, err {} occurred, defaulting to 4 workers", e);
                4  //default number of workers
            }
        }}
    };
    let verbose: bool = cli.verbose;

    let top_level_dir = get_top_level_dir(source);
    let mut file_queue = SafeQueue::new();
    walk_directory(&source, &mut file_queue, destination, &top_level_dir);

    println!("number of files to copy {:?}", file_queue.len());

    let mut handles = vec![];
    let files_per_queue: usize = 10_000;
    for _ in 0..n_workers {
        let handle = get_thread(file_queue.clone(), files_per_queue, verbose);
        handles.push(handle);
    }

    println!("number of handles {}", handles.len());
    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed_time = now.elapsed();
    println!("Running multi threaded copy took {} millis.", elapsed_time.as_millis());

}

fn get_thread(file_queue_copy: SafeQueue<FileMover>, files_per_queue: usize, verbose: bool) -> thread::JoinHandle<()> {

    thread::spawn(move || {
            while !file_queue_copy.is_empty() {
                let files_to_move = file_queue_copy.drain(files_per_queue);
                for file_mover in files_to_move {
                    let _ = copy_file(file_mover, verbose);
                }
            }
        })
}

fn walk_directory(path: &str, file_names: &mut SafeQueue<FileMover>, destination: &str, top_level_dir: &str) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    // If it's a file, add its full path to the vector
                    if let Some(file_name_str) = path.to_str() {
                        let file_mover = get_file_mover_obj(file_name_str.to_string(), destination, top_level_dir);
                        file_names.push(file_mover);
                    }
                } else if path.is_dir() {
                    // If it's a directory, recursively walk through it
                    walk_directory(&path.to_string_lossy(), file_names, destination, top_level_dir);
                }
            }
        }
    }
    else {
        panic!("Could not read source path {}", path);
    }
}

fn get_top_level_dir(destination: &str)-> String {
     // Convert strings to iterators of characters
    let mut destination_manipulator = String::from(destination);

    if destination_manipulator.chars().last().unwrap() == '/' {
        destination_manipulator.pop();
    }

    String::from(destination_manipulator.split("/").last().unwrap())
}

fn get_file_mover_obj(source_path: String, destination: &str, top_level_dir: &str)-> FileMover {

    //formatting the destination to consist of the destination dir, top level common source dir and the subdirectories unique to each file
    let destination = format!("{}{}{}", destination, top_level_dir, source_path.clone().split(top_level_dir).skip(1).collect::<String>());
    FileMover{
        source_path: source_path,
        destination: destination
    }
}

fn copy_file(file_mover: FileMover, verbose: bool)-> io::Result<()> {

    // Create the destination directory and its parent directories if they don't exist
    if let Some(parent_dir) = Path::new(&file_mover.destination).parent() {
        fs::create_dir_all(parent_dir)?;
    }

    // Attempt to copy the file
    match fs::copy(&file_mover.source_path, &file_mover.destination) {
        Ok(_) => {
            if verbose {
                println!("{} copy successfully", &file_mover.destination);
            }
            Ok(())
        },
        Err(err) => {
            eprintln!("Error copying file: {:?}, err is: {}",&file_mover.destination,  err);
            Err(err)
        }
    }
}