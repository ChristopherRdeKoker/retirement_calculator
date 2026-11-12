use super::geometry::{svg_number, ChartGeometry, HEIGHT, PAD_BOTTOM, PAD_LEFT, PAD_TOP};
use crate::format::{format_compact, format_rand};
use leptos::prelude::*;

const HOVER_BOX_WIDTH: f64 = 150.0;
const HOVER_BOX_HEIGHT: f64 = 46.0;
const HOVER_BOX_OFFSET: f64 = 12.0;
const HOVER_TEXT_INSET: f64 = 10.0;
const HOVER_TITLE_BASELINE: f64 = 18.0;
const HOVER_VALUE_BASELINE: f64 = 36.0;
const Y_LABEL_GAP: f64 = 8.0;
const Y_LABEL_NUDGE: f64 = 4.0;
const X_LABEL_LIFT: f64 = 8.0;
const RETIRE_LABEL_DROP: f64 = 12.0;
const LIFE_MARKER_DROP: f64 = 20.0;
const LIFE_LABEL_DROP: f64 = 32.0;

fn grid(geometry: &ChartGeometry) -> impl IntoView {
    let right = svg_number(geometry.plot_right());
    geometry
        .ticks
        .iter()
        .map(|tick| {
            let y = svg_number(geometry.y(*tick));
            view! {
                <line class="grid" x1=svg_number(PAD_LEFT) x2=right.clone() y1=y.clone() y2=y.clone()></line>
                <text class="axis" x=svg_number(PAD_LEFT - Y_LABEL_GAP) y=svg_number(geometry.y(*tick) + Y_LABEL_NUDGE) text-anchor="end">{format_compact(*tick)}</text>
            }
        })
        .collect_view()
}

fn age_axis(geometry: &ChartGeometry) -> impl IntoView {
    let step = geometry.age_step();
    (geometry.start..=geometry.end)
        .filter(|age| age.is_multiple_of(step))
        .map(|age| {
            view! {
                <text class="axis" x=svg_number(geometry.x(age as f64)) y=svg_number(HEIGHT - X_LABEL_LIFT) text-anchor="middle">{age.to_string()}</text>
            }
        })
        .collect_view()
}

fn series_paths(geometry: &ChartGeometry) -> (String, String) {
    let base = svg_number(geometry.y(0.0));
    let corners: Vec<String> = geometry
        .visible()
        .map(|point| {
            format!(
                "{},{}",
                svg_number(geometry.x(point.age as f64)),
                svg_number(geometry.y(point.real))
            )
        })
        .collect();
    let line_path = format!("M{}", corners.join(" L"));
    let last_x = geometry
        .visible()
        .last()
        .map(|point| geometry.x(point.age as f64))
        .unwrap_or(PAD_LEFT);
    let area_path = format!(
        "{line_path} L{},{base} L{},{base} Z",
        svg_number(last_x),
        svg_number(PAD_LEFT)
    );
    (line_path, area_path)
}

pub fn static_layer(geometry: &ChartGeometry) -> impl IntoView {
    let base = geometry.y(0.0);
    let (line_path, area_path) = series_paths(geometry);
    let retire_x = geometry.x(geometry.data.retire_age as f64);
    let life_x = geometry.x(geometry.data.life_expectancy as f64);
    let (retire_anchor, retire_label_x) = geometry.label_anchor(retire_x);
    let (life_anchor, life_label_x) = geometry.label_anchor(life_x);
    let depleted = geometry
        .data
        .depleted_at
        .filter(|age| *age <= geometry.end as f64)
        .map(|age| view! { <circle class="depleted" cx=svg_number(geometry.x(age)) cy=svg_number(base) r="5"></circle> });
    view! {
        <g aria-hidden="true">
            {grid(geometry)}
            {age_axis(geometry)}
            <path class="area" d=area_path></path>
            <path class="line" d=line_path></path>
            <line class="marker" x1=svg_number(retire_x) x2=svg_number(retire_x) y1=svg_number(PAD_TOP) y2=svg_number(base)></line>
            <text class="label" x=svg_number(retire_label_x) y=svg_number(PAD_TOP + RETIRE_LABEL_DROP) text-anchor=retire_anchor>{format!("Retire at {}", geometry.data.retire_age)}</text>
            <line class="marker marker-life" x1=svg_number(life_x) x2=svg_number(life_x) y1=svg_number(PAD_TOP + LIFE_MARKER_DROP) y2=svg_number(base)></line>
            <text class="label label-muted" x=svg_number(life_label_x) y=svg_number(PAD_TOP + LIFE_LABEL_DROP) text-anchor=life_anchor>{format!("Age {}", geometry.data.life_expectancy)}</text>
            <line class="baseline" x1=svg_number(PAD_LEFT) x2=svg_number(geometry.plot_right()) y1=svg_number(base) y2=svg_number(base)></line>
            {depleted}
        </g>
    }
}

pub fn hover_layer(geometry: &ChartGeometry, age: u32) -> impl IntoView {
    let point = geometry.visible().find(|point| point.age == age).copied();
    point.map(|point| {
        let x = geometry.x(point.age as f64);
        let y = geometry.y(point.real);
        let box_x = if x + HOVER_BOX_OFFSET + HOVER_BOX_WIDTH > geometry.plot_right() {
            x - HOVER_BOX_OFFSET - HOVER_BOX_WIDTH
        } else {
            x + HOVER_BOX_OFFSET
        };
        let box_y = (y - HOVER_BOX_HEIGHT / 2.0).clamp(PAD_TOP, HEIGHT - PAD_BOTTOM - HOVER_BOX_HEIGHT);
        view! {
            <g class="hover" aria-hidden="true">
                <line class="crosshair" x1=svg_number(x) x2=svg_number(x) y1=svg_number(PAD_TOP) y2=svg_number(geometry.y(0.0))></line>
                <circle class="dot" cx=svg_number(x) cy=svg_number(y) r="5"></circle>
                <rect class="tip" x=svg_number(box_x) y=svg_number(box_y) width=svg_number(HOVER_BOX_WIDTH) height=svg_number(HOVER_BOX_HEIGHT) rx="6"></rect>
                <text class="tip-title" x=svg_number(box_x + HOVER_TEXT_INSET) y=svg_number(box_y + HOVER_TITLE_BASELINE)>{format!("Age {}", point.age)}</text>
                <text class="tip-value" x=svg_number(box_x + HOVER_TEXT_INSET) y=svg_number(box_y + HOVER_VALUE_BASELINE)>{format_rand(point.real)}</text>
            </g>
        }
    })
}
