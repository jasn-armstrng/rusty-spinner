// [Note] TCP: Transmission Control Protocol. Lower-level protocol that describes how data is transmitted over a network but not what the information is.
// [Note] HTTP: Hypertext Transfer Protocol. HTTP builds on top of TCP by defining the contents of the requests and responses. HTTP sends its data over TCP.
use std::{
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

        // println!("Connection established!");
        handle_connection(stream);
    }

    fn handle_connection(mut stream: TcpStream) {
        // Reading from the TcpStream and printing the data.
        let buf_reader = BufReader::new(&mut stream);

        // Collect the data from the none-empty lines in the stream
        let http_request: Vec<_> = buf_reader
            .lines() // Access the lines
            .map(|result| result.unwrap()) // Unwrap the result
            .take_while(|line| !line.is_empty()) // Take lines until an empty line is encountered. This is an iterator adaptor (on lines) that takes elements from the iterator while the predicate is true.
            .collect();

        println!("Request: {:#?}", http_request);
    }
}
