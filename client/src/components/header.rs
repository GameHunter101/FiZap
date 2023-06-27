use yew::prelude::*;
use gloo_console::log;
use web_sys::{EventTarget, HtmlInputElement};
use wasm_bindgen::JsCast;

use crate::pages::dashboard::SearchContext;

#[function_component(Header)]
pub fn header() -> Html {
    let search_context = use_context::<SearchContext>().expect("No context found");

    let oninput = Callback::from(move |e: InputEvent| {
        let target:Option<EventTarget> = e.target();
        let input = target.and_then(|t| {
            t.dyn_into::<HtmlInputElement>().ok()
        });

        if let Some(input) = input {
            log!(input.value());
            search_context.dispatch(input.value())
        }
    });

    html! {
    <div class={"header"}>
      <div class={"text-cornell-red text-bold font-bold"}>
        {"FiZap"}
      </div>
      <div class={""}>
        /* <button {oninput}>
            {"Press me"}
        </button> */
        <input type="text" class="bg-transparent border-b-black border-b-2" placeholder="Search..." {oninput}/>
      </div>
      <div>
          {"Logout"}
      </div>
    </div>
    }
}
