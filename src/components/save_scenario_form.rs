use crate::calc::{Inputs, Issue};
use crate::scenarios::MAX_NAME;
use leptos::prelude::*;

#[component]
pub fn SaveScenarioForm(
    name: RwSignal<String>,
    parsed: Memo<Result<Inputs, Vec<Issue>>>,
    on_save: Callback<()>,
) -> impl IntoView {
    view! {
        <form
            class="save-row"
            on:submit=move |event| {
                event.prevent_default();
                on_save.run(());
            }
        >
            <label for="scenario-name" class="visually-hidden">"Scenario name"</label>
            <input
                id="scenario-name"
                type="text"
                maxlength=MAX_NAME.to_string()
                placeholder="Name, e.g. Retire at 60"
                autocomplete="off"
                prop:value=move || name.get()
                on:input=move |event| name.set(event_target_value(&event))
            />
            <button type="submit" class="btn" disabled=move || parsed.get().is_err()>"Save scenario"</button>
        </form>
    }
}
