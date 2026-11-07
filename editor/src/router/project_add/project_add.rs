use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;

use crate::routes::Route;

#[component]
pub fn ProjectAdd() -> Element {
    let nav = use_navigator();
    use_effect(move || {
        nav.replace(Route::Browser {});
    });
    rsx! {}
}
