use super::number_field::NumberField;
use crate::calc::{Inputs, Issue};
use crate::form::{FormText, FIELD_GROUPS};
use leptos::prelude::*;

#[component]
pub fn InputForm(text: FormText, parsed: Memo<Result<Inputs, Vec<Issue>>>) -> impl IntoView {
    let groups = FIELD_GROUPS
        .iter()
        .map(|(title, fields)| {
            view! {
                <fieldset class="group">
                    <legend>{*title}</legend>
                    <div class="group-fields">
                        {fields
                            .iter()
                            .map(|field| view! { <NumberField field=*field text=text.signal(*field) parsed /> })
                            .collect_view()}
                    </div>
                </fieldset>
            }
        })
        .collect_view();
    view! { <form class="inputs" novalidate on:submit=|event| event.prevent_default()>{groups}</form> }
}
