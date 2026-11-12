use crate::calc::{format_compact, format_rand, Point, MAX_AGE};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

const HEIGHT: f64 = 280.0;
const PAD_L: f64 = 52.0;
const PAD_R: f64 = 14.0;
const PAD_T: f64 = 16.0;
const PAD_B: f64 = 30.0;

#[derive(Clone, Debug, PartialEq)]
pub struct ChartData {
    pub points: Vec<Point>,
    pub retire_age: u32,
    pub life_expectancy: u32,
    pub depleted_at: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
struct Geom {
    width: f64,
    start: u32,
    end: u32,
    y_max: f64,
    ticks: Vec<f64>,
    data: ChartData,
}

impl Geom {
    fn new(data: ChartData, width: f64) -> Self {
        let start = data.points.first().map(|p| p.age).unwrap_or(0);
        let end = (data.life_expectancy + 5).min(MAX_AGE).max(start + 1);
        let peak = data
            .points
            .iter()
            .filter(|p| p.age <= end)
            .map(|p| p.real)
            .filter(|v| v.is_finite())
            .fold(0.0_f64, f64::max);
        let (y_max, ticks) = nice_ticks(peak);
        Geom {
            width,
            start,
            end,
            y_max,
            ticks,
            data,
        }
    }

    fn plot_w(&self) -> f64 {
        (self.width - PAD_L - PAD_R).max(10.0)
    }

    fn x(&self, age: f64) -> f64 {
        PAD_L + (age - self.start as f64) / (self.end - self.start) as f64 * self.plot_w()
    }

    fn y(&self, value: f64) -> f64 {
        let plot_h = HEIGHT - PAD_T - PAD_B;
        let v = if value.is_finite() { value } else { self.y_max };
        PAD_T + plot_h - (v / self.y_max).clamp(0.0, 1.0) * plot_h
    }

    fn visible(&self) -> impl Iterator<Item = &Point> {
        self.data.points.iter().filter(move |p| p.age <= self.end)
    }
}

fn nice_ticks(peak: f64) -> (f64, Vec<f64>) {
    if !(peak.is_finite() && peak > 0.0) {
        return (1.0, vec![0.0, 1.0]);
    }
    let raw = peak / 4.0;
    let mag = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0]
        .iter()
        .map(|m| m * mag)
        .find(|s| *s >= raw)
        .unwrap_or(10.0 * mag);
    let count = (peak / step).ceil().max(1.0) as usize;
    let ticks = (0..=count).map(|n| n as f64 * step).collect();
    (count as f64 * step, ticks)
}

fn f(v: f64) -> String {
    format!("{v:.1}")
}

fn static_layer(g: &Geom) -> impl IntoView {
    let base = g.y(0.0);
    let right = PAD_L + g.plot_w();
    let grid = g
        .ticks
        .iter()
        .map(|t| {
            let y = f(g.y(*t));
            view! {
                <line class="grid" x1=f(PAD_L) x2=f(right) y1=y.clone() y2=y.clone()></line>
                <text class="axis" x=f(PAD_L - 8.0) y=f(g.y(*t) + 4.0) text-anchor="end">{format_compact(*t)}</text>
            }
        })
        .collect_view();
    let span = g.end - g.start;
    let step = if span > 40 {
        10
    } else if span > 15 {
        5
    } else {
        2
    };
    let x_ticks = (g.start..=g.end)
        .filter(|a| a % step == 0)
        .map(|a| {
            view! {
                <text class="axis" x=f(g.x(a as f64)) y=f(HEIGHT - 8.0) text-anchor="middle">{a.to_string()}</text>
            }
        })
        .collect_view();
    let line: Vec<String> = g
        .visible()
        .map(|p| format!("{},{}", f(g.x(p.age as f64)), f(g.y(p.real))))
        .collect();
    let line_path = format!("M{}", line.join(" L"));
    let last_x = g
        .visible()
        .last()
        .map(|p| g.x(p.age as f64))
        .unwrap_or(PAD_L);
    let area_path = format!(
        "{line_path} L{},{} L{},{} Z",
        f(last_x),
        f(base),
        f(PAD_L),
        f(base)
    );
    let rx = g.x(g.data.retire_age as f64);
    let lx = g.x(g.data.life_expectancy as f64);
    let retire_label = format!("Retire at {}", g.data.retire_age);
    let life_label = format!("Age {}", g.data.life_expectancy);
    let retire_anchor = if rx > PAD_L + g.plot_w() * 0.6 {
        "end"
    } else {
        "start"
    };
    let retire_tx = if retire_anchor == "end" {
        rx - 6.0
    } else {
        rx + 6.0
    };
    let life_anchor = if lx > PAD_L + g.plot_w() * 0.6 {
        "end"
    } else {
        "start"
    };
    let life_tx = if life_anchor == "end" {
        lx - 6.0
    } else {
        lx + 6.0
    };
    let depleted = g
        .data
        .depleted_at
        .filter(|a| *a <= g.end as f64)
        .map(|a| view! { <circle class="depleted" cx=f(g.x(a)) cy=f(base) r="5"></circle> });
    view! {
        <g aria-hidden="true">
            {grid}
            {x_ticks}
            <path class="area" d=area_path></path>
            <path class="line" d=line_path></path>
            <line class="marker" x1=f(rx) x2=f(rx) y1=f(PAD_T) y2=f(base)></line>
            <text class="label" x=f(retire_tx) y=f(PAD_T + 12.0) text-anchor=retire_anchor>{retire_label}</text>
            <line class="marker marker-life" x1=f(lx) x2=f(lx) y1=f(PAD_T + 20.0) y2=f(base)></line>
            <text class="label label-muted" x=f(life_tx) y=f(PAD_T + 32.0) text-anchor=life_anchor>{life_label}</text>
            <line class="baseline" x1=f(PAD_L) x2=f(right) y1=f(base) y2=f(base)></line>
            {depleted}
        </g>
    }
}

