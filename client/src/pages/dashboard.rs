use std::rc::Rc;

use yew::prelude::*;

use crate::components::{header::Header, sidebar::Sidebar, file_manager::FileManager};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Search {
    pub query: String,
}

impl Reducible for Search {
    type Action = String;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        Search {query: action}.into()
    }
}

pub type SearchContext = UseReducerHandle<Search>;

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    let search = use_reducer(|| Search {
        query: "".to_owned()
    });

    html! {
        <div class={"flex h-screen flex-col"}>
            <ContextProvider<SearchContext> context = {search}>
                <Header />
                <div class={"overflow-auto flex-grow relative"}>
                    <Sidebar />
                    <FileManager />
                </div>
            </ContextProvider<SearchContext>>
        </div>
    }
}
