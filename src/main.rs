use std::fs;
use std::io;
use std::path::Path;
use std::{thread, time};
use std::sync::{Arc, Mutex};
use clap::Parser;

#[derive(Parser)]
#[command(name="fcp",version="0.3", about="multi threaded file copying", long_about = None)]
struct Cli {
    source: String,
    destination: String,
    #[arg(short, long="workers")]
    workers: Option<i32>,
    #[arg(short, long="verbose")]
    verbose: bool
}

struct SafeQueue<T> {
    queue: Arc<Mutex<Vec<T>>>
}

impl<T> Clone for SafeQueue<T> {
    fn clone(&self) -> Self {
        Self{
            queue: self.queue.clone()
        }
    }
}

impl<T> SafeQueue<T> {
    fn new() -> SafeQueue<T> {
        SafeQueue {
            queue: Arc::new(Mutex::new(Vec::new()))
        }
    }

    fn is_empty(&self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.is_empty()
    }

    fn push(&self, item: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(item)
    }

    fn len(&self) -> usize {
        let queue = self.queue.lock().unwrap();
        queue.len()
    }

    fn drain(&self, n_elements: usize) -> Vec<T> {
        let mut queue = self.queue.lock().unwrap();
        if n_elements > queue.len()
        {
            return queue.drain(..).collect();
        }
        queue.drain(..n_elements).collect()
    }
}

#[derive(PartialEq, Debug, Clone)]
struct FileMover{
    source_path: String,
    destination: String
}

fn main() {
    let now = time::Instant::now();
    let cli = Cli::parse();

    // let args: Vec<String> = env::args().collect();

    // if args.len() !=4 {
    //     eprintln!("Usage: {} <directory_path>, <destination_path>, <n_workers>", args[0]);
    //     std::process::exit(1);
    // }

    //args[0] is the program name, which we don't need here.
    let source = &cli.source;
    let destination = &cli.destination;
    let n_workers = match cli.workers {
        Some(i) => i,
        None => 4 //default number of workers
    };
    let verbose: bool = cli.verbose;

    let top_level_dir = get_top_level_dir(source);
    let mut file_queue = SafeQueue::new();

    walk_directory(&source, &mut file_queue, destination, &top_level_dir);

    let mut handles = vec![];

    println!("number of files to copy {:?}", file_queue.len());

    let files_per_queue: usize = 10_000;
    for _ in 0..n_workers {
        let file_queue_copy = file_queue.clone();
        let handle = thread::spawn(move || {
            while !file_queue_copy.is_empty() {
                let files_to_move = file_queue_copy.drain(files_per_queue);
                for file_mover in files_to_move {
                    let _ = copy_file(file_mover, &verbose);
                }
            }
        });
        handles.push(handle);
    }

    println!("number of handles {}", handles.len());
    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed_time = now.elapsed();
    println!("Running multi threaded copy took {} millis.", elapsed_time.as_millis());

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

fn copy_file(file_mover: FileMover, verbose: &bool)-> io::Result<()> {

    // Create the destination directory and its parent directories if they don't exist
    if let Some(parent_dir) = Path::new(&file_mover.destination).parent() {
        fs::create_dir_all(parent_dir)?;
    }

    // Attempt to copy the file
    match fs::copy(&file_mover.source_path, &file_mover.destination) {
        Ok(_) => {
            if *verbose {
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