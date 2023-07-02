use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use yewdux::prelude::*;

#[derive(Debug, Default, PartialEq, Store, Serialize, Deserialize, Clone)]
#[store(storage = "local", storage_tab_sync)]
pub struct Store {
    pub loaded_files: HashMap<i32, String>,
}

pub fn display_box(id: i32, dispatch: Dispatch<Store>) {
    dispatch.reduce_mut(move |store| {
        if store.loaded_files.get(&id) == None {
            store.loaded_files.insert(id, String::new());
        }
    })
}