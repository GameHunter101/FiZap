use std::fmt::Debug;

use gloo_console::log;
use reqwasm::http::Request;
use serde::Serialize;

pub async fn send_post_request<T: Serialize + Debug>(url: &str, body: &T) -> Result<String, String> {
    let token = "";
    let auth_header = format!("Bearer {}", token);
    let request = Request::post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", &auth_header)
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await;
    let response = match request {
        Ok(res) => res,
        Err(e) => return Err(format!("Failed to make request, {}", e)),
    };

    let res_json = response.text().await;
    match res_json {
        Ok(data) => Ok(data),
        Err(_) => Err("Failed to parse response".to_string()),
    }
}

pub async fn send_get_request(url: &str) -> Result<String, String> {
    let token = "";
    let auth_header = format!("Bearer {}", token);
    let request = Request::get(url)
        .header("Content-Type", "application/json")
        .header("Authorization", &auth_header)
        .send()
        .await;
    let response = match request {
        Ok(res) => res,
        Err(e) => return Err(format!("Failed to make request, {}", e)),
    };

    let res_json = response.text().await;
    match res_json {
        Ok(data) => Ok(data),
        Err(_) => Err("Failed to parse response".to_string()),
    }
}
