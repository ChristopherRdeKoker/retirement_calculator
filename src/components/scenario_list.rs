use super::scenario_panel::ScenarioOutcome;
use crate::format::format_rand;
use crate::outcome::lasts_label;
use crate::scenarios::Scenario;
use leptos::prelude::*;

fn detail(item: &ScenarioOutcome) -> String {
    let inputs = item.scenario.inputs;
    item.outcome
        .as_ref()
        .map(|outcome| {
            format!(
                "Retire at {} · {} a month · {}",
                inputs.retire_age,
                format_rand(inputs.desired_income),
                lasts_label(&outcome.projection).to_lowercase()
            )
        })
        .unwrap_or_default()
}

#[component]
pub fn ScenarioList(
    outcomes: Memo<Vec<ScenarioOutcome>>,
    on_load: Callback<Scenario>,
    on_delete: Callback<String>,
) -> impl IntoView {
    let items = move || {
        outcomes
            .get()
            .into_iter()
            .map(|item| {
                let detail = detail(&item);
                let scenario = item.scenario;
                let name = scenario.name.clone();
                let name_to_delete = scenario.name.clone();
                view! {
                    <li class="scenario">
                        <div class="scenario-text">
                            <span class="scenario-name">{name.clone()}</span>
                            <span class="scenario-detail">{detail}</span>
                        </div>
                        <div class="scenario-actions">
                            <button
                                type="button"
                                class="btn btn-small"
                                aria-label=format!("Load {name}")
                                on:click=move |_| on_load.run(scenario.clone())
                            >
                                "Load"
                            </button>
                            <button
                                type="button"
                                class="btn btn-small btn-quiet"
                                aria-label=format!("Delete {name}")
                                on:click=move |_| on_delete.run(name_to_delete.clone())
                            >
                                "Delete"
                            </button>
                        </div>
                    </li>
                }
            })
            .collect_view()
    };
    view! { <ul class="scenarios">{items}</ul> }
}
