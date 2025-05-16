// netwoek.rs
use native_tls::TlsConnector;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use url::Url;
// a little “trait alias” for anything that implements both Read and Writ
trait ReadWrite: Read + Write {}
impl<T: Read + Write> ReadWrite for T {}

#[derive(Debug)]
pub struct BrowserError(String);

impl fmt::Display for BrowserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for BrowserError {}

pub fn load_url(url_str: &str) -> Result<String, Box<dyn Error>> {
    println!("URL: {}", url_str);

    let parsed_url = Url::parse(url_str)?;

    let scheme = parsed_url.scheme().to_string();
    // only suport http & https
    if scheme != "http" && scheme != "https" {
        return Err(BrowserError(format!("Unsupported scheme: {}", scheme)).into());
    }

    let host = parsed_url
        .host_str()
        .ok_or(BrowserError("URL has no host".into()))? // Return error if host is missing
        .to_string();

    // Get port, using default if not specified
    let port = parsed_url
        .port_or_known_default()
        .unwrap_or_else(|| if scheme == "http" { 80 } else { 443 });

    // Get the path. Url::path() returns "/something" or "" for the root.
    // The url crate's path() is simpler and correct; "" maps to "/".
    let path = {
        let p = parsed_url.path();
        if p.is_empty() {
            "/".to_string()
        } else {
            p.to_string()
        }
    };

    // Connect to the host and port
    let address = format!("{}:{}", host, port);
    let tcp = TcpStream::connect(&address)?;
    // TcpStream implements Read and Write traits

    // now use our ReadWrite super-trait
    let mut stream: Box<dyn ReadWrite> = if scheme == "https" {
        // native-tls equivalent of ssl.create_default_context() and ctx.wrap_socket()
        let connector = TlsConnector::new()?;
        let tls_stream = connector.connect(&host, tcp)?;
        Box::new(tls_stream) // hold either TlsStream or TcpStream
    } else {
        Box::new(tcp)
    };

    // Construct the HTTP request string and convert to bytes
    // Use \r\n for newlines as required by HTTP
    let request = format!("GET {} HTTP/1.0\r\nHost: {}\r\n\r\n", path, host);
    // Rust strings are UTF-8 by default.
    stream.write_all(request.as_bytes())?; // Send the request bytes

    // Use BufReader for efficient line-by-line reading of the response
    let mut reader = BufReader::new(stream);

    // Read the status line
    let mut statusline = String::new();
    reader.read_line(&mut statusline)?; // Reads until \n, includes \r if present

    let parts: Vec<&str> = statusline.trim_end().splitn(3, ' ').collect();
    if parts.len() < 3 {
        return Err(BrowserError(format!("Malformed status line: {}", statusline.trim())).into());
    }
    let _version = parts[0]; // Python ignores these parts
    let _status = parts[1];
    let _explanation = parts[2];

    // Read headers into a HashMap
    let mut response_headers = HashMap::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?; // Reads a line

        // An empty header line is just "\r\n" or "\n".
        // Check if the line is just the newline characters.
        if line == "\r\n" || line == "\n" {
            break; // Found the blank line separating headers from body
        }

        // Python splits line.split(":", 1). Replicate this using splitn(2, ':').
        let header_parts: Vec<&str> = line.trim_end().splitn(2, ':').collect();
        if header_parts.len() != 2 {
            // Handle malformed header line. loop continues.
            eprintln!("Warning: Malformed header line: {}", line.trim_end());
            continue;
        }

        // to_lowercase() is sufficient for HTTP headers.
        let header_name = header_parts[0].trim().to_lowercase(); // Trim whitespace around name
        let header_value = header_parts[1].trim_start().to_string(); // Trim whitespace at start of value (per RFC)

        response_headers.insert(header_name, header_value);
    }

    assert!(
        !response_headers.contains_key("transfer-encoding"),
        "Unsupported Transfer-Encoding"
    );
    assert!(
        !response_headers.contains_key("content-encoding"),
        "Unsupported Content-Encoding"
    );

    // For HTTP/1.0, the server typically closes the connection after sending the body,
    // signaling the end of the body. BufReader.read_to_string reads until EOF.
    let mut body = String::new();
    reader.read_to_string(&mut body)?;

    // The socket is automatically closed when the `stream` variable goes out of scope
    // due to Rust's RAII (Resource Acquisition Is Initialization). No explicit `close()` needed.
    Ok(body) // Return the body as a String
}
