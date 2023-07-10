use gloo_console::log;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    HtmlDivElement, IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit,
};
use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::store::{load_item, Store};

#[derive(Properties, PartialEq)]
pub struct FileProps {
    pub name: String,
}

#[function_component(File)]
pub fn file(props: &FileProps) -> Html {
    let (_, dispatch) = use_store::<Store>();
    let name = props.name.clone();
    let div_ref = use_node_ref();
    let item_name = use_state(|| String::new());
    let name_state = item_name.clone();

    {
        let div = div_ref.clone();
        use_effect(move || {
            let options = IntersectionObserverInit::new();

            let div = div.cast::<HtmlDivElement>().expect("Div not set");

            let callback = Closure::wrap(Box::new(
                move |entries: Vec<JsValue>, _observer: IntersectionObserver| {
                    for entry in entries {
                        let entry = IntersectionObserverEntry::from(entry);
                        let is_intersecting = entry.is_intersecting();

                        let clone = name.clone();
                        let dispatch_clone = dispatch.clone();
                        spawn_local(async move {
                            if is_intersecting {
                                let res =
                                    load_item(clone.parse::<u32>().unwrap(), dispatch_clone).await;
                                match res {
                                    Ok(res) => name_state.set(res.0),
                                    Err(_) => {}
                                };
                            }
                        });
                    }
                },
            )
                as Box<dyn FnMut(Vec<JsValue>, IntersectionObserver)>);

            let observer =
                IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &options)
                    .unwrap();

            observer.observe(&div);

            move || {
                callback.forget();
                observer.unobserve(&div);
            }
        });
    }

    html! {
          <div class={"file"} ref={div_ref}>
              {format!("Box {}", props.name.clone())}
          </div>

    }
}
