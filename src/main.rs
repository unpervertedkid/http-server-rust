use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{TcpStream, TcpListener};
use std::io::BufReader;

struct Request {
    _method: String,
    path: String,
    _body: String,
    headers: HashMap<String, String>,
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
        let mut headers = HashMap::new();
        for line in http_request.iter().skip(1) {
            let parts: Vec<&str> = line.split(": ").collect();
            if parts.len() == 2 {
                headers.insert(parts[0].to_string(), parts[1].to_string());
            }
        }

        Self {
            _method: http_request[0].split_whitespace().nth(0).unwrap().to_string(),
            path: http_request[0].split_whitespace().nth(1).unwrap().to_string(),
            _body: http_request[http_request.len() - 1].to_string(),
            headers,
        }
    }

    fn handle_root(&self, mut stream: TcpStream) {
        let response = "HTTP/1.1 200 OK\r\n\r\n";
        stream.write(response.as_bytes()).unwrap();
    }

    fn handle_echo(&self, mut stream: TcpStream) {
        let string = self.path.replace("/echo/", "");
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            string.len(),
            string
        );
        stream.write(response.as_bytes()).unwrap();
    }

    fn handle_user_agent(&self, mut stream: TcpStream) {
        if let Some(user_agent) = self.headers.get("User-Agent") {
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                user_agent.len(),
                user_agent
            );
            stream.write(response.as_bytes()).unwrap();
        }
    }

    fn handle_not_found(&self, mut stream: TcpStream) {
        let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
        stream.write(response.as_bytes()).unwrap();
    }

    fn handle_request(&self, stream: TcpStream) {
        if self.path == "/" {
            self.handle_root(stream);
        } else if self.path.starts_with("/echo") {
            self.handle_echo(stream);
        } else if self.path == "/user-agent" {
            self.handle_user_agent(stream);
        } else {
            self.handle_not_found(stream);
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