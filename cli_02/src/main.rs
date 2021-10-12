use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

// https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
// https://www.adityathebe.com/raw-http-request-with-sockets-nodejs
// https://stackoverflow.com/questions/15772355/how-to-send-an-http-request-using-telnet#15772575
fn main() {
    let mut stream = TcpStream::connect("localhost:3000").unwrap();

    let mut http_req = String::new();
    http_req.push_str("GET / HTTP/1.1");
    http_req.push_str("\r\n");
    http_req.push_str("Host: localhost");
    http_req.push_str("\r\n");
    http_req.push_str("Connection: close");
    http_req.push_str("\r\n");
    http_req.push_str("\r\n");

    println!("http_req = {:?}", http_req);

    let req_bytes: &[u8] = http_req.as_bytes();
    let req_feedback = stream.write_all(req_bytes);
    println!("is req_feedback OK? {}", req_feedback.is_ok());

    let mut http_res = String::new();
    let res_feedback = stream.read_to_string(&mut http_res);
    println!("is res_feedback OK? {}", res_feedback.is_ok());
    println!("http_res = \n-----\n{}\n-----\n", http_res);

    println!("Done with handmade HTTP over TCP!");
}
