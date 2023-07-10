use std::{any::Any, rc::Rc};

use gloo::events::EventListener;
use gloo_console::log;
// use gloo_net::http::Request;
use reqwasm::http::Request;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{window, HtmlDivElement, Window};
use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::{
    components::file::*,
    pages::dashboard::SearchContext,
    store::{set_row_size, set_total_count, Store},
    utils::send_get_request,
};

#[function_component(FileManager)]
pub fn file_manager() -> Html {
    let (_, dispatch) = use_store::<Store>();
    let search_ctx = use_context::<SearchContext>().unwrap();
    let query = search_ctx.query.to_owned();
    let item_count = use_state(|| 0);
    let count_state = item_count.clone();

    let dispatch_clone = dispatch.clone();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let response = send_get_request("/api/count").await;
                let count = response.unwrap().parse::<u32>().unwrap();
                set_total_count(count, dispatch_clone);
                count_state.set(count);
            });
            || ()
        },
        (),
    );
    let file_names = (0..*item_count)
        .map(
            |i| i.to_string(), /* match state.loaded_files.get(&i) {
                                   Some(name) => name.to_string(),
                                   None => String::new(),
                               } */
        )
        .filter(|name| {
            if name.len() > 0 {
                name.contains(&query)
            } else {
                false
            }
        })
        .collect::<Vec<String>>();
    let files = file_names
        .iter()
        .map(|i| html! {<File name={i.clone()} />})
        .collect::<Vec<Html>>();

    let div_ref = use_node_ref();
    use_effect_with_deps(
        {
            let div = div_ref.clone();
            move |_| {
                let div = div.cast::<HtmlDivElement>().expect("Div not set");
                let div_clone = div.clone();

                let get_columns = move || {
                    let window = web_sys::window().expect("Failed to get window");
                    let style = window
                        .get_computed_style(&div_clone)
                        .expect("Failed to get computed style")
                        .unwrap();
                    let grid_columns_attribute =
                        style.get_property_value("grid-template-columns").unwrap();
                    grid_columns_attribute.split(" ").count() as u32
                };

                let cols = get_columns();
                set_row_size(cols, dispatch.clone());

                let mut _resize_listener = None;

                let onresize = Callback::from(move |_: Event| {
                    let cols = get_columns();
                    set_row_size(cols, dispatch.clone());
                });

                let listener =
                    Closure::<dyn Fn(Event)>::wrap(Box::new(move |e: Event| onresize.emit(e)));

                window()
                    .unwrap()
                    .add_event_listener_with_callback("resize", listener.as_ref().unchecked_ref())
                    .unwrap();
                _resize_listener = Some(listener);

                move || drop(_resize_listener)

                // let mut resize_listener = None;

                // let onresize = Callback::from(move |_: Event| {
                //     let cols = get_columns();
                //     log!(format!("Cols: {}", cols));
                //     col_state.set(cols);
                // });

                // let listener =
                //     EventListener::new(&div, "resize", move |e| onresize.emit(e.clone()));

                // resize_listener = Some(listener);

                // move || drop(resize_listener)
            }
        },
        (),
    );

    html! {
        <div class={"file-display"} ref={div_ref}>
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
