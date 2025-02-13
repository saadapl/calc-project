use rusqlite::{params, Connection, Result};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::str;

fn main() -> Result<()> {
    let conn = Connection::open("calculations.db")?;
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

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server running on http://127.0.0.1:8080");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..]);

        if request.starts_with("GET /calculate") {
            let (num1, num2) = parse_query_params(&request);
            let add = num1 + num2;
            let sub = num1 - num2;
            let mul = num1 * num2;
            let div = if num2 != 0.0 { Some(num1 / num2) } else { None };

            conn.execute(
                "INSERT INTO calculations (num1, num2, addition, subtraction, multiplication, division) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![num1, num2, add, sub, mul, match div {
                    Some(result) => result.to_string(),
                    None => "undefined (division by zero)".to_string(),
                }],
            )?;

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
        }
    }
    Ok(())
}

fn parse_query_params(request: &str) -> (f64, f64) {
    let query_string = request.split_whitespace().nth(1).unwrap_or("").split('?').nth(1).unwrap_or("");
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