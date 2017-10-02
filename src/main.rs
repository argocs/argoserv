#![allow(unused_must_use)]
use std::net::{TcpStream, TcpListener};
use std::path::Path;
use std::fs::{self, ReadDir};
use std::io::{Read, Write};
use std::thread;
use std::env;

/*
 * TODO
 *  > File sending
 *  > Custom directory as an argument
 *  > Gophermap
 */

fn main() {
    let ipaddr = match env::args().nth(1) {
        Some(ipaddr) => ipaddr,
        None => {
            eprintln!(
                "Usage: {} <address to bind to>",
                env::args().nth(0).unwrap_or("./argoserv".to_owned())
            );
            return;
        }
    };
    let server = TcpListener::bind(&format!("{}:70", ipaddr)).unwrap();
    println!(
        "argoserv listening on {}:{}",
        server.local_addr().unwrap().ip(),
        server.local_addr().unwrap().port()
    );
    for client in server.incoming() {
        match client {
            Ok(client) => {
                println!("CONNECT <- {}", client.peer_addr().unwrap().ip());
                thread::spawn(move || { handle(client); });
            }
            Err(e) => eprintln!("ERROR : {:?}", e),
        }
    }
}

fn handle(mut client: TcpStream) {
    let mut buffer = [0; 512];
    let read = client.read(&mut buffer);
    match read {
        Ok(n) => {
            //The client has sent the request. buffer = [ selector... \r\n ]
            let mut selector = String::new();
            let mut index = 0;
            'outer: while index < n {
                selector.push(buffer[index] as char);
                if index + 2 < buffer.len() {
                    //If the next two bytes are \r\n, we're done parsing the selector.
                    if buffer[index + 1] == 0x0d && buffer[index + 2] == 0x0a {
                        break 'outer;
                    }
                }
                index += 1;
            }
            println!(
                "LIST '{}' -> {}",
                selector,
                client.peer_addr().unwrap().ip()
            );
            // Prepend a "." to the selector to keep us in the working directory of the server
            if selector == "/" {
                selector = ".".to_owned();
            } else if selector.starts_with("/") {
                selector = ".".to_owned() + &selector;
            }
            let cwd = env::current_dir().unwrap();
            let mut path = Path::new(&selector);
            // Check if the path the client requested is outside of our jail
            if !path.canonicalize().unwrap().starts_with(
                cwd.as_path()
                    .canonicalize()
                    .unwrap()
                    .as_path(),
            )
            {
                //Reset them to the root of our jail
                path = cwd.as_path();
            }
            if path.is_file() {
                // TODO implement sending files
            } else {
                let iter: ReadDir = fs::read_dir(&path).unwrap();
                for entry in iter {
                    if let Ok(entry) = entry {
                        let ltype = if entry.path().is_file() { "0" } else { "1" };
                        let selector =
                            format!("{}/{}", selector, entry.file_name().to_str().unwrap());
                        client.write(
                            format(
                                ltype,
                                entry.file_name().to_str().unwrap(),
                                selector.as_str(),
                                "192.168.1.115",
                                70,
                            ).as_bytes(),
                        );
                    }
                }
            }
        }
        Err(e) => eprintln!("READ ERROR {} <- {}", e, client.peer_addr().unwrap().ip()),
    }
}

fn format(ltype: &str, display: &str, selector: &str, domain: &str, port: u32) -> String {
    format!(
        "{}{}\t{}\t{}\t{}\r\n",
        ltype,
        display,
        selector,
        domain,
        port
    )
}
