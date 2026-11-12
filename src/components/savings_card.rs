use super::projection_table::ProjectionTable;
use crate::chart::{Chart, ChartData};
use crate::outcome::Outcome;
use leptos::prelude::*;

#[component]
pub fn SavingsCard(result: Memo<Option<Outcome>>) -> impl IntoView {
    let chart_data = Memo::new(move |_| {
        result.get().map(|outcome| ChartData {
            points: outcome.projection.points.clone(),
            retire_age: outcome.inputs.retire_age,
            end_age: outcome.inputs.display_end_age(),
            life_expectancy: outcome.inputs.life_expectancy,
            depleted_at: outcome.projection.depleted_at,
        })
    });
    let has_chart = Memo::new(move |_| chart_data.with(|data| data.is_some()));
    view! {
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
            <ProjectionTable result />
        </section>
    }
}
