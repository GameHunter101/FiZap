use yew::prelude::*;

use crate::{components::file::*, pages::dashboard::SearchContext};

#[function_component(FileManager)]
pub fn file_manager() -> Html {
    let search_ctx = use_context::<SearchContext>().unwrap();
    let query = search_ctx.query.to_owned();

    let file_names = (0..50).map(|i| i.to_string()).filter(|name| name.contains(&query)).collect::<Vec<String>>();
    let files = file_names.iter()
        .map(|i| html! {<File name={i.clone()} />})
        .collect::<Vec<Html>>();
    html! {
        <div class={"file-display"}>
            {files}
        </div>
    }
}
