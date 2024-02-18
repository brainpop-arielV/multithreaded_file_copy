use std::fs;
use std::env;
use std::io;
use std::path::Path;
use std::{thread, time};
use std::borrow::Borrow;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// struct SafeQueue<T> {
//     queue: Arc<Mutex<Vec<T>>>
// }

// impl<T> Clone for SafeQueue<T> {
//     fn clone(&self) -> Self {
//         Self{
//             queue: self.queue.clone()
//         }
//     }
// }

// impl<T> SafeQueue<T> {
//     fn new() -> SafeQueue<T> {
//         SafeQueue {
//             queue: Arc::new(Mutex::new(Vec::new()))
//         }
//     }

//     fn is_empty(&self) -> bool {
//         let queue = self.queue.lock().unwrap();
//         queue.is_empty()
//     }

//     fn push(&self, item: T) {
//         let mut queue = self.queue.lock().unwrap();
//         queue.push(item)
//     }

//     fn pop(&self) -> Option<T> {
//         let mut queue = self.queue.lock().unwrap();
//         queue.pop()
//     }
// }

#[derive(PartialEq, Debug, Clone)]
struct FileMover{
    source_path: String,
    destination: String
}

fn main() {
    let now = time::Instant::now();
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <directory_path>, <destination_path>", args[0]);
        std::process::exit(1);
    }

    //args[0] is the program name, which we don't need here.
    let top_level_dir = &args[1];
    let destination = &args[2];
    let common_prefix_len = get_common_prefix_len(top_level_dir, destination);
    let mut test_vec = vec![];

    walk_directory(&top_level_dir, &mut test_vec, destination, common_prefix_len);

    //let mut file_queue = SafeQueue::<FileMover>::new();

    let mut i = 0;
    let thread_limit = 10;
    let file_num = test_vec.len() as i32;
    let mut thread_files_limit: f64 = ((file_num / thread_limit) as f64).ceil();
    let mut handles = vec![];

    loop {
        if i + thread_files_limit as i32 >= file_num {
            let files_to_move = test_vec[i as usize..].to_vec();
            let handle = thread::spawn(move || {
                for file_mover in files_to_move {
                    copy_file(file_mover);
                    //println!("thread {} copy done", i);
                }
            });
            handles.push(handle);
            break;
        }

        if i == file_num {
            break;
        }

        let files_to_move = test_vec[i as usize..i as usize + thread_files_limit as usize].to_vec();
        let handle = thread::spawn(move || {
                for file_mover in files_to_move {
                    copy_file(file_mover);
                    //println!("thread {} copy done", i);
                }
            });
            handles.push(handle);
        i = i + thread_files_limit as i32

    }

    // for file_mover in test_vec {
    //     copy_file(file_mover);
    // }
    // for i in (0..10) {
    //     let mut file_queue_copy = file_queue.clone();
    //     let handle = thread::spawn(move || {
    //         while !file_queue_copy.is_empty() {
    //             let file_mover = file_queue_copy.pop();
    //             copy_file(file_mover.unwrap());
    //             println!("Thread {} copy successful", i);
    //         }
    //     });
    //     handles.push(handle);
    // }

    println!("number of handles {}", handles.len());
    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed_time = now.elapsed();
    println!("Running multi threaded copy took {} seconds.", elapsed_time.as_secs());

}

fn walk_directory(path: &str, file_names: &mut Vec<FileMover>, destination: &str, common_prefix_len: usize) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    // If it's a file, add its full path to the vector
                    if let Some(file_name_str) = path.to_str() {
                        file_names.push(get_file_mover_obj(file_name_str.to_string(), destination, common_prefix_len));
                    }
                } else if path.is_dir() {
                    // If it's a directory, recursively walk through it
                    walk_directory(&path.to_string_lossy(), file_names, destination, common_prefix_len);
                }
            }
        }
    }
}

fn get_common_prefix_len(source_path: &str, destination: &str)-> usize {
     // Convert strings to iterators of characters
    let mut iter1 = source_path.chars();
    let mut iter2 = destination.chars();

    // Find the common prefix length
    iter1
        .zip(&mut iter2)
        .take_while(|(c1, c2)| c1 == c2)
        .count()
}

fn get_file_mover_obj(source_path: String, destination: &str, common_prefix_len: usize)-> FileMover {
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
            //println!("{} copy successfully", &file_mover.source_path);
            Ok(())
        },
        Err(err) => {
            eprintln!("Error copying file: {}", err);
            Err(err)
        }
    }
}