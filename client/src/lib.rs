use yew::prelude::*;
use yew_router::{
    history::{AnyHistory, History, MemoryHistory},
    prelude::*,
};

use pages::dashboard::Dashboard;

mod pages {
    pub mod dashboard;
}

mod components {
    pub mod file;
    pub mod file_manager;
    pub mod header;
    pub mod sidebar;
    pub mod sidebar_button;
}

mod store;
mod utils;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
}

#[derive(Properties, PartialEq, Debug)]
pub struct ServerAppProps {
    pub url: AttrValue,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {<Dashboard />},
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
    <>
        <Router history={history}>
            <Switch<Route> render={switch} />
        </Router>
    </>
    }
}
