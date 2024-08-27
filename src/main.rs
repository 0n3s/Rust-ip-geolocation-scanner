mod core;
mod cloud_ip_checker; // Add this line to import the cloud_ip_checker module

use std::sync::Arc;
use std::path::PathBuf;

use axum::{
    routing::post,
    Router,
    Json,
    extract::State,
};
use tower_http::cors::CorsLayer;
use reqwest::Client;

// Import the types and functions from the core module
use crate::core::{IpInput, ApiResponse, ApiMetricsResponse, process_ips, write_to_file, get_default_output_path};
use crate::cloud_ip_checker::CloudIpChecker; // Add this line to import CloudIpChecker

#[derive(Clone)]
struct AppState {
    client: Client,
    cloud_checker: CloudIpChecker, // Add this field
}

async fn handle_web_request(
    State(state): State<Arc<AppState>>,
    Json(input): Json<IpInput>,
) -> Json<ApiResponse> {
    let ips: Vec<String> = input.ips.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let total_ips = ips.len();

    // Pass the cloud_checker to process_ips
    let (results, metrics) = process_ips(&state.client, &ips, &state.cloud_checker).await;

    let output_path = if input.use_default_output {
        get_default_output_path()
    } else {
        PathBuf::from("custom_output.csv")
    };

    let message = match write_to_file(&results, &output_path) {
        Ok(_) => format!("Results have been written to {}", output_path.display()),
        Err(e) => format!("Error writing to file: {}", e),
    };

    let total_requests = metrics.success_count + metrics.failure_count;
    let success_rate = if total_requests > 0 {
        (metrics.success_count as f64 / total_requests as f64) * 100.0
    } else {
        0.0
    };
    let average_response_time = if total_requests > 0 {
        metrics.total_time.as_secs_f64() * 1000.0 / total_requests as f64
    } else {
        0.0
    };

    Json(ApiResponse { 
        message,
        metrics: ApiMetricsResponse {
            total_requests,
            success_rate,
            average_response_time,
        },
        results,
        total_ips,
    })
}

#[tokio::main]
async fn main() {
    let client = Client::new();
    let cloud_checker = CloudIpChecker::new(); // Initialize CloudIpChecker
    let state = Arc::new(AppState { client, cloud_checker });

    let app = Router::new()
        .route("/api/process-ips", post(handle_web_request))
        .layer(CorsLayer::permissive())
        .with_state(state);

    println!("Server running on http://localhost:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}