use yew::prelude::*;
use yew_icons::IconId;

use crate::components::sidebar_button::*;

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    let hovering = use_state(|| false);
    let onmouseenter = {
        let hovering = hovering.clone();
        Callback::from(move |_| hovering.set(true))
    };
    let onmouseleave = {
        let hovering = hovering.clone();
        Callback::from(move |_| hovering.set(false))
    };
    html! {
      <div class={"sidebar"} {onmouseenter} {onmouseleave}>
          <SidebarButton button_text={"All"} hovering={*hovering} icon={IconId::BootstrapFileEarmark}/>
          <SidebarButton button_text={"Images"} hovering={*hovering} icon={IconId::BootstrapFileEarmarkImage}/>
      </div>
    }
}
