mod app;
mod calc;
mod chart;
mod format;
mod store;

use wasm_bindgen::JsCast;

fn main() {
    let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("app"))
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        return;
    };
    root.set_text_content(None);
    leptos::mount::mount_to(root, app::App).forget();
}
