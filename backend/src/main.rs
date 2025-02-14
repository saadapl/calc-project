use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
struct Calculation {
    num1: f64,
    num2: f64,
    addition: f64,
    subtraction: f64,
    multiplication: f64,
    division: String,
}

#[derive(Deserialize)]
struct QueryParams {
    num1: f64,
    num2: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
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

    // Build our application with a single route
    let app = Router::new()
        .route("/calculate", get(calculate))
        .route("/history", get(get_history));

    // Run our app
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Backend running on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn calculate(Query(params): Query<QueryParams>) -> Result<Json<Calculation>, StatusCode> {
    let num1 = params.num1;
    let num2 = params.num2;

    let add = num1 + num2;
    let sub = num1 - num2;
    let mul = num1 * num2;
    let div = if num2 != 0.0 {
        Some(num1 / num2)
    } else {
        None
    };

    let conn = Connection::open("calculations.db").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    conn.execute(
        "INSERT INTO calculations (num1, num2, addition, subtraction, multiplication, division) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![num1, num2, add, sub, mul, match div {
            Some(result) => result.to_string(),
            None => "undefined (division by zero)".to_string(),
        }],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Calculation {
        num1,
        num2,
        addition: add,
        subtraction: sub,
        multiplication: mul,
        division: match div {
            Some(result) => result.to_string(),
            None => "undefined (division by zero)".to_string(),
        },
    }))
}

async fn get_history() -> Result<Json<Vec<Calculation>>, StatusCode> {
    let conn = Connection::open("calculations.db").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut stmt = conn
        .prepare("SELECT num1, num2, addition, subtraction, multiplication, division FROM calculations")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let calculations = stmt
        .query_map(params![], |row| {
            Ok(Calculation {
                num1: row.get(0)?,
                num2: row.get(1)?,
                addition: row.get(2)?,
                subtraction: row.get(3)?,
                multiplication: row.get(4)?,
                division: row.get(5)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut history = Vec::new();
    for calc in calculations {
        history.push(calc.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    }

    Ok(Json(history))
}