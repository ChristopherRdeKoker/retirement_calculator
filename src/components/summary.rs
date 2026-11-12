use crate::calc::{Inputs, Issue};
use crate::format::format_rand;
use crate::outcome::{headline, Outcome};
use leptos::prelude::*;

const AFFORDABLE_ROUNDING: f64 = 100.0;

fn affordable_hint(outcome: &Outcome) -> String {
    outcome
        .affordable
        .map(|affordable| {
            format!(
                " To last until {}, plan on about {} a month in today's money.",
                outcome.inputs.life_expectancy,
                format_rand((affordable / AFFORDABLE_ROUNDING).floor() * AFFORDABLE_ROUNDING)
            )
        })
        .unwrap_or_default()
}

fn verdict(outcome: &Outcome) -> AnyView {
    let inputs = &outcome.inputs;
    if outcome.projection.sustainable {
        return view! {
            <p class="verdict verdict-good">
                <span class="badge" aria-hidden="true">"✓"</span>
                <span>
                    <strong>"Sustainable. "</strong>
                    {format!(
                        "An income of {} a month in today's money lasts past your planning age of {}.",
                        format_rand(inputs.desired_income),
                        inputs.life_expectancy
                    )}
                </span>
            </p>
        }
        .into_any();
    }
    let years_short = outcome
        .projection
        .lasts_until()
        .map(|age| inputs.life_expectancy.saturating_sub(age))
        .unwrap_or(0);
    view! {
        <p class="verdict verdict-bad">
            <span class="badge" aria-hidden="true">"!"</span>
            <span>
                <strong>"Not sustainable. "</strong>
                {format!(
                    "An income of {} a month runs out {} {} before age {}.{}",
                    format_rand(inputs.desired_income),
                    years_short,
                    if years_short == 1 { "year" } else { "years" },
                    inputs.life_expectancy,
                    affordable_hint(outcome)
                )}
            </span>
        </p>
    }
    .into_any()
}

fn issue_list(issues: Vec<Issue>) -> AnyView {
    view! {
        <section class="card summary summary-invalid" aria-labelledby="summary-title">
            <h2 id="summary-title" class="headline">"Check your inputs"</h2>
            <ul class="issues">
                {issues.into_iter().map(|issue| view! { <li>{issue.message}</li> }).collect_view()}
            </ul>
        </section>
    }
    .into_any()
}

fn projection_summary(outcome: Outcome) -> AnyView {
    let projection = &outcome.projection;
    let inputs = &outcome.inputs;
    view! {
        <section class="card summary" aria-labelledby="summary-title">
            <p class="eyebrow">"Your projection"</p>
            <h2 id="summary-title" class="headline">{headline(projection)}</h2>
            {verdict(&outcome)}
            <dl class="stats">
                <div class="stat">
                    <dt>{format!("Pot at age {}", inputs.retire_age)}</dt>
                    <dd class="stat-value">{format_rand(projection.pot_nominal)}</dd>
                    <dd class="stat-note">"Future rand"</dd>
                </div>
                <div class="stat">
                    <dt>"In today's money"</dt>
                    <dd class="stat-value">{format_rand(projection.pot_real)}</dd>
                    <dd class="stat-note">{format!("After {}% inflation a year", inputs.inflation)}</dd>
                </div>
                <div class="stat">
                    <dt>"First monthly withdrawal"</dt>
                    <dd class="stat-value">{format_rand(projection.first_withdrawal)}</dd>
                    <dd class="stat-note">{format!("{} in today's money", format_rand(inputs.desired_income))}</dd>
                </div>
            </dl>
        </section>
    }
    .into_any()
}

#[component]
pub fn Summary(
    result: Memo<Option<Outcome>>,
    parsed: Memo<Result<Inputs, Vec<Issue>>>,
) -> impl IntoView {
    move || match result.get() {
        Some(outcome) => projection_summary(outcome),
        None => issue_list(parsed.get().err().unwrap_or_default()),
    }
}
