use super::compare_table::CompareTable;
use super::save_scenario_form::SaveScenarioForm;
use super::scenario_list::ScenarioList;
use crate::calc::{Inputs, Issue};
use crate::form::FormText;
use crate::outcome::Outcome;
use crate::scenarios::{upsert, without, SaveError, SaveOutcome, Scenario, MAX_SCENARIOS};
use crate::store;
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ScenarioOutcome {
    pub scenario: Scenario,
    pub outcome: Option<Outcome>,
}

#[component]
pub fn ScenarioPanel(
    saved: Vec<Scenario>,
    text: FormText,
    parsed: Memo<Result<Inputs, Vec<Issue>>>,
    notice: RwSignal<String>,
    storage_ok: RwSignal<bool>,
) -> impl IntoView {
    let scenarios = RwSignal::new(saved);
    let name = RwSignal::new(String::new());
    let outcomes = Memo::new(move |_| {
        scenarios
            .get()
            .into_iter()
            .map(|scenario| ScenarioOutcome {
                outcome: Outcome::new(scenario.inputs.clone()),
                scenario,
            })
            .collect::<Vec<_>>()
    });

    let persist = move |list: Vec<Scenario>, done: String| {
        let stored = store::save_scenarios(&list);
        scenarios.set(list);
        if stored {
            notice.set(done);
        } else {
            storage_ok.set(false);
            notice.set(format!(
                "{done} Browser storage is unavailable, so it will be lost on reload."
            ));
        }
    };

    let on_load = Callback::new(move |scenario: Scenario| {
        text.fill(&scenario.inputs);
        name.set(scenario.name.clone());
        notice.set(format!("Loaded “{}”.", scenario.name));
    });

    let on_delete = Callback::new(move |target: String| {
        let list = without(scenarios.get_untracked(), &target);
        persist(list, format!("Deleted “{target}”."));
    });

    let on_save = Callback::new(move |()| {
        let Ok(inputs) = parsed.get_untracked() else {
            notice.set("Fix the highlighted inputs before saving.".to_string());
            return;
        };
        let mut list = scenarios.get_untracked();
        match upsert(&mut list, &name.get_untracked(), inputs) {
            Ok(SaveOutcome::Updated(label)) => persist(list, format!("Updated “{label}”.")),
            Ok(SaveOutcome::Added(label)) => {
                name.set(label.clone());
                persist(list, format!("Saved “{label}”."));
            }
            Err(SaveError::Full) => notice.set(format!(
                "You can keep {MAX_SCENARIOS} scenarios. Delete one to save another."
            )),
        }
    });

    view! {
        <section class="card area-scenarios" aria-labelledby="scenarios-title">
            <div class="card-head">
                <h2 id="scenarios-title">"Scenarios"</h2>
                <p class="card-sub">"Saved in this browser only"</p>
            </div>
            <SaveScenarioForm name parsed on_save />
            <p class="notice" role="status">{move || notice.get()}</p>
            <ScenarioList outcomes on_load on_delete />
            <CompareTable outcomes />
            {move || (!storage_ok.get()).then(|| view! {
                <p class="warning">"Browser storage is unavailable. Your inputs and scenarios will not be kept after you close this page."</p>
            })}
        </section>
    }
}
