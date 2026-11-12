use super::deposit_section::DepositSection;
use super::number_field::NumberField;
use crate::calc::{Field, Inputs, Issue};
use crate::form::{FormText, HelpKey, FIELD_GROUPS};
use leptos::prelude::*;

const DEPOSITS_AFTER: Field = Field::ContributionIncrease;

#[component]
pub fn InputForm(text: FormText, parsed: Memo<Result<Inputs, Vec<Issue>>>) -> impl IntoView {
    let open_help = RwSignal::new(None::<HelpKey>);
    let handle = window_event_listener(leptos::ev::keydown, move |event| {
        if event.key() == "Escape" && open_help.get_untracked().is_some() {
            open_help.set(None);
        }
    });
    on_cleanup(move || handle.remove());
    let groups = FIELD_GROUPS
        .iter()
        .map(|(title, fields)| {
            view! {
                <fieldset class="group">
                    <legend>{*title}</legend>
                    <div class="group-fields">
                        {fields
                            .iter()
                            .map(|field| view! { <NumberField field=*field text=text.signal(*field) parsed open_help /> })
                            .collect_view()}
                    </div>
                </fieldset>
                {(fields.last() == Some(&DEPOSITS_AFTER)).then(|| view! { <DepositSection text parsed open_help /> })}
            }
        })
        .collect_view();
    view! { <form class="inputs" novalidate on:submit=|event| event.prevent_default()>{groups}</form> }
}
