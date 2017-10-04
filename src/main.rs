#![allow(unused_must_use)]
use std::net::{TcpStream, TcpListener};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Write, BufReader};
use std::thread;
use std::env;

fn main() {
    let ipaddr = match env::args().nth(1) {
        Some(ipaddr) => ipaddr,
        None => {
            eprintln!("Missing IP address to bind to.");
            return;
        }
    };
    let directory = match env::args().nth(2) {
        Some(dir) => dir,
        None => ".".to_owned(),
    };
    let address = &format!("{}:70", ipaddr);
    let server = match TcpListener::bind(address) {
        Ok(server) => server,
        Err(_) => {
            eprintln!("Failed to start server.");
            return;
        }
    };
    println!("argoserv listening on {}:70", ipaddr);

    for client in server.incoming() {
        match client {
            Ok(client) => {
                let addr = ipaddr.clone();
                let dir = directory.clone();
                thread::spawn(move || { handle(client, addr, dir); });
            }
            Err(e) => eprintln!("ERROR : {:?}", e),
        }
    }
}

fn handle(mut client: TcpStream, ipaddr: String, directory: String) {
    let client_ip = client.peer_addr().unwrap().ip();

    let mut buffer = [0; 1024];
    let read = client.read(&mut buffer);
    let mut selector = match read {
        Ok(n) => {
            let mut wordbuf = String::new();
            let mut index = 0;
            'outer: while index < n {
                wordbuf.push(buffer[index] as char);
                if index + 2 < buffer.len() {
                    //If the next two bytes are \r\n, we're done parsing the selector.
                    if buffer[index + 1] == 0x0d && buffer[index + 2] == 0x0a {
                        break 'outer;
                    }
                }
                index += 1;
            }
            wordbuf
        }
        Err(e) => {
            eprintln!("READ ERROR {} <- {}", e, client_ip);
            return;
        }
    };
    println!("LIST '{}' -> {}", selector, client_ip);
    // Prepend a "." to the selector to keep us in the working directory of the server
    if selector == "/" {
        selector = ".".to_owned();
    } else if selector.starts_with("/") {
        selector = ".".to_owned() + &selector;
    }

    let _selector = selector.clone();
    let home = Path::new(&directory); //The home directory of the server
    let mut dest = Path::new(&directory).to_path_buf();
    dest.push(_selector);
    let mut dest = dest.as_path();

    let canonical_path = dest.canonicalize().unwrap_or(PathBuf::new());
    let canonical_cwd = home.canonicalize().unwrap_or(PathBuf::new());
    if !dest.exists() {
        client.write(b"iNo content.\r\n");
        client.write(&format!("1Return home\t/\t{}\t70\r\n.", ipaddr).as_bytes());
        return;
    } else if !canonical_path.starts_with(canonical_cwd) {
        dest = Path::new(&directory);
    }

    //Serve the client
    if dest.is_file() {
        //Send the data of this file.
        let mut file = File::open(dest).unwrap();
        let mut buffer = vec![];
        match file.read_to_end(&mut buffer) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("READ FILE ERROR : {} -> {}", e, client_ip);
                client.write(b"Error reading requested file.\r\n.");
                return;
            }
        }
        match client.write_all(&buffer) {
            Ok(_) => println!("SEND '{}' -> {}", selector, client_ip),
            Err(e) => eprintln!("SEND FILE ERROR: {} -> {}", e, client_ip),
        }
    } else {
        let mut menu_file = dest.to_path_buf();
        menu_file.push("index.gph");
        let menu_file = menu_file.as_path();
        if menu_file.exists() {
            let contents = read_file(&menu_file);
            match contents {
                Ok(contents) => {
                    for line in contents.lines() {
                        let mut line = line.to_owned();
                        line += "\r\n";
                        line = line
                                .replace("$ADDRESS$", ipaddr.as_ref()) // IP replacement
                                .replace("\\t", "\t"); // More convenient than inserting tabs manually.
                        client.write(line.as_bytes());
                    }
                    client.write(b".");
                }
                Err(_) => {
                    client.write(b"iFailed to read gophermenu for this directory.\r\n.");
                    client.write(&format!("1Return home\t/\t{}\t70\r\n.", ipaddr).as_bytes());
                }
            }
        } else {
            client.write(b"iNo content.\r\n");
            client.write(&format!("1Return home\t/\t{}\t70\r\n.", ipaddr).as_bytes());
        }
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents);
    Ok(contents)
}
