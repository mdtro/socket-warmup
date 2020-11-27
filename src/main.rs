use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::sync::{mpsc, Arc, Mutex};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// This is using the examples in the Rust Book to multi-thread
    /// this simple echo server: https://doc.rust-lang.org/book/ch20-02-multithreaded.html
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        return ThreadPool { workers, sender};
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} received job. Executing... ", id);
            job();
        });
        
        return Worker { id, thread }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:5656").unwrap();
    let pool = ThreadPool::new(5);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        })
    }
}

fn handle_connection(mut stream: TcpStream) {
    loop {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(size) => {
                if size == 0 {
                    stream.shutdown(Shutdown::Both).unwrap();
                } else {
                    println!("Received: {:?}", String::from_utf8_lossy(&buffer[0..size]));
                    stream.write(&buffer[0..size]).unwrap();
                }
            }
            Err(e) => {
                panic!(e);
            }
        }
    }
}
