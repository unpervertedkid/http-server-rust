use std::io::prelude::*;
use std::net::{TcpStream, TcpListener};
use std::io::BufReader;

struct Request {
    _method: String,
    path: String,
    _body: String,
}

impl Request {
    fn read_request(stream: TcpStream) -> Vec<String> {
        let buffer = BufReader::new(stream.try_clone().unwrap());
        buffer
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect()
    }

    fn from_http_request(http_request: Vec<String>) -> Self {
        Self {
            _method: http_request[0].split_whitespace().nth(0).unwrap().to_string(),
            path: http_request[0].split_whitespace().nth(1).unwrap().to_string(),
            _body: http_request[http_request.len() - 1].to_string(),
        }
    }

    fn handle_request(&self, mut stream: TcpStream) {
        if self.path == "/" {
            let response = "HTTP/1.1 200 OK\r\n\r\n";
            stream.write(response.as_bytes()).unwrap();
        } else if self.path.starts_with("/echo") {
            let string = self.path.replace("/echo/", "");
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                string.len(),
                string
            );
            stream.write(response.as_bytes()).unwrap();
        } else {
            let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
            stream.write(response.as_bytes()).unwrap();
        }
    }
}

fn handle_connection(stream: TcpStream) {
    let http_request = Request::read_request(stream.try_clone().unwrap());
    let request = Request::from_http_request(http_request);
    request.handle_request(stream);
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