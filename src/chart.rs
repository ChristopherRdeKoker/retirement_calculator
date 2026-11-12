mod geometry;
mod layers;

pub use geometry::ChartData;

use crate::calc::whole_age;
use crate::format::format_rand;
use geometry::{svg_number, ChartGeometry, HEIGHT, PAD_LEFT, PAD_TOP};
use layers::{hover_layer, static_layer};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

const FALLBACK_WIDTH: f64 = 640.0;

#[component]
pub fn Chart(data: Memo<Option<ChartData>>) -> impl IntoView {
    let wrap = NodeRef::<leptos::html::Div>::new();
    let width = RwSignal::new(FALLBACK_WIDTH);
    let hover = RwSignal::new(None::<u32>);
    let measure = move || {
        if let Some(element) = wrap.get_untracked() {
            let measured = element.client_width() as f64;
            if measured > 0.0 {
                width.set(measured);
            }
        }
    };
    Effect::new(move |_| {
        if wrap.get().is_some() {
            measure();
        }
    });
    let handle = window_event_listener(leptos::ev::resize, move |_| measure());
    on_cleanup(move || handle.remove());

    let geometry = Memo::new(move |_| data.get().map(|data| ChartGeometry::new(data, width.get())));

    let age_at = move |client_x: f64, target: Option<web_sys::EventTarget>| -> Option<u32> {
        let chart = geometry.get_untracked()?;
        let element = target?.dyn_into::<web_sys::Element>().ok()?;
        let rect = element.get_bounding_client_rect();
        if rect.width() <= 0.0 {
            return None;
        }
        let share = ((client_x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let age = chart.start as f64 + share * (chart.end - chart.start) as f64;
        Some(age.round() as u32)
    };

    let on_key = move |event: web_sys::KeyboardEvent| {
        let Some(chart) = geometry.get_untracked() else {
            return;
        };
        let current = hover.get_untracked();
        let next = match event.key().as_str() {
            "ArrowRight" => Some(current.map_or(chart.start, |age| (age + 1).min(chart.end))),
            "ArrowLeft" => {
                Some(current.map_or(chart.start, |age| age.saturating_sub(1).max(chart.start)))
            }
            "Home" => Some(chart.start),
            "End" => Some(chart.end),
            "Escape" => {
                hover.set(None);
                None
            }
            _ => None,
        };
        if let Some(age) = next {
            event.prevent_default();
            hover.set(Some(age));
        }
    };

    let description = move || {
        data.get()
            .map(|data| {
                let peak = data
                    .points
                    .iter()
                    .map(|point| point.real)
                    .fold(0.0_f64, f64::max);
                format!(
                    "Savings balance in today's money by age. Peaks at about {} and retirement is at age {}. Use the arrow keys to read values.",
                    format_rand(peak),
                    data.retire_age
                )
            })
            .unwrap_or_default()
    };

    let legend = move || {
        data.get().map(|data| {
            let runs_out = data.depleted_at.map(|age| {
                view! {
                    <li><span class="key key-dot"></span>{format!("Runs out at {}", whole_age(age))}</li>
                }
            });
            view! {
                <ul class="legend" aria-hidden="true">
                    <li><span class="key key-line"></span>"Savings, today's money"</li>
                    <li><span class="key key-dash"></span>{format!("Retirement at {}", data.retire_age)}</li>
                    {runs_out}
                </ul>
            }
        })
    };

    view! {
        {legend}
        <div class="chart-wrap" node_ref=wrap>
            {move || {
                geometry.get()
                    .map(|chart| {
                        let width = chart.width;
                        let plot_width = chart.plot_width();
                        let plot_height = chart.plot_height();
                        let static_chart = chart.clone();
                        view! {
                            <svg
                                class="chart"
                                width=svg_number(width)
                                height=svg_number(HEIGHT)
                                viewBox=format!("0 0 {} {}", svg_number(width), svg_number(HEIGHT))
                                role="img"
                                tabindex="0"
                                aria-label=description
                                on:keydown=on_key
                                on:blur=move |_| hover.set(None)
                            >
                                {static_layer(&static_chart)}
                                {move || hover.get().map(|age| hover_layer(&chart, age))}
                                <rect
                                    class="hit"
                                    x=svg_number(PAD_LEFT)
                                    y=svg_number(PAD_TOP)
                                    width=svg_number(plot_width)
                                    height=svg_number(plot_height)
                                    on:pointermove=move |event| hover.set(age_at(event.client_x() as f64, event.current_target()))
                                    on:pointerdown=move |event| hover.set(age_at(event.client_x() as f64, event.current_target()))
                                    on:pointerleave=move |event| {
                                        if event.pointer_type() == "mouse" {
                                            hover.set(None);
                                        }
                                    }
                                ></rect>
                            </svg>
                        }
                    })
            }}
        </div>
    }
}
