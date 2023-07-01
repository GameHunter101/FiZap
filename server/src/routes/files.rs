use actix_web::{get, http::StatusCode, web, HttpResponse};
use tokio::fs;

use crate::{middleware::AuthenticationExtractor, utils::CustomError};

#[get("/count")]
pub async fn get_file_count(auth: AuthenticationExtractor) -> Result<HttpResponse, CustomError> {
    let id = auth.clone();
    let dir = fs::read_dir(format!("./files/{}", id)).await;
    return match dir {
        Ok(mut entries) => {
            let mut count = 0;
            while let Some(entry) = entries.next_entry().await.unwrap() {
                count += 1;
            }

            return Ok(HttpResponse::build(StatusCode::OK).body(count.to_string()));
        }
        Err(_) => Err(CustomError::MissingPath),
    };
}
