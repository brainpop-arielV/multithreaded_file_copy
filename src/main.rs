use std::fs;
use std::env;
use std::io;
use std::path::Path;
use std::collections::VecDeque;

#[derive(PartialEq, Debug)]
struct FileMover{
    source_path: String,
    destination: String
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <directory_path>, <destination_path>", args[0]);
        std::process::exit(1);
    }

    //args[0] is the program name, which we don't need here.
    let top_level_dir = &args[1];
    let destination = &args[2];
    let mut file_names = Vec::new();
    walk_directory(&top_level_dir, &mut file_names);

    let mut file_queue: VecDeque<FileMover> = VecDeque::new();

    for file_name in file_names {
        let file_mover = get_file_mover_obj(file_name, destination);
        file_queue.push_front(file_mover);
    }

    match copy_file(file_queue.pop_front().unwrap()) {
        Err(x) => {
            eprintln!("Error occurred while copying file {:?}", x);
        }
        Ok(_) => {
            println!("all good, nothing to see here");
        }
    }
}

fn walk_directory(path: &str, file_names: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    // If it's a file, add its full path to the vector
                    if let Some(file_name_str) = path.to_str() {
                        file_names.push(file_name_str.to_string());
                    }
                } else if path.is_dir() {
                    // If it's a directory, recursively walk through it
                    walk_directory(&path.to_string_lossy(), file_names);
                }
            }
        }
    }
}

fn get_file_mover_obj(source_path: String, destination: &str)-> FileMover {
     // Convert strings to iterators of characters
    let mut iter1 = source_path.chars();
    let mut iter2 = destination.chars();

    // Find the common prefix length
    let common_prefix_len = iter1
        .zip(&mut iter2)
        .take_while(|(c1, c2)| c1 == c2)
        .count();

    let diff1 = &source_path[common_prefix_len..];
    let diff2 = &destination[common_prefix_len..];

    let interpolated_destination: String = format!{"{}/{}", diff2.to_string(), diff1.to_string()};

    FileMover{
        source_path: source_path,
        destination: format!("{}{}", &destination[..common_prefix_len], &interpolated_destination)
    }
}

fn copy_file(file_mover: FileMover)-> io::Result<()> {

    // Create the destination directory and its parent directories if they don't exist
    if let Some(parent_dir) = Path::new(&file_mover.destination).parent() {
        fs::create_dir_all(parent_dir)?;
    }

    // Attempt to copy the file
    match fs::copy(&file_mover.source_path, &file_mover.destination) {
        Ok(_) => {
            println!("{} copy successfully", &file_mover.source_path);
            Ok(())
        },
        Err(err) => {
            eprintln!("Error copying file: {}", err);
            Err(err)
        }
    }
}