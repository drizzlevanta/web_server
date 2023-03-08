use std::{
    fs, io,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use web_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878");

    match listener {
        Ok(listener) => loop_conn(listener),
        Err(e) => {
            println!("Error creating listener: {}", e);
        }
    }
}

fn loop_conn(listener: TcpListener) {
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();
        pool.execute(|| {
            let _ = handle_conn(stream);
            // thread::spawn(|| {
            // if let Ok(s) = stream {
            //     println!("Connection established!");
            //     // let _ = handle_conn(s);

            //     if let Err(e) = handle_conn(s) {
            //         println!("Error occurred handling connection: {e}");
            //     }
        });
    }

    println!("Shutting down.");
}

// fn return_html(stream: &mut TcpStream, path: &str) -> Result<(), io::Error> {
//     let status_line = "HTTP/1.1 200 OK";

//     let contents = fs::read_to_string(path)?;

//     let length = contents.len();

//     let response = format!("{status_line}\r\nContent-length:{length}\r\n\r\n{contents}");
//     Ok(stream.write_all(response.as_bytes())?)
// }

fn handle_conn(mut stream: TcpStream) -> Result<(), io::Error> {
    let buf_reader = BufReader::new(&mut stream);

    let requests: Vec<_> = buf_reader
        .lines()
        .map(|r| r.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // println!("Requests: {:?} \n EOR", requests);

    let (status_line, file_path) = match requests.get(0) {
        Some(first_line) if first_line == "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        Some(first_line) if first_line == "GET /sleep HTTP/1.1" => {
            println!("sleeping...");
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
        // None => Err(Error::new(ErrorKind::InvalidInput, "Invalid requests")),
    };

    let contents = fs::read_to_string(file_path)?;

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-length:{length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}
