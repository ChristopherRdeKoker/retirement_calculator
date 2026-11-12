mod app;
mod calc;
mod chart;
mod components;
mod form;
mod format;
mod outcome;
mod scenarios;
mod store;

use leptos::logging::error;
use wasm_bindgen::JsCast;

const ROOT_ID: &str = "app";

fn main() {
    let Some(root) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(ROOT_ID))
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        error!("Retirement calculator: no #{ROOT_ID} element on the page, so nothing was mounted.");
        return;
    };
    root.set_text_content(None);
    leptos::mount::mount_to(root, app::App).forget();
}
