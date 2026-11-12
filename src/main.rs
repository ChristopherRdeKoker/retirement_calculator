mod app;
mod calc;
mod chart;
mod components;
mod form;
mod format;
mod outcome;
mod scenarios;
mod store;

use wasm_bindgen::JsCast;

fn main() {
    let Some(root) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("app"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        return;
    };
    root.set_text_content(None);
    leptos::mount::mount_to(root, app::App).forget();
}
