use gloo_console::log;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{
    HtmlDivElement, IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit,
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FileProps {
    pub name: String,
}

#[function_component(File)]
pub fn file(props: &FileProps) -> Html {
    let name = props.name.clone();
    let div_ref = use_node_ref();

    {
        let div = div_ref.clone();
        use_effect(move || {
            let mut options = IntersectionObserverInit::new();

            let div = div.cast::<HtmlDivElement>().expect("Div not set");

            let callback = Closure::wrap(Box::new(
                move |entries: Vec<JsValue>, _observer: IntersectionObserver| {
                    for entry in entries {
                        let entry = IntersectionObserverEntry::from(entry);
                        let is_intersecting = entry.is_intersecting();

                        if is_intersecting {
                            // log::info!("hi");
                            log!(format!("hi {}",name.clone()));
                        }
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
