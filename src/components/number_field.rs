use crate::calc::{Field, Inputs, Issue};
use crate::form::{field_def, group_digits, parse, FieldKind};
use leptos::prelude::*;

struct Adornment {
    prefix: Option<&'static str>,
    suffix: Option<&'static str>,
    input_mode: &'static str,
}

fn adornment(kind: FieldKind) -> Adornment {
    match kind {
        FieldKind::Age => Adornment {
            prefix: None,
            suffix: Some("years"),
            input_mode: "numeric",
        },
        FieldKind::Money => Adornment {
            prefix: Some("R"),
            suffix: None,
            input_mode: "decimal",
        },
        FieldKind::Percent => Adornment {
            prefix: None,
            suffix: Some("%"),
            input_mode: "decimal",
        },
    }
}

fn regroup_money(text: RwSignal<String>) {
    if let Some(value) = parse(FieldKind::Money, &text.get_untracked()) {
        if value >= 0.0 {
            text.set(group_digits(value));
        }
    }
}

#[component]
pub fn NumberField(
    field: Field,
    text: RwSignal<String>,
    parsed: Memo<Result<Inputs, Vec<Issue>>>,
) -> impl IntoView {
    let def = field_def(field);
    let kind = def.kind;
    let id = def.id;
    let hint_id = format!("{id}-hint");
    let error_id = format!("{id}-error");
    let has_hint = !def.hint.is_empty();
    let Adornment {
        prefix,
        suffix,
        input_mode,
    } = adornment(kind);
    let issue = move || {
        parsed
            .get()
            .err()
            .and_then(|issues| issues.into_iter().find(|issue| issue.field == field))
            .map(|issue| issue.message)
    };
    let described_by = move || {
        let mut ids = Vec::new();
        if has_hint {
            ids.push(format!("{id}-hint"));
        }
        if issue().is_some() {
            ids.push(format!("{id}-error"));
        }
        ids.join(" ")
    };
    view! {
        <div class="field" class:field-invalid=move || issue().is_some()>
            <label for=id>{def.label}</label>
            <div class="control">
                {prefix.map(|symbol| view! { <span class="affix affix-pre" aria-hidden="true">{symbol}</span> })}
                <input
                    id=id
                    name=id
                    type="text"
                    inputmode=input_mode
                    autocomplete="off"
                    spellcheck="false"
                    class:has-pre=prefix.is_some()
                    class:has-suf=suffix.is_some()
                    aria-invalid=move || if issue().is_some() { "true" } else { "false" }
                    aria-describedby=described_by
                    prop:value=move || text.get()
                    on:input=move |event| text.set(event_target_value(&event))
                    on:change=move |_| {
                        if kind == FieldKind::Money {
                            regroup_money(text);
                        }
                    }
                />
                {suffix.map(|symbol| view! { <span class="affix affix-suf" aria-hidden="true">{symbol}</span> })}
            </div>
            {has_hint.then(|| view! { <p class="hint" id=hint_id>{def.hint}</p> })}
            {move || issue().map(|message| view! { <p class="error" id=error_id.clone()>{message}</p> })}
        </div>
    }
}
