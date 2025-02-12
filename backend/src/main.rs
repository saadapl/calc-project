use axum::{
    extract::Query,
    routing::get,
    Router,
    Json,
};
use rusqlite::{params, Connection}; // Removed unused `Result`
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
struct CalculationResult {
    addition: f64,
    subtraction: f64,
    multiplication: f64,
    division: String,
}

#[derive(Deserialize)]
struct CalculationParams {
    num1: f64,
    num2: f64,
}

async fn calculate(Query(params): Query<CalculationParams>) -> Json<CalculationResult> {
    let num1 = params.num1;
    let num2 = params.num2;

    // Perform calculations
    let add = num1 + num2;
    let sub = num1 - num2;
    let mul = num1 * num2;
    let div = if num2 != 0.0 {
        Some(num1 / num2)
    } else {
        None
    };

    // Save the calculation to the database
    let conn = Connection::open("calculations.db").unwrap();
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
    ).unwrap();
    conn.execute(
        "INSERT INTO calculations (num1, num2, addition, subtraction, multiplication, division) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![num1, num2, add, sub, mul, match div {
            Some(result) => result.to_string(),
            None => "undefined (division by zero)".to_string(),
        }],
    ).unwrap();

    // Return the result as JSON
    Json(CalculationResult {
        addition: add,
        subtraction: sub,
        multiplication: mul,
        division: match div {
            Some(result) => result.to_string(),
            None => "undefined (division by zero)".to_string(),
        },
    })
}

#[tokio::main]
async fn main() {
    // Build the backend application
    let app = Router::new().route("/calculate", get(calculate));

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Backend running on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}