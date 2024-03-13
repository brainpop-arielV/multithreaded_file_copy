use std::sync::{Arc, Mutex};
use clap::Parser;

#[derive(PartialEq, Debug, Clone)]
pub struct FileMover{
    pub source_path: String,
    pub destination: String
}

pub struct SafeQueue<T> {
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
    pub fn new() -> SafeQueue<T> {
    SafeQueue {
            queue: Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn is_empty(&self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.is_empty()
    }

    pub fn push(&self, item: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(item)
    }

    pub fn len(&self) -> usize {
        let queue = self.queue.lock().unwrap();
        queue.len()
    }

    pub fn drain(&self, n_elements: usize) -> Vec<T> {
        let mut queue = self.queue.lock().unwrap();
        if n_elements > queue.len()
        {
            return queue.drain(..).collect();
        }
        queue.drain(..n_elements).collect()
    }
}

#[derive(Parser)]
#[command(name="fcp",version="0.3", about="multi threaded file copying", long_about = None)]
pub struct Cli {
    pub source: String,
    pub destination: String,
    #[arg(short, long="workers")]
    pub workers: Option<i32>,
    #[arg(short, long="verbose")]
    pub verbose: bool
}

impl Cli {
    pub fn new() -> Self {
        Cli::parse()
    }
}