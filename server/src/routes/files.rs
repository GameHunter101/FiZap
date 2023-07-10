use actix_web::{get, http::StatusCode, post, web, HttpResponse};
use serde::Deserialize;
use tokio::fs::{self, DirEntry};

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
    indices: Vec<u32>,
    count: u32,
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
                let mut indices_clone = body.indices.clone();

                let sorted_indices = indices_clone.as_mut_slice();
                sorted_indices.sort();
                let highest_index = *sorted_indices.last().unwrap();
                let mut files: Vec<DirEntry> = vec![];
                if highest_index == 0 {
                    files.push(entries.next_entry().await.unwrap().unwrap());
                } else {
                    for _ in 0..highest_index as usize {
                        files.push(entries.next_entry().await.unwrap().unwrap());
                    }
                }
                let filtered_files: Vec<(String, String)> = files
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| indices_clone.contains(&(*i as u32)))
                    .map(|(_, entry)| {
                        let entry_name = entry.file_name().to_string_lossy().to_string();
                        let entry_type = {
                            let path = entry.path();
                            if let Some(extension) = path.extension() {
                                extension.to_string_lossy().to_string()
                            } else {
                                String::new()
                            }
                        };
                        (entry_name, entry_type)
                    })
                    .collect();

                return Ok(HttpResponse::build(StatusCode::OK).json(filtered_files));
            }
            Err(_) => Err(CustomError::MissingPath),
        }
    } else {
        Err(CustomError::MissingPath)
    }
}
