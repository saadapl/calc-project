use eframe::egui;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize)]
struct CalculationResult {
    addition: f64,
    subtraction: f64,
    multiplication: f64,
    division: String,
}

struct CalculatorApp {
    num1: String,
    num2: String,
    result: Arc<Mutex<String>>, // Shared result
    client: Arc<Client>,
    runtime: Runtime, // Tokio runtime
}

impl CalculatorApp {
    fn new() -> Self {
        // Create a Tokio runtime
        let runtime = Runtime::new().unwrap();

        Self {
            num1: String::new(),
            num2: String::new(),
            result: Arc::new(Mutex::new(String::new())),
            client: Arc::new(Client::new()),
            runtime,
        }
    }

    fn calculate(&mut self) {
        let num1: f64 = self.num1.parse().unwrap_or(0.0);
        let num2: f64 = self.num2.parse().unwrap_or(0.0);

        let url = format!("http://127.0.0.1:8080/calculate?num1={}&num2={}", num1, num2);
        let client = Arc::clone(&self.client);
        let result = Arc::clone(&self.result);

        // Spawn the asynchronous task using the Tokio runtime
        self.runtime.spawn(async move {
            let response = client.get(&url).send().await;

            match response {
                Ok(resp) => {
                    if let Ok(result_data) = resp.json::<CalculationResult>().await {
                        let result_string = format!(
                            "Addition: {}\nSubtraction: {}\nMultiplication: {}\nDivision: {}",
                            result_data.addition, result_data.subtraction, result_data.multiplication, result_data.division
                        );
                        let mut result = result.lock().unwrap();
                        *result = result_string;
                    } else {
                        let mut result = result.lock().unwrap();
                        *result = "Error parsing response".to_string();
                    }
                }
                Err(_) => {
                    let mut result = result.lock().unwrap();
                    *result = "Error connecting to backend".to_string();
                }
            }
        });
    }
}

impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Calculator");

            // Input fields for numbers
            ui.horizontal(|ui| {
                ui.label("Number 1:");
                ui.text_edit_singleline(&mut self.num1);
            });
            ui.horizontal(|ui| {
                ui.label("Number 2:");
                ui.text_edit_singleline(&mut self.num2);
            });

            // Calculate button
            if ui.button("Calculate").clicked() {
                self.calculate();
            }

            // Display result
            ui.label("Result:");
            let result = self.result.lock().unwrap();
            ui.monospace(&*result);
        });
    }
}

fn main() {
    let app = CalculatorApp::new();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 300.0)),
        ..Default::default()
    };

    // Handle the Result returned by `eframe::run_native`
    let _ = eframe::run_native(
        "Calculator App",
        options,
        Box::new(|_cc| Box::new(app)),
    );
}