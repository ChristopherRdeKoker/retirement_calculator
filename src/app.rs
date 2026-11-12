use crate::calc::{project, sustainable_income, Field, Inputs, Projection};
use crate::chart::{Chart, ChartData};
use crate::form::{field_def, group_digits, parse, FieldKind, FormText, FIELD_GROUPS};
use crate::format::{format_compact, format_rand};
use crate::store::{self, Loaded, Scenario, MAX_NAME, MAX_SCENARIOS};
use leptos::prelude::*;

#[derive(Clone, PartialEq)]
struct Outcome {
    inputs: Inputs,
    projection: Projection,
    affordable: Option<f64>,
}

fn outcome(inputs: Inputs) -> Option<Outcome> {
    let projection = project(&inputs).ok()?;
    let affordable = sustainable_income(&inputs);
    Some(Outcome {
        inputs,
        projection,
        affordable,
    })
}

fn headline(o: &Outcome) -> String {
    match o.projection.lasts_until() {
        Some(age) => format!("Your savings last until age {age}"),
        None => "Your savings last beyond age 100".to_string(),
    }
}

fn lasts_text(p: &Projection) -> String {
    match p.lasts_until() {
        Some(age) => format!("Age {age}"),
        None => "Beyond 100".to_string(),
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (initial, inputs_state) = store::load_inputs();
    let (saved, scenarios_state) = store::load_scenarios();
    let storage_ok = RwSignal::new(store::available());
    let repaired = inputs_state == Loaded::Repaired || scenarios_state == Loaded::Repaired;

    let text = FormText::new(&initial);
    let parsed = Memo::new(move |_| text.read());
    let result = Memo::new(move |_| parsed.get().ok().and_then(outcome));
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

    let chart_data = Memo::new(move |_| {
        result.get().map(|o| ChartData {
            points: o.projection.points.clone(),
            retire_age: o.inputs.retire_age,
            life_expectancy: o.inputs.life_expectancy,
            depleted_at: o.projection.depleted_at,
        })
    });

    let has_chart = Memo::new(move |_| chart_data.with(|d| d.is_some()));

    let announcement = move || match result.get() {
        Some(o) => format!(
            "{}. {}",
            headline(&o),
            if o.projection.sustainable {
                "Your desired income is sustainable."
            } else {
                "Your desired income is not sustainable."
            }
        ),
        None => "Check your inputs.".to_string(),
    };

    let issue_for = move |field: Field| {
        parsed
            .get()
            .err()
            .and_then(|list| list.into_iter().find(|i| i.field == field))
            .map(|i| i.message)
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

    let summary = move || match result.get() {
        None => {
            let issues = parsed.get().err().unwrap_or_default();
            view! {
                <section class="card summary summary-invalid" aria-labelledby="summary-title">
                    <h2 id="summary-title" class="headline">"Check your inputs"</h2>
                    <ul class="issues">
                        {issues.into_iter().map(|i| view! { <li>{i.message}</li> }).collect_view()}
                    </ul>
                </section>
            }
            .into_any()
        }
        Some(o) => {
            let p = o.projection.clone();
            let i = o.inputs;
            let verdict = if p.sustainable {
                view! {
                    <p class="verdict verdict-good">
                        <span class="badge" aria-hidden="true">"✓"</span>
                        <span>
                            <strong>"Sustainable. "</strong>
                            {format!(
                                "An income of {} a month in today's money lasts past your planning age of {}.",
                                format_rand(i.desired_income),
                                i.life_expectancy
                            )}
                        </span>
                    </p>
                }
                .into_any()
            } else {
                let short = p
                    .lasts_until()
                    .map(|a| i.life_expectancy.saturating_sub(a))
                    .unwrap_or(0);
                let afford = o
                    .affordable
                    .map(|a| {
                        format!(
                            " To last until {}, plan on about {} a month in today's money.",
                            i.life_expectancy,
                            format_rand((a / 100.0).floor() * 100.0)
                        )
                    })
                    .unwrap_or_default();
                view! {
                    <p class="verdict verdict-bad">
                        <span class="badge" aria-hidden="true">"!"</span>
                        <span>
                            <strong>"Not sustainable. "</strong>
                            {format!(
                                "An income of {} a month runs out {} {} before age {}.{}",
                                format_rand(i.desired_income),
                                short,
                                if short == 1 { "year" } else { "years" },
                                i.life_expectancy,
                                afford
                            )}
                        </span>
                    </p>
                }
                .into_any()
            };
            view! {
                <section class="card summary" aria-labelledby="summary-title">
                    <p class="eyebrow">"Your projection"</p>
                    <h2 id="summary-title" class="headline">{headline(&o)}</h2>
                    {verdict}
                    <dl class="stats">
                        <div class="stat">
                            <dt>{format!("Pot at age {}", i.retire_age)}</dt>
                            <dd class="stat-value">{format_rand(p.pot_nominal)}</dd>
                            <dd class="stat-note">"Future rand"</dd>
                        </div>
                        <div class="stat">
                            <dt>"In today's money"</dt>
                            <dd class="stat-value">{format_rand(p.pot_real)}</dd>
                            <dd class="stat-note">{format!("After {}% inflation a year", i.inflation)}</dd>
                        </div>
                        <div class="stat">
                            <dt>"First monthly withdrawal"</dt>
                            <dd class="stat-value">{format_rand(p.first_withdrawal)}</dd>
                            <dd class="stat-note">{format!("{} in today's money", format_rand(i.desired_income))}</dd>
                        </div>
                    </dl>
                </section>
            }
            .into_any()
        }
    };

    let table_rows = move || {
        result.get().map(|o| {
            o.projection
                .points
                .iter()
                .filter(|p| p.age <= o.inputs.display_end_age())
                .map(|p| {
                    view! {
                        <tr>
                            <th scope="row">{p.age}</th>
                            <td>{format_rand(p.real)}</td>
                            <td>{format_rand(p.nominal)}</td>
                        </tr>
                    }
                })
                .collect_view()
        })
    };

    let field_view = move |field: Field| {
        let def = field_def(field);
        let sig = text.signal(field);
        let kind = def.kind;
        let id = def.id;
        let hint_id = format!("{id}-hint");
        let err_id = format!("{id}-error");
        let has_hint = !def.hint.is_empty();
        let described = move || {
            let mut ids = Vec::new();
            if has_hint {
                ids.push(format!("{id}-hint"));
            }
            if issue_for(field).is_some() {
                ids.push(format!("{id}-error"));
            }
            ids.join(" ")
        };
        let (prefix, suffix, mode) = match kind {
            FieldKind::Age => (None, Some("years"), "numeric"),
            FieldKind::Money => (Some("R"), None, "decimal"),
            FieldKind::Percent => (None, Some("%"), "decimal"),
        };
        view! {
            <div class="field" class:field-invalid=move || issue_for(field).is_some()>
                <label for=id>{def.label}</label>
                <div class="control">
                    {prefix.map(|p| view! { <span class="affix affix-pre" aria-hidden="true">{p}</span> })}
                    <input
                        id=id
                        name=id
                        type="text"
                        inputmode=mode
                        autocomplete="off"
                        spellcheck="false"
                        class:has-pre=prefix.is_some()
                        class:has-suf=suffix.is_some()
                        aria-invalid=move || if issue_for(field).is_some() { "true" } else { "false" }
                        aria-describedby=described
                        prop:value=move || sig.get()
                        on:input=move |ev| sig.set(event_target_value(&ev))
                        on:change=move |_| {
                            if kind == FieldKind::Money {
                                if let Some(v) = parse(kind, &sig.get_untracked()) {
                                    if v >= 0.0 {
                                        sig.set(group_digits(v));
                                    }
                                }
                            }
                        }
                    />
                    {suffix.map(|s| view! { <span class="affix affix-suf" aria-hidden="true">{s}</span> })}
                </div>
                {has_hint.then(|| view! { <p class="hint" id=hint_id>{def.hint}</p> })}
                {move || issue_for(field).map(|m| view! { <p class="error" id=err_id.clone()>{m}</p> })}
            </div>
        }
    };

    let form = FIELD_GROUPS
        .iter()
        .map(|(title, members)| {
            view! {
                <fieldset class="group">
                    <legend>{*title}</legend>
                    <div class="group-fields">
                        {members.iter().map(|field| field_view(*field)).collect_view()}
                    </div>
                </fieldset>
            }
        })
        .collect_view();

    let scenario_list = move || {
        scenarios
            .get()
            .into_iter()
            .map(|s| {
                let to_load = s.clone();
                let to_delete = s.name.clone();
                let name_for_load = s.name.clone();
                let name_for_delete = s.name.clone();
                let detail = outcome(s.inputs)
                    .map(|o| {
                        format!(
                            "Retire at {} · {} a month · {}",
                            s.inputs.retire_age,
                            format_rand(s.inputs.desired_income),
                            lasts_text(&o.projection).to_lowercase()
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
            .map(|s| (s.name.clone(), outcome(s.inputs)))
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
            ("Lasts until", |o| match o.projection.lasts_until() {
                Some(a) => a.to_string(),
                None => "100+".to_string(),
            }),
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
                <div class="area-summary">{summary}</div>
                <section class="card area-chart" aria-labelledby="chart-title">
                    <div class="card-head">
                        <h2 id="chart-title">"Savings by age"</h2>
                        <p class="card-sub">"Balance in today's money: growth while you save, then drawdown"</p>
                    </div>
                    {move || {
                        if has_chart.get() {
                            view! { <Chart data=chart_data /> }.into_any()
                        } else {
                            view! { <p class="empty">"The chart appears once the inputs are valid."</p> }.into_any()
                        }
                    }}
                    <details class="data-table">
                        <summary>"Show as a table"</summary>
                        <div class="table-scroll" tabindex="0" role="region" aria-label="Savings by age table">
                            <table>
                                <thead>
                                    <tr>
                                        <th scope="col">"Age"</th>
                                        <th scope="col">"Today's money"</th>
                                        <th scope="col">"Future rand"</th>
                                    </tr>
                                </thead>
                                <tbody>{table_rows}</tbody>
                            </table>
                        </div>
                    </details>
                </section>
                <section class="card area-form" aria-labelledby="form-title">
                    <div class="card-head card-head-row">
                        <h2 id="form-title">"Your numbers"</h2>
                        <button type="button" class="btn btn-small btn-quiet" on:click=reset>"Reset"</button>
                    </div>
                    <form class="inputs" novalidate on:submit=|ev| ev.prevent_default()>{form}</form>
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
