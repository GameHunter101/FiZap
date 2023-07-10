use std::{collections::HashMap, rc::Rc};

use gloo_console::log;
use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yewdux::prelude::*;

use crate::utils::send_post_request;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ItemData {
    pub index: u32,
    pub name: String,
}

#[derive(Debug, Default, PartialEq, Store, Serialize, Deserialize, Clone)]
#[store(storage = "local", storage_tab_sync)]
pub struct Store {
    pub loaded_items: Vec<ItemData>,
    pub row_size: u32,
    pub total_items: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GetFiles {
    indices: Vec<u32>,
    count: u32,
}

#[allow(unused_must_use)]
pub async fn load_item(id: u32, dispatch: Dispatch<Store>) -> Result<(String, String), ()> {
    let mut temp_store = Rc::new(Store::default());
    dispatch.reduce_mut(|store| {
        if store.loaded_items.iter().position(|x| x.index == id) == None {
            log!(format!("Id: {}", id));
            store.loaded_items.push(ItemData {
                index: id,
                name: String::new(),
            });
        }
        temp_store = Rc::new(store.clone());
    });
    let temp_store = temp_store.clone();

    let count = temp_store.loaded_items.len() as u32;
    log!(format!("total items: {:?}", temp_store.loaded_items));
    if count % temp_store.row_size == 0 || count < temp_store.row_size {
        let mut requested_files: Vec<ItemData> = temp_store
            .loaded_items
            .iter()
            .map(|item| item.clone())
            .filter(|item| item.name.len() == 0)
            .collect();
        requested_files.reverse();
        let mut returned_files: Vec<u32> = requested_files
            [..temp_store.row_size.min(count) as usize]
            .iter()
            .map(|item| item.index)
            .collect();
        returned_files.reverse();

        let request_body = GetFiles {
            indices: returned_files,
            count: temp_store.total_items,
        };

        let res = serde_json::from_str::<Vec<(String, String)>>(
            &send_post_request("/api/indices", &request_body)
                .await
                .unwrap(),
        )
        .unwrap();

        let mut sorted_indices = request_body.indices;
        sorted_indices.sort();
        let index = sorted_indices.iter().position(|&i| i == id).unwrap();
        let item_data = res[index].clone();

        return Ok(item_data);
    }
    Err(())
}

pub fn set_total_count(count: u32, dispatch: Dispatch<Store>) {
    dispatch.reduce_mut(move |store| {
        store.total_items = count;
    });
}

pub fn set_row_size(size: u32, dispatch: Dispatch<Store>) {
    dispatch.reduce_mut(move |store| {
        store.row_size = size;
    });
}
