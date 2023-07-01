// use gloo_net::http::Request;
use reqwasm::http::Request;
use wasm_bindgen::JsValue;
use yew::prelude::*;

use crate::{components::file::*, pages::dashboard::SearchContext};

pub async fn send_request(url: &str, body: Option<JsValue>) -> Result<String, String> {
    let token = "some token";
    let auth_header = format!("Bearer {}", token);
    let request = match body {
        Some(body) => {
            Request::post(url)
                .header("Content-Type", "application/json")
                .header("Authorization", &auth_header)
                .body(body)
                .send()
                .await
        }
        None => {
            Request::get(url)
                .header("Content-Type", "application/json")
                .header("Authorization", &auth_header)
                .send()
                .await
        }
    };
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

#[function_component(FileManager)]
pub fn file_manager() -> Html {
    let search_ctx = use_context::<SearchContext>().unwrap();
    let query = search_ctx.query.to_owned();
    let item_count = use_state(|| 0);
    let count_state = item_count.clone();

    use_effect(move || {
        wasm_bindgen_futures::spawn_local(async move {
            let response = send_request("/api/count", None).await;
            let count = response.unwrap().parse::<i32>().unwrap();
            count_state.set(count);
        });
        || ()
    });
    let file_names = (0..*item_count)
        .map(|i| i.to_string())
        .filter(|name| name.contains(&query))
        .collect::<Vec<String>>();
    let files = file_names
        .iter()
        .map(|i| html! {<File name={i.clone()} />})
        .collect::<Vec<Html>>();
    html! {
        <div class={"file-display"}>
            if *item_count > 0 {
                {files}
            } else {
                <div class={"col-span-full flex justify-center items-center"}>
                    {"No files found..."}
                </div>
            }
        </div>
    }
}
