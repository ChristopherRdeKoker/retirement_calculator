use crate::form::HelpKey;
use leptos::prelude::*;

#[component]
pub fn HelpToggle(
    key: HelpKey,
    label: String,
    help_id: String,
    open_help: RwSignal<Option<HelpKey>>,
) -> impl IntoView {
    let is_open = move || open_help.get() == Some(key);
    let toggle =
        move |_| open_help.update(|open| *open = if *open == Some(key) { None } else { Some(key) });
    view! {
        <button
            type="button"
            class="help-toggle"
            aria-label=format!("About {label}")
            aria-controls=help_id
            aria-expanded=move || if is_open() { "true" } else { "false" }
            on:click=toggle
        >
            "?"
        </button>
    }
}

#[component]
pub fn HelpText(
    key: HelpKey,
    help_id: String,
    text: &'static str,
    open_help: RwSignal<Option<HelpKey>>,
) -> impl IntoView {
    view! { <p class="help" id=help_id hidden=move || open_help.get() != Some(key)>{text}</p> }
}
