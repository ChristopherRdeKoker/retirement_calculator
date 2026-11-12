use super::help::{HelpText, HelpToggle};
use super::number_field::{adornment, regroup_money, Adornment};
use crate::calc::{Inputs, Issue, Subject};
use crate::form::deposits::{AMOUNT_HELP, YEAR_HELP};
use crate::form::{DepositRow, FieldKind, FormText, HelpKey};
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Year,
    Amount,
}

struct PartDef {
    name: &'static str,
    label: &'static str,
    help: &'static str,
    kind: FieldKind,
}

fn part_def(part: Part) -> PartDef {
    match part {
        Part::Year => PartDef {
            name: "year",
            label: "Year",
            help: YEAR_HELP,
            kind: FieldKind::Age,
        },
        Part::Amount => PartDef {
            name: "amount",
            label: "Amount",
            help: AMOUNT_HELP,
            kind: FieldKind::Money,
        },
    }
}

fn subject(part: Part, row: usize) -> Subject {
    match part {
        Part::Year => Subject::DepositYear(row),
        Part::Amount => Subject::DepositAmount(row),
    }
}

fn help_id(part: Part, key: usize) -> String {
    format!("deposit-{key}-{}-help", part_def(part).name)
}

fn help_key(part: Part, key: usize) -> HelpKey {
    match part {
        Part::Year => HelpKey::DepositYear(key),
        Part::Amount => HelpKey::DepositAmount(key),
    }
}

#[component]
fn DepositInput(
    part: Part,
    key: usize,
    position: Signal<usize>,
    text: RwSignal<String>,
    parsed: Memo<Result<Inputs, Vec<Issue>>>,
    open_help: RwSignal<Option<HelpKey>>,
) -> impl IntoView {
    let def = part_def(part);
    let kind = def.kind;
    let id = format!("deposit-{key}-{}", def.name);
    let help_id = help_id(part, key);
    let error_id = format!("{id}-error");
    let help = help_key(part, key);
    let Adornment {
        prefix, input_mode, ..
    } = adornment(kind);
    let touched = RwSignal::new(false);
    let issue = move || {
        if !touched.get() && text.with(|value| value.trim().is_empty()) {
            return None;
        }
        let wanted = subject(part, position.get());
        parsed
            .get()
            .err()
            .and_then(|issues| issues.into_iter().find(|issue| issue.subject == wanted))
            .map(|issue| issue.message)
    };
    let described_by = {
        let help_id = help_id.clone();
        let error_id = error_id.clone();
        move || {
            let mut ids = Vec::new();
            if open_help.get() == Some(help) {
                ids.push(help_id.clone());
            }
            if issue().is_some() {
                ids.push(error_id.clone());
            }
            ids.join(" ")
        }
    };
    let help_label = format!("deposit {}", def.label.to_lowercase());
    view! {
        <div class="field" class:field-invalid=move || issue().is_some()>
            <div class="field-head">
                <label for=id.clone()>{def.label}</label>
                <HelpToggle key=help label=help_label help_id=help_id.clone() open_help />
            </div>
            <div class="control">
                {prefix.map(|symbol| view! { <span class="affix affix-pre" aria-hidden="true">{symbol}</span> })}
                <input
                    id=id.clone()
                    name=id
                    type="text"
                    inputmode=input_mode
                    autocomplete="off"
                    spellcheck="false"
                    class:has-pre=prefix.is_some()
                    aria-invalid=move || if issue().is_some() { "true" } else { "false" }
                    aria-describedby=described_by
                    prop:value=move || text.get()
                    on:input=move |event| text.set(event_target_value(&event))
                    on:blur=move |_| touched.set(true)
                    on:change=move |_| {
                        if kind == FieldKind::Money {
                            regroup_money(text);
                        }
                    }
                />
            </div>
            {move || issue().map(|message| view! { <p class="error" id=error_id.clone()>{message}</p> })}
        </div>
    }
}

#[component]
pub fn DepositRowView(
    row: DepositRow,
    text: FormText,
    parsed: Memo<Result<Inputs, Vec<Issue>>>,
    open_help: RwSignal<Option<HelpKey>>,
) -> impl IntoView {
    let key = row.key;
    let position = Signal::derive(move || {
        text.deposits()
            .with(|rows| rows.iter().position(|item| item.key == key))
            .unwrap_or(0)
    });
    let number = move || position.get() + 1;
    view! {
        <div class="deposit-row">
            <div class="deposit-row-head">
                <p class="deposit-row-title">{move || format!("Deposit {}", number())}</p>
                <button
                    type="button"
                    class="btn btn-small btn-quiet"
                    aria-label=move || format!("Remove deposit {}", number())
                    on:click=move |_| text.remove_deposit(key)
                >
                    "Remove"
                </button>
            </div>
            {[Part::Year, Part::Amount]
                .map(|part| view! { <HelpText key=help_key(part, key) help_id=help_id(part, key) text=part_def(part).help open_help /> })}
            <div class="deposit-row-fields">
                <DepositInput part=Part::Year key position text=row.year parsed open_help />
                <DepositInput part=Part::Amount key position text=row.amount parsed open_help />
            </div>
        </div>
    }
}
