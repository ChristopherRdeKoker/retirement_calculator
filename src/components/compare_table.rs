use super::scenario_panel::ScenarioOutcome;
use crate::calc::MAX_AGE;
use crate::format::format_compact;
use crate::outcome::{lasts_short, Outcome};
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tone {
    Plain,
    Good,
    Bad,
}

struct Cell {
    text: String,
    tone: Tone,
}

fn plain(text: String) -> Cell {
    Cell {
        text,
        tone: Tone::Plain,
    }
}

fn sustainable_cell(outcome: &Outcome) -> Cell {
    if outcome.projection.sustainable {
        Cell {
            text: "Yes".to_string(),
            tone: Tone::Good,
        }
    } else {
        Cell {
            text: "No".to_string(),
            tone: Tone::Bad,
        }
    }
}

fn deposits_cell(outcome: &Outcome) -> Cell {
    let deposits = &outcome.inputs.deposits;
    let total: f64 = deposits.iter().map(|deposit| deposit.amount).sum();
    let text = if deposits.is_empty() {
        "None".to_string()
    } else {
        format_compact(total)
    };
    plain(text)
}

type Row = (&'static str, fn(&Outcome) -> Cell);

const ROWS: [Row; 7] = [
    ("Retire at", |outcome| {
        plain(outcome.inputs.retire_age.to_string())
    }),
    ("Saving / month", |outcome| {
        plain(format_compact(outcome.inputs.monthly_contribution))
    }),
    ("Deposits", deposits_cell),
    ("Pot (today)", |outcome| {
        plain(format_compact(outcome.projection.pot_real))
    }),
    ("Income (today)", |outcome| {
        plain(format_compact(outcome.inputs.desired_income))
    }),
    ("Lasts until", |outcome| {
        plain(lasts_short(&outcome.projection))
    }),
    ("Sustainable", sustainable_cell),
];

fn table(outcomes: Vec<ScenarioOutcome>) -> AnyView {
    let headers = outcomes
        .iter()
        .map(|item| view! { <th scope="col">{item.scenario.name.clone()}</th> })
        .collect_view();
    let rows = ROWS
        .iter()
        .map(|(label, cell_for)| {
            let cells = outcomes
                .iter()
                .map(|item| {
                    let cell = item
                        .outcome
                        .as_ref()
                        .map(cell_for)
                        .unwrap_or_else(|| plain("—".to_string()));
                    view! {
                        <td class:bad=cell.tone == Tone::Bad class:good=cell.tone == Tone::Good>{cell.text}</td>
                    }
                })
                .collect_view();
            view! {
                <tr>
                    <th scope="row">{*label}</th>
                    {cells}
                </tr>
            }
        })
        .collect_view();
    view! {
        <p class="compare-note">{format!("Amounts in today's money, except deposits, which are the rand you pay in. “{MAX_AGE}+” means the savings outlast age {MAX_AGE}.")}</p>
        <div class="table-scroll" tabindex="0" role="region" aria-label="Scenario comparison">
            <table class="compare">
                <thead>
                    <tr>
                        <td></td>
                        {headers}
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
    .into_any()
}

#[component]
pub fn CompareTable(outcomes: Memo<Vec<ScenarioOutcome>>) -> impl IntoView {
    move || {
        let list = outcomes.get();
        if list.is_empty() {
            view! { <p class="empty">"Save up to three scenarios to compare them side by side."</p> }.into_any()
        } else {
            table(list)
        }
    }
}