fn hover_layer(g: &Geom, age: u32) -> impl IntoView {
    let point = g.visible().find(|p| p.age == age).copied();
    point.map(|p| {
        let x = g.x(p.age as f64);
        let y = g.y(p.real);
        let box_w = 150.0;
        let box_h = 46.0;
        let bx = if x + 12.0 + box_w > PAD_L + g.plot_w() {
            x - 12.0 - box_w
        } else {
            x + 12.0
        };
        let by = (y - box_h / 2.0).clamp(PAD_T, HEIGHT - PAD_B - box_h);
        view! {
            <g class="hover" aria-hidden="true">
                <line class="crosshair" x1=f(x) x2=f(x) y1=f(PAD_T) y2=f(g.y(0.0))></line>
                <circle class="dot" cx=f(x) cy=f(y) r="5"></circle>
                <rect class="tip" x=f(bx) y=f(by) width=f(box_w) height=f(box_h) rx="6"></rect>
                <text class="tip-title" x=f(bx + 10.0) y=f(by + 18.0)>{format!("Age {}", p.age)}</text>
                <text class="tip-value" x=f(bx + 10.0) y=f(by + 36.0)>{format_rand(p.real)}</text>
            </g>
        }
    })
}

#[component]
pub fn Chart(data: Memo<Option<ChartData>>) -> impl IntoView {
    let wrap = NodeRef::<leptos::html::Div>::new();
    let width = RwSignal::new(640.0_f64);
    let hover = RwSignal::new(None::<u32>);
    let measure = move || {
        if let Some(el) = wrap.get_untracked() {
            let w = el.client_width() as f64;
            if w > 0.0 {
                width.set(w);
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

    let geom = Memo::new(move |_| data.get().map(|d| Geom::new(d, width.get())));

    let age_at = move |client_x: f64, target: Option<web_sys::EventTarget>| -> Option<u32> {
        let g = geom.get_untracked()?;
        let el = target?.dyn_into::<web_sys::Element>().ok()?;
        let rect = el.get_bounding_client_rect();
        if rect.width() <= 0.0 {
            return None;
        }
        let frac = ((client_x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let age = g.start as f64 + frac * (g.end - g.start) as f64;
        Some(age.round() as u32)
    };

    let on_key = move |ev: web_sys::KeyboardEvent| {
        let Some(g) = geom.get_untracked() else {
            return;
        };
        let current = hover.get_untracked();
        let next = match ev.key().as_str() {
            "ArrowRight" => Some(current.map_or(g.start, |a| (a + 1).min(g.end))),
            "ArrowLeft" => Some(current.map_or(g.start, |a| a.saturating_sub(1).max(g.start))),
            "Home" => Some(g.start),
            "End" => Some(g.end),
            "Escape" => {
                hover.set(None);
                None
            }
            _ => None,
        };
        if let Some(a) = next {
            ev.prevent_default();
            hover.set(Some(a));
        }
    };

    let summary = move || {
        data.get()
            .map(|d| {
                let peak = d.points.iter().map(|p| p.real).fold(0.0_f64, f64::max);
                format!(
                    "Savings balance in today's money by age. Peaks at about {} and retirement is at age {}. Use the arrow keys to read values.",
                    format_rand(peak),
                    d.retire_age
                )
            })
            .unwrap_or_default()
    };

    let legend = move || {
        data.get().map(|d| {
            let runs_out = d.depleted_at.map(|a| {
                view! {
                    <li><span class="key key-dot"></span>{format!("Runs out at {}", a.floor() as u32)}</li>
                }
            });
            view! {
                <ul class="legend" aria-hidden="true">
                    <li><span class="key key-line"></span>"Savings, today's money"</li>
                    <li><span class="key key-dash"></span>{format!("Retirement at {}", d.retire_age)}</li>
                    {runs_out}
                </ul>
            }
        })
    };

    view! {
        {legend}
        <div class="chart-wrap" node_ref=wrap>
            {move || {
                geom.get()
                    .map(|g| {
                        let w = g.width;
                        let plot_w = g.plot_w();
                        let gs = g.clone();
                        view! {
                            <svg
                                class="chart"
                                width=f(w)
                                height=f(HEIGHT)
                                viewBox=format!("0 0 {} {}", f(w), f(HEIGHT))
                                role="img"
                                tabindex="0"
                                aria-label=summary
                                on:keydown=on_key
                                on:blur=move |_| hover.set(None)
                            >
                                {static_layer(&gs)}
                                {move || hover.get().map(|a| hover_layer(&g, a))}
                                <rect
                                    class="hit"
                                    x=f(PAD_L)
                                    y=f(PAD_T)
                                    width=f(plot_w)
                                    height=f(HEIGHT - PAD_T - PAD_B)
                                    on:pointermove=move |ev| hover.set(age_at(ev.client_x() as f64, ev.current_target()))
                                    on:pointerdown=move |ev| hover.set(age_at(ev.client_x() as f64, ev.current_target()))
                                    on:pointerleave=move |ev| {
                                        if ev.pointer_type() == "mouse" {
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
