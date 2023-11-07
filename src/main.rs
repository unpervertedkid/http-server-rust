use std::{
    io::{prelude::*, BufReader, Write},
    net::TcpListener,
};

fn handle_connection(mut stream: std::net::TcpStream) {
    let buffer = BufReader::new(stream.try_clone().unwrap());
    let http_request: Vec<_> = buffer
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let path = http_request[0].split_whitespace().nth(1).unwrap();

    if path.starts_with("/") {
        let response = "HTTP/1.1 200 OK\r\n\r\n";
        stream.write(response.as_bytes()).unwrap();
    } else {
        let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
        stream.write(response.as_bytes()).unwrap();
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
