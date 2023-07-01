use yew::prelude::*;
use web_sys::{Element, IntersectionObserver, IntersectionObserverEntry};

#[derive(Properties, PartialEq)]
pub struct FileProps {
    pub name: String,
}

#[function_component(File)]
pub fn file(props: &FileProps) -> Html {
    html! {
          <div class={"file"}>
              {format!("Box {}", props.name.clone())}
          </div>

    }
}
