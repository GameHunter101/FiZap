use yew::prelude::*;

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    html! {
    <div>
      <div class={"header"}>
        <div>
          {"FiZap"}
        </div>
        <div class={"grid grid-cols-3 gap-3"}>
            <button>
                {"All files"}
            </button>
            <button>
                {"Images"}
            </button>
            <button>
                {"Folders"}
            </button>
        </div>
        <div>
            {"Logout"}
        </div>
      </div>
      <div class={"file-display"}>
      {(0..50).map(|i|
      render_box(i)
      ).collect::<Html>()}
      </div>
    </div>
    }
}

pub fn render_box(index: usize) -> Html {

  html! {
    <div class={"file"}>
        {format!("Box {}", index)}
    </div>
  }
}