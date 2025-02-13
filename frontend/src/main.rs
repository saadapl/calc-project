use eframe::egui;
use reqwest;
use serde_json::Value;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("Calculator", options, Box::new(|_cc| Ok(Box::new(App::default()))))?;
    Ok(())
}

struct App {
    num1: String,
    num2: String,
    result: Arc<Mutex<String>>,  // ✅ Store result inside Arc<Mutex<String>>
}

impl Default for App {
    fn default() -> Self {
        Self {
            num1: "".to_owned(),
            num2: "".to_owned(),
            result: Arc::new(Mutex::new("".to_owned())),  // ✅ Initialize Mutex
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rust Web Calculator");

            ui.horizontal(|ui| {
                ui.label("Number 1:");
                ui.text_edit_singleline(&mut self.num1);
            });

            ui.horizontal(|ui| {
                ui.label("Number 2:");
                ui.text_edit_singleline(&mut self.num2);
            });

            if ui.button("Calculate").clicked() {
                let num1 = self.num1.clone();
                let num2 = self.num2.clone();
                let result_ref = Arc::clone(&self.result); // ✅ Clone Arc to use in async

                let ctx = ctx.clone();

                tokio::spawn(async move {
                    let result = get_calculation(&num1, &num2).await;

                    // ✅ Lock mutex and store result safely
                    if let Ok(mut result_lock) = result_ref.lock() {
                        *result_lock = result;
                    }

                    ctx.request_repaint(); // ✅ Refresh UI
                });
            }

            // ✅ Lock the mutex before reading `result`
            ui.label(format!("Result: {}", self.result.lock().unwrap()));
        });
    }
}

async fn get_calculation(num1: &str, num2: &str) -> String {
    let url = format!("http://127.0.0.1:8080/calculate?num1={}&num2={}", num1, num2);

    match reqwest::get(&url).await {
        Ok(response) => match response.json::<Value>().await {
            Ok(json) => json.to_string(),
            Err(_) => "Failed to parse response".to_string(),
        },
        Err(_) => "Request failed".to_string(),
    }
}