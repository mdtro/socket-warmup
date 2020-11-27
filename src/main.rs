use std::thread;
use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:5656").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        thread::spawn(move || {
            handle_connection(stream)
        });
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
                    println!("Received: {:?}", &buffer[0..size]);
                    stream.write(&buffer[0..size]).unwrap();
                }
            }
            Err(e) => {
                panic!(e);
            }
        }
    }
}
