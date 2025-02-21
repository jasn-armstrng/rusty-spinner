// [Note] TCP: Transmission Control Protocol. Lower-level protocol that describes how data is transmitted over a network but not what the information is.
// [Note] HTTP: Hypertext Transfer Protocol. HTTP builds on top of TCP by defining the contents of the requests and responses. HTTP sends its data over TCP.
//
// [Motivation] With a single-threaded server, we can only handle one request at a time. If a request takes a long time to process, it will block all other requests.
// To handle multiple requests at the same time, we need to use a multi-threaded server.
use std::{
    fs,
    io::{prelude::*, BufReader}, // Gives us access to traits and types that let us read from and write to the stream.
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn main() {
    // Create a TCP listener on the specified address and port.
    // [Note] bind = connect. So we're connecting our listener to local host at port 7878.
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap(); // bind returns a Result<T, E>. unwrap here will panic if an error occurs.

    // [Note] We're using a for loop to accept incoming connections. The for loop will iterate over the incoming connections and handle each one.
    // [Note] stream is is a Result<TcpStream, Error> type.
    // [Note] The below will stay open until the program is terminated.
    for stream in listener.incoming() {
        // [Note] Incoming returns an iterator that gives us a sequence of streams -  here eah stream (TcpStream) is a connection attempt.
        // [Note] You can test a connection using netcat from the CLI. nc -vz localhost 7878
        // [Note] A single stream represents an open connection between the client and the server.
        // [Note] A connection is the name for the full duplex (request/response) communication channel between the client and the server.
        // [Note] A TCP stream is readable and writable.
        // [Note] The TcpStream (which is your HANDLE to the TCP connection) is created when a client connects and is yielded by the listener.incoming() iterator.
        // [Note] a "handle" is an abstraction that represents an underlying resource, in this case, a TCP connection.
        //        It's a way for the program to interact with that resource without needing to know all the low-level details of how it's managed
        let stream = stream.unwrap();

        // Utilize the stream handle for communication
        handle_connection(stream);
    }

    fn handle_connection(mut stream: TcpStream) {
        // Read from the TcpStream
        let buf_reader = BufReader::new(&mut stream);
        let request_line = buf_reader
            .lines()
            .next() // Returns an Option<Result<String>>
            .unwrap() // unwrap the Option. [Note] In production, we would handle errors gracefully.
            .unwrap(); // unwrap the Result

        // If the user requests the root page, respond with a 200 OK status and the contents of hello.html.
        let (status_line, filename) = match &request_line[..] {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
            "GET /sleep HTTP/1.1" => {
                // Simulating a slow response
                // Server sleep for 5 seconds before rendering the successful HTML page.
                // As we are still single-threaded, any simultaneous requests will be queued and processed sequentially.
                thread::sleep(Duration::from_secs(5));
                ("HTTP/1.1 200 OK", "hello.html")
            }
            _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
        };

        let contents = fs::read_to_string(filename).unwrap();
        let length = contents.len(); // [Note] len returns number of bytes in the string.

        let log = format!(
            "{status_line}\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {length}"
        );

        let response = format!("{status_line}\r\n\r\n{contents}");

        println!("{log}"); // "console.log"
        stream.write_all(response.as_bytes()).unwrap(); // Write the response to the stream.
    }
}

// _52766_276787664

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::blocking::Client;
    use std::{thread, time::Duration, time::Instant};

    /// Helper function to start the server in a background thread.
    fn start_server() {
        thread::spawn(|| {
            main(); // or start_server() if you have a dedicated function
        });
        // Give the server a moment to bind (helps avoid connection errors).
        thread::sleep(Duration::from_millis(500));
    }

    #[test]
    fn test_ok_response() {
        start_server();
        let client = Client::new();

        // Send a GET request to root ("/")
        let resp = client.get("http://127.0.0.1:7878/").send().unwrap();
        assert_eq!(resp.status(), 200, "Expected HTTP 200 for /");

        // Verify the body contains the expected content
        let contents = fs::read_to_string("hello.html").unwrap();
        let body = resp.text().unwrap();
        assert!(
            body.contains(&contents),
            "Body did not contain expected 'Hello, world!' text"
        );
    }

    #[test]
    fn test_404_response() {
        start_server();
        let client = Client::new();

        // Request a non-existent path
        let resp = client.get("http://127.0.0.1:7878/notfound").send().unwrap();
        assert_eq!(resp.status(), 404, "Expected HTTP 404 for /notfound");

        // Verify the body contains the expected content
        let contents = fs::read_to_string("404.html").unwrap();
        let body = resp.text().unwrap();
        assert!(
            body.contains(&contents),
            "Body did not contain expected '404' text"
        );
    }

    #[test]
    fn test_sleep_response() {
        start_server();
        let client = Client::new();

        // Measure how long the request takes
        let start = Instant::now();
        let resp = client.get("http://127.0.0.1:7878/sleep").send().unwrap();
        let duration = start.elapsed();

        assert_eq!(resp.status(), 200, "Expected HTTP 200 for /sleep");
        // The server sleeps for 5s, ensure it took at least that long
        assert!(
            duration.as_secs() >= 5,
            "Expected at least ~5s delay, got {:?} instead",
            duration
        );
    }

    #[test]
    fn test_multiple_connections() {
        start_server();
        let client = Client::new();

        // URIs to request in parallel
        let uris = vec![
            "http://127.0.0.1:7878/",
            "http://127.0.0.1:7878/sleep",
            "http://127.0.0.1:7878/notfound",
        ];

        // Spawn threads to simulate multiple concurrent connections
        let mut handles = Vec::new();
        for uri in uris {
            let c = client.clone();
            let u = uri;
            let handle = std::thread::spawn(move || {
                // Each thread performs a GET request
                c.get(u).send()
            });
            handles.push((u, handle));
        }

        // Collect all responses and assert on status codes
        for (uri, handle) in handles {
            let resp = handle.join().unwrap().unwrap();
            match uri {
                "http://127.0.0.1:7878/" => {
                    assert_eq!(resp.status(), 200, "Expected 200 for /");
                }
                "http://127.0.0.1:7878/sleep" => {
                    assert_eq!(resp.status(), 200, "Expected 200 for /sleep");
                }
                "http://127.0.0.1:7878/notfound" => {
                    assert_eq!(resp.status(), 404, "Expected 404 for /notfound");
                }
                _ => unreachable!("Unexpected URI"),
            }
        }
    }
}
