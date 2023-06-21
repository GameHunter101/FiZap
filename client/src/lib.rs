use std::collections::HashMap;

use yew::prelude::*;
use yew_router::{
    history::{AnyHistory, History, MemoryHistory},
    prelude::*,
};

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
}

#[derive(Properties, PartialEq, Eq, Debug)]
pub struct ServerAppProps {
    pub url: AttrValue,
    pub queries: HashMap<String, String>,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {<p class={classes!("bg-black","text-red-500")}>{"Test!"}</p>},
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

#[function_component(ServerApp)]
pub fn server_app(props: &ServerAppProps) -> Html {
    let history = AnyHistory::from(MemoryHistory::new());
    history.push(&*props.url);
    html! {
        <Router history={history}>
            <Switch<Route> render={switch} />
        </Router>
    }
}
