use rusqlite::{params, Connection, Result};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::str;

fn main() -> Result<()> {
    // Connect to SQLite database (or create it if it doesn't exist)
    let conn = Connection::open("calculations.db")?;

    // Create a table to store calculation results if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS calculations (
            id INTEGER PRIMARY KEY,
            num1 REAL NOT NULL,
            num2 REAL NOT NULL,
            addition REAL NOT NULL,
            subtraction REAL NOT NULL,
            multiplication REAL NOT NULL,
            division TEXT
        )",
        params![],
    )?;

    // Bind the TCP listener to the address and port
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server running on http://127.0.0.1:8080");

    // Loop over incoming TCP connections
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        // Buffer to read data from the stream
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        // Convert buffer to string to interpret the HTTP request
        let request = String::from_utf8_lossy(&buffer[..]);
        // Check if the request is a GET request to the /calculate endpoint
        if request.starts_with("GET /calculate") {
            // Parse query parameters from the URL
            let (num1, num2) = parse_query_params(&request);
            // Perform calculations
            let add = num1 + num2;
            let sub = num1 - num2;
            let mul = num1 * num2;
            let div = if num2 != 0.0 { Some(num1 / num2) } else { None };
            // Save the calculation to the database
            conn.execute(
                "INSERT INTO calculations (num1, num2, addition, subtraction, multiplication, division) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![num1, num2, add, sub, mul, match div {
                    Some(result) => result.to_string(),
                    None => "undefined (division by zero)".to_string(),
                }],
            )?;
            // Build the response
            let response_body = format!(
                "{{ \"addition\": {}, \"subtraction\": {}, \"multiplication\": {}, \"division\": {} }}",
                add,
                sub,
                mul,
                match div {
                    Some(result) => result.to_string(),
                    None => "undefined (division by zero)".to_string(),
                }
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\n\
                Content-Type: application/json\r\n\
                Access-Control-Allow-Origin: *\r\n\
                \r\n\
                {}",
                response_body
            );
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        } else if request.starts_with("GET /history") {
            // Retrieve calculation history from the database
            let mut stmt = conn.prepare("SELECT num1, num2, addition, subtraction, multiplication, division FROM calculations")?;
            let calculations = stmt.query_map(params![], |row| {
                Ok((
                    row.get::<_, f64>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })?;
            let mut history = Vec::new();
            for calc in calculations {
                let (num1, num2, add, sub, mul, div) = calc?;
                history.push(format!(
                    "{{ \"num1\": {}, \"num2\": {}, \"addition\": {}, \"subtraction\": {}, \"multiplication\": {}, \"division\": \"{}\" }}",
                    num1, num2, add, sub, mul, div
                ));
            }
            let response_body = format!("[{}]", history.join(","));
            let response = format!(
                "HTTP/1.1 200 OK\r\n\
                Content-Type: application/json\r\n\
                Access-Control-Allow-Origin: *\r\n\
                \r\n\
                {}",
                response_body
            );
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        } else {
            // Handle other requests
            let response = "HTTP/1.1 404 NOT FOUND\r\n\
                           Content-Type: text/plain\r\n\
                           Access-Control-Allow-Origin: *\r\n\
                           \r\n\
                           Not Found";
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
    Ok(())
}

// Helper function to parse query parameters from the request
fn parse_query_params(request: &str) -> (f64, f64) {
    let query_string = request.split_whitespace().nth(1).unwrap_or("");
    let query_string = query_string.split('?').nth(1).unwrap_or("");
    let mut num1 = 0.0;
    let mut num2 = 0.0;
    for param in query_string.split('&') {
        let mut key_value = param.split('=');
        let key = key_value.next().unwrap_or("");
        let value = key_value.next().unwrap_or("");
        match key {
            "num1" => num1 = value.parse().unwrap_or(0.0),
            "num2" => num2 = value.parse().unwrap_or(0.0),
            _ => (),
        }
    }
    (num1, num2)
}