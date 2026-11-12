use super::deposit_row::DepositRowView;
use super::help::{HelpText, HelpToggle};
use crate::calc::{Inputs, Issue};
use crate::form::deposits::{SECTION_HELP, SECTION_TITLE};
use crate::form::{FormText, HelpKey};
use leptos::prelude::*;

#[component]
pub fn DepositSection(
    text: FormText,
    parsed: Memo<Result<Inputs, Vec<Issue>>>,
    open_help: RwSignal<Option<HelpKey>>,
) -> impl IntoView {
    let is_open = text.deposits_open();
    let rows = text.deposits();
    view! {
        <div class="deposits" role="group" aria-labelledby="deposits-title">
            <div class="deposits-head">
                <button
                    type="button"
                    id="deposits-title"
                    class="disclosure"
                    aria-controls="deposits-body"
                    aria-expanded=move || if is_open.get() { "true" } else { "false" }
                    on:click=move |_| is_open.update(|value| *value = !*value)
                >
                    {SECTION_TITLE}
                </button>
                <HelpToggle key=HelpKey::Deposits label=SECTION_TITLE.to_string() help_id="deposits-help".to_string() open_help />
            </div>
            <HelpText key=HelpKey::Deposits help_id="deposits-help".to_string() text=SECTION_HELP open_help />
            <div id="deposits-body" class="deposits-body" hidden=move || !is_open.get()>
                <For each=move || rows.get() key=|row| row.key let:row>
                    <DepositRowView row text parsed open_help />
                </For>
                <button type="button" class="btn btn-small" on:click=move |_| text.add_deposit()>
                    "Add deposit"
                </button>
            </div>
        </div>
    }
}
