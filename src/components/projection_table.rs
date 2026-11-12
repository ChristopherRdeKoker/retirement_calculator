use crate::format::format_rand;
use crate::outcome::Outcome;
use leptos::prelude::*;

#[component]
pub fn ProjectionTable(result: Memo<Option<Outcome>>) -> impl IntoView {
    let rows = move || {
        result.get().map(|outcome| {
            let end_age = outcome.inputs.display_end_age();
            outcome
                .projection
                .points
                .iter()
                .filter(|point| point.age <= end_age)
                .map(|point| {
                    view! {
                        <tr>
                            <th scope="row">{point.age}</th>
                            <td>{format_rand(point.real)}</td>
                            <td>{format_rand(point.nominal)}</td>
                        </tr>
                    }
                })
                .collect_view()
        })
    };
    view! {
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
                    <tbody>{rows}</tbody>
                </table>
            </div>
        </details>
    }
}
