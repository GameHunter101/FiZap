use actix_web::{get, http::StatusCode, post, web, HttpResponse};
use serde::Deserialize;
use tokio::fs;

use crate::{middleware::AuthenticationExtractor, utils::CustomError};

#[get("/count")]
pub async fn get_file_count(auth: AuthenticationExtractor) -> Result<HttpResponse, CustomError> {
    let id = auth.clone();
    let dir = fs::read_dir(format!("./files/{}", id)).await;
    return match dir {
        Ok(mut entries) => {
            let mut count = 0;
            while let Some(_entry) = entries.next_entry().await.unwrap() {
                count += 1;
            }

            return Ok(HttpResponse::build(StatusCode::OK).body(count.to_string()));
        }
        Err(_) => Err(CustomError::MissingPath),
    };
}

#[derive(Debug, Deserialize)]
pub struct GetFiles {
    indices: Vec<i32>,
    count: i32,
}

#[post("/indices")]
pub async fn get_files_indices(
    body: web::Json<GetFiles>,
    auth: AuthenticationExtractor,
) -> Result<HttpResponse, CustomError> {
    let id = auth.clone();
    let dir = fs::read_dir(format!("./files/{}", id)).await;
    if body.count > 0 {
        match dir {
            Ok(mut entries) => {
                let mut prev_index = 0;
                let mut files: Vec<(String, String)> = Vec::with_capacity(body.indices.len());
                for file_index in body.indices.iter() {
                    for _ in prev_index..*file_index {
                        let next_entry = entries.next_entry().await.unwrap().unwrap();
                        let file_name = next_entry.file_name().to_string_lossy().to_string();
                        let file_type = {
                            let file_path = next_entry.path();
                            if let Some(extension) = file_path.extension() {
                                extension.to_string_lossy().to_string()
                            } else {
                                String::new()
                            }
                        };

                        files[(*file_index) as usize] = (file_name, file_type);
                    }
                    prev_index = *file_index;
                }
                return Ok(HttpResponse::build(StatusCode::OK).json(files));
            }
            Err(_) => Err(CustomError::MissingPath),
        }
    } else {
        Err(CustomError::MissingPath)
    }
}
