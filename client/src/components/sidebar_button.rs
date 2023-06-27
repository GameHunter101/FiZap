use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub button_text: String,
    pub icon: IconId,
    pub hovering: bool,
}

#[function_component(SidebarButton)]
pub fn sidebar_button(props: &Props) -> Html {
    let local_hovering = use_state(|| false);
    let onmouseenter = {
        let hovering = local_hovering.clone();
        Callback::from(move |_| hovering.set(true))
    };
    let onmouseleave = {
        let hovering = local_hovering.clone();
        Callback::from(move |_| hovering.set(false))
    };
    html! {
        <button
            {onmouseenter}
            {onmouseleave}
            class={format!("sidebar-item {}", if props.hovering {"justify-end mr-2"} else {""})}
        >
            if props.hovering {
                <p class={format!("mx-auto transition-all {}", if *local_hovering {"text-black"} else {""})}>
                    {&props.button_text}
                </p>
            }
            <Icon icon_id={props.icon}/>
        </button>
    }
}
