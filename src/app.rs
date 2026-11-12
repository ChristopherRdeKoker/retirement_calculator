use crate::calc::Inputs;
use crate::components::input_form::InputForm;
use crate::components::savings_card::SavingsCard;
use crate::components::summary::Summary;
use crate::form::FormText;
use crate::format::{format_compact, format_rand};
use crate::outcome::{headline, lasts_label, lasts_short, sustainability_sentence, Outcome};
use crate::store::{self, Loaded, Scenario, MAX_NAME, MAX_SCENARIOS};
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let (initial, inputs_state) = store::load_inputs();
    let (saved, scenarios_state) = store::load_scenarios();
    let storage_ok = RwSignal::new(store::available());
    let repaired = inputs_state == Loaded::Repaired || scenarios_state == Loaded::Repaired;

    let text = FormText::new(&initial);
    let parsed = Memo::new(move |_| text.read());
    let result = Memo::new(move |_| parsed.get().ok().and_then(Outcome::new));
    let scenarios = RwSignal::new(saved);
    let name = RwSignal::new(String::new());
    let notice = RwSignal::new(if repaired {
        "Some saved data could not be read, so it was reset.".to_string()
    } else {
        String::new()
    });

    Effect::new(move |_| {
        if let Ok(i) = parsed.get() {
            if !store::save_inputs(&i) {
                storage_ok.set(false);
            }
        }
    });

    let announcement = move || match result.get() {
        Some(outcome) => format!(
            "{}. {}",
            headline(&outcome.projection),
            sustainability_sentence(&outcome.projection)
        ),
        None => "Check your inputs.".to_string(),
    };

    let load = move |s: Scenario| {
        text.fill(&s.inputs);
        name.set(s.name.clone());
        notice.set(format!("Loaded “{}”.", s.name));
    };

    let persist = move |list: Vec<Scenario>, done: String| {
        if store::save_scenarios(&list) {
            scenarios.set(list);
            notice.set(done);
        } else {
            storage_ok.set(false);
            scenarios.set(list);
            notice.set(format!(
                "{done} Browser storage is unavailable, so it will be lost on reload."
            ));
        }
    };

    let delete = move |target: String| {
        let list: Vec<Scenario> = scenarios
            .get_untracked()
            .into_iter()
            .filter(|s| s.name != target)
            .collect();
        persist(list, format!("Deleted “{target}”."));
    };

    let save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Ok(inputs) = parsed.get_untracked() else {
            notice.set("Fix the highlighted inputs before saving.".to_string());
            return;
        };
        let mut list = scenarios.get_untracked();
        let mut label = store::clean_name(&name.get_untracked());
        if label.is_empty() {
            label = (1..=MAX_SCENARIOS + 1)
                .map(|n| format!("Scenario {n}"))
                .find(|c| !list.iter().any(|s| &s.name == c))
                .unwrap_or_else(|| "Scenario".to_string());
        }
        if let Some(existing) = list.iter_mut().find(|s| s.name == label) {
            existing.inputs = inputs;
            persist(list, format!("Updated “{label}”."));
        } else if list.len() >= MAX_SCENARIOS {
            notice.set(format!(
                "You can keep {MAX_SCENARIOS} scenarios. Delete one to save another."
            ));
        } else {
            list.push(Scenario {
                name: label.clone(),
                inputs,
            });
            name.set(label.clone());
            persist(list, format!("Saved “{label}”."));
        }
    };

    let reset = move |_| {
        text.fill(&Inputs::default());
        notice.set("Inputs reset to the defaults.".to_string());
    };

    let scenario_list = move || {
        scenarios
            .get()
            .into_iter()
            .map(|s| {
                let to_load = s.clone();
                let to_delete = s.name.clone();
                let name_for_load = s.name.clone();
                let name_for_delete = s.name.clone();
                let detail = Outcome::new(s.inputs)
                    .map(|o| {
                        format!(
                            "Retire at {} · {} a month · {}",
                            s.inputs.retire_age,
                            format_rand(s.inputs.desired_income),
                            lasts_label(&o.projection).to_lowercase()
                        )
                    })
                    .unwrap_or_default();
                view! {
                    <li class="scenario">
                        <div class="scenario-text">
                            <span class="scenario-name">{s.name.clone()}</span>
                            <span class="scenario-detail">{detail}</span>
                        </div>
                        <div class="scenario-actions">
                            <button
                                type="button"
                                class="btn btn-small"
                                aria-label=format!("Load {name_for_load}")
                                on:click=move |_| load(to_load.clone())
                            >
                                "Load"
                            </button>
                            <button
                                type="button"
                                class="btn btn-small btn-quiet"
                                aria-label=format!("Delete {name_for_delete}")
                                on:click=move |_| delete(to_delete.clone())
                            >
                                "Delete"
                            </button>
                        </div>
                    </li>
                }
            })
            .collect_view()
    };

    let compare = move || {
        let list = scenarios.get();
        if list.is_empty() {
            return view! {
                <p class="empty">"Save up to three scenarios to compare them side by side."</p>
            }
            .into_any();
        }
        let outcomes: Vec<(String, Option<Outcome>)> = list
            .iter()
            .map(|s| (s.name.clone(), Outcome::new(s.inputs)))
            .collect();
        type Row = (&'static str, fn(&Outcome) -> String);
        let rows: [Row; 6] = [
            ("Retire at", |o| o.inputs.retire_age.to_string()),
            ("Saving / month", |o| {
                format_compact(o.inputs.monthly_contribution)
            }),
            ("Pot (today)", |o| format_compact(o.projection.pot_real)),
            ("Income (today)", |o| {
                format_compact(o.inputs.desired_income)
            }),
            ("Lasts until", |o| lasts_short(&o.projection)),
            ("Sustainable", |o| {
                if o.projection.sustainable {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                }
            }),
        ];
        view! {
            <p class="compare-note">"Amounts in today's money. “100+” means the savings outlast age 100."</p>
            <div class="table-scroll" tabindex="0" role="region" aria-label="Scenario comparison">
                <table class="compare">
                    <thead>
                        <tr>
                            <td></td>
                            {outcomes.iter().map(|(n, _)| view! { <th scope="col">{n.clone()}</th> }).collect_view()}
                        </tr>
                    </thead>
                    <tbody>
                        {rows
                            .iter()
                            .map(|(label, get)| {
                                view! {
                                    <tr>
                                        <th scope="row">{*label}</th>
                                        {outcomes
                                            .iter()
                                            .map(|(_, o)| {
                                                let value = o.as_ref().map(get).unwrap_or_else(|| "—".to_string());
                                                let bad = value == "No";
                                                let good = value == "Yes";
                                                view! { <td class:bad=bad class:good=good>{value}</td> }
                                            })
                                            .collect_view()}
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        }
        .into_any()
    };

    view! {
        <div class="page">
            <header class="masthead">
                <h1>"Retirement calculator"</h1>
                <p class="lede">"See how long your savings could last. All amounts are in South African rand, and nothing leaves your browser."</p>
            </header>
            <main class="layout">
                <p class="visually-hidden" role="status" aria-live="polite">{announcement}</p>
                <div class="area-summary"><Summary result parsed /></div>
                <SavingsCard result />
                <section class="card area-form" aria-labelledby="form-title">
                    <div class="card-head card-head-row">
                        <h2 id="form-title">"Your numbers"</h2>
                        <button type="button" class="btn btn-small btn-quiet" on:click=reset>"Reset"</button>
                    </div>
                    <InputForm text parsed />
                </section>
                <section class="card area-scenarios" aria-labelledby="scenarios-title">
                    <div class="card-head">
                        <h2 id="scenarios-title">"Scenarios"</h2>
                        <p class="card-sub">"Saved in this browser only"</p>
                    </div>
                    <form class="save-row" on:submit=save>
                        <label for="scenario-name" class="visually-hidden">"Scenario name"</label>
                        <input
                            id="scenario-name"
                            type="text"
                            maxlength=MAX_NAME.to_string()
                            placeholder="Name, e.g. Retire at 60"
                            autocomplete="off"
                            prop:value=move || name.get()
                            on:input=move |ev| name.set(event_target_value(&ev))
                        />
                        <button type="submit" class="btn" disabled=move || parsed.get().is_err()>"Save scenario"</button>
                    </form>
                    <p class="notice" role="status">{move || notice.get()}</p>
                    <ul class="scenarios">{scenario_list}</ul>
                    {compare}
                    {move || (!storage_ok.get()).then(|| view! {
                        <p class="warning">"Browser storage is unavailable. Your inputs and scenarios will not be kept after you close this page."</p>
                    })}
                </section>
            </main>
            <footer class="footnote">
                <p>"Illustration only, not financial advice. Returns are assumed constant and compound monthly; contributions rise once a year; withdrawals rise with inflation once a year. Tax and fees are not modelled."</p>
            </footer>
        </div>
    }
}
