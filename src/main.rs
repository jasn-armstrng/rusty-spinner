// [Note] TCP: Transmission Control Protocol. Lower-level protocol that describes how data is transmitted over a network but not what the information is.
// [Note] HTTP: Hypertext Transfer Protocol. HTTP builds on top of TCP by defining the contents of the requests and responses. HTTP sends its data over TCP.
use std::{
    fs,
    io::{prelude::*, BufReader}, // Gives us access to traits and types that let us read from and write to the stream.
    net::{TcpListener, TcpStream},
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
        let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
            ("HTTP/1.1 200 OK", "hello.html")
        } else {
            ("HTTP/1.1 404 NOT FOUND", "404.html")
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
