use axum::{
    extract::Query,
    response::Html,
    routing::get,
    Router,
    Json
};
use reqwest::Client;
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
async fn main() {
    // Build our application with routes
    let app = Router::new()
        .route("/", get(index))
        .route("/calculate", get(calculate));

    // Run our app
    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    println!("Frontend running on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Serve a simple HTML page with sliders and results
async fn index() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Calculator</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    margin: 20px;
                }
                .slider-container {
                    margin-bottom: 20px;
                    position: relative;
                }
                .results {
                    margin-top: 20px;
                }
                input[type="range"] {
                    width: 200px;
                }
                #num1 {
                    width: 500px; /* Increase width */
                    position: relative;
                    top: 6.5px; /* Move down the y-axis */
                }
                #num2 {
                    width: 500px; /* Increase width */
                    position: relative;
                    top: 6.5px; /* Move down the y-axis */
                }
            </style>
        </head>
        <body>
            <h1>Calculator</h1>
            <div class="slider-container">
                <label for="num1">Number 1:</label>
                <input type="range" id="num1" name="num1" min="0" max="1000" value="100">
                <span id="num1-value">100</span> <!-- Updated to match slider default value -->
            </div>
            <div class="slider-container">
                <label for="num2">Number 2:</label>
                <input type="range" id="num2" name="num2" min="0" max="1000" value="100">
                <span id="num2-value">100</span> <!-- Updated to match slider default value -->
            </div>
            <div class="results">
                <h2>Results</h2>
                <p>Addition: <span id="addition">200</span></p> <!-- Updated to match default calculation -->
                <p>Subtraction: <span id="subtraction">0</span></p>
                <p>Multiplication: <span id="multiplication">10000</span></p> <!-- Updated to match default calculation -->
                <p>Division: <span id="division">1</span></p>
            </div>
            <script>
                // Function to update results
                async function updateResults() {
                    const num1 = document.getElementById('num1').value;
                    const num2 = document.getElementById('num2').value;

                    const response = await fetch(`/calculate?num1=${num1}&num2=${num2}`);
                    const data = await response.json();

                    document.getElementById('addition').textContent = data.addition;
                    document.getElementById('subtraction').textContent = data.subtraction;
                    document.getElementById('multiplication').textContent = data.multiplication;
                    document.getElementById('division').textContent = data.division;
                }

                // Function to update displayed slider values
                function updateSliderValue(sliderId, valueId) {
                    const slider = document.getElementById(sliderId);
                    const value = document.getElementById(valueId);

                    // Update displayed value as slider is moved
                    slider.addEventListener('input', () => {
                        value.textContent = slider.value;
                    });

                    // Update results when slider is released
                    slider.addEventListener('change', updateResults);
                }

                // Initialize slider value updates
                updateSliderValue('num1', 'num1-value');
                updateSliderValue('num2', 'num2-value');

                // Initialize results on page load
                document.addEventListener('DOMContentLoaded', () => {
                    // Set initial values for sliders and displayed text
                    document.getElementById('num1-value').textContent = document.getElementById('num1').value;
                    document.getElementById('num2-value').textContent = document.getElementById('num2').value;

                    // Update results with default values
                    updateResults();
                });
            </script>
        </body>
        </html>
        "#,
    )
}

// Handle calculation requests
async fn calculate(Query(params): Query<QueryParams>) -> Json<Calculation> {
    let client = Client::new();
    let url = format!("http://127.0.0.1:8080/calculate?num1={}&num2={}", params.num1, params.num2);
    let response = client.get(&url).send().await.unwrap();
    let calculation: Calculation = response.json().await.unwrap();

    Json(calculation)
}