use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::{env, fs};

struct Request {
    _method: String,
    path: String,
    headers: HashMap<String, String>,
    _body: String,
    directory: Option<String>,
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

    fn from_http_request(http_request: Vec<String>, directory: Option<String>) -> Self {
        let mut headers = HashMap::new();
        for line in http_request.iter().skip(1) {
            let parts: Vec<&str> = line.split(": ").collect();
            if parts.len() == 2 {
                headers.insert(parts[0].to_string(), parts[1].to_string());
            }
        }

        Self {
            _method: http_request[0]
                .split_whitespace()
                .nth(0)
                .unwrap()
                .to_string(),
            path: http_request[0]
                .split_whitespace()
                .nth(1)
                .unwrap()
                .to_string(),
            headers,
            _body: http_request[http_request.len() - 1].to_string(),
            directory,
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

    fn handle_files(&self, mut stream: TcpStream) {
        if let Some(directory) = &self.directory {
            let filename = &self.path[7..]; // remove "/files/" from the path
            let filepath = format!("{}/{}", directory, filename);

            match fs::read(&filepath) {
                Ok(contents) => {
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                        contents.len(),
                        String::from_utf8_lossy(&contents)
                    );
                    stream.write(response.as_bytes()).unwrap();
                }
                Err(_) => {
                    let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
                    stream.write(response.as_bytes()).unwrap();
                }
            }
        } else {
            let response = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n";
            stream.write(response.as_bytes()).unwrap();
        }
    }

    fn handle_request(&self, stream: TcpStream) {
        if self.path == "/" {
            self.handle_root(stream);
        } else if self.path.starts_with("/echo") {
            self.handle_echo(stream);
        } else if self.path == "/user-agent" {
            self.handle_user_agent(stream);
        } else if self.path.starts_with("/files") {
            self.handle_files(stream);
        }
        else {
            self.handle_not_found(stream);
        }
    }
}

fn handle_connection(directory: Option<String>, stream: TcpStream) {
    let http_request = Request::read_request(stream.try_clone().unwrap());
    let request = Request::from_http_request(http_request, directory);
    request.handle_request(stream);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let directory = if args.len() > 2 { Some(args[2].clone()) } else { None };

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let directory = directory.clone();
                thread::spawn(|| {
                    handle_connection(directory, stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
