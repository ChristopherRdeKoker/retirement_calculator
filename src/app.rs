use crate::calc::Inputs;
use crate::components::input_form::InputForm;
use crate::components::savings_card::SavingsCard;
use crate::components::scenario_panel::ScenarioPanel;
use crate::components::summary::Summary;
use crate::form::FormText;
use crate::outcome::{headline, sustainability_sentence, Outcome};
use crate::store::{self, Loaded};
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
    let notice = RwSignal::new(if repaired {
        "Some saved data could not be read, so it was reset.".to_string()
    } else {
        String::new()
    });

    Effect::new(move |_| {
        if let Ok(inputs) = parsed.get() {
            if !store::save_inputs(&inputs) {
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

    let reset = move |_| {
        text.fill(&Inputs::default());
        notice.set("Inputs reset to the defaults.".to_string());
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
                <ScenarioPanel saved text parsed notice storage_ok />
            </main>
            <footer class="footnote">
                <p>"Illustration only, not financial advice. Returns are assumed constant and compound monthly; contributions rise once a year; withdrawals rise with inflation once a year. Tax and fees are not modelled."</p>
            </footer>
        </div>
    }
}
