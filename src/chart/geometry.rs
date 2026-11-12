use crate::calc::Point;

pub const HEIGHT: f64 = 280.0;
pub const PAD_LEFT: f64 = 52.0;
pub const PAD_RIGHT: f64 = 14.0;
pub const PAD_TOP: f64 = 16.0;
pub const PAD_BOTTOM: f64 = 30.0;
const MIN_PLOT_WIDTH: f64 = 10.0;
const TARGET_Y_TICKS: f64 = 4.0;
const TICK_MULTIPLIERS: [f64; 5] = [1.0, 2.0, 2.5, 5.0, 10.0];
const LABEL_FLIP_SHARE: f64 = 0.6;
const LABEL_GAP: f64 = 6.0;

#[derive(Clone, Debug, PartialEq)]
pub struct ChartData {
    pub points: Vec<Point>,
    pub retire_age: u32,
    pub end_age: u32,
    pub life_expectancy: u32,
    pub depleted_at: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChartGeometry {
    pub width: f64,
    pub start: u32,
    pub end: u32,
    pub y_max: f64,
    pub ticks: Vec<f64>,
    pub data: ChartData,
}

impl ChartGeometry {
    pub fn new(data: ChartData, width: f64) -> Self {
        let start = data.points.first().map(|point| point.age).unwrap_or(0);
        let end = data.end_age.max(start + 1);
        let peak = data
            .points
            .iter()
            .filter(|point| point.age <= end)
            .map(|point| point.real)
            .filter(|value| value.is_finite())
            .fold(0.0_f64, f64::max);
        let (y_max, ticks) = nice_ticks(peak);
        ChartGeometry {
            width,
            start,
            end,
            y_max,
            ticks,
            data,
        }
    }

    pub fn plot_width(&self) -> f64 {
        (self.width - PAD_LEFT - PAD_RIGHT).max(MIN_PLOT_WIDTH)
    }

    pub fn plot_height(&self) -> f64 {
        HEIGHT - PAD_TOP - PAD_BOTTOM
    }

    pub fn plot_right(&self) -> f64 {
        PAD_LEFT + self.plot_width()
    }

    pub fn x(&self, age: f64) -> f64 {
        PAD_LEFT + (age - self.start as f64) / (self.end - self.start) as f64 * self.plot_width()
    }

    pub fn y(&self, value: f64) -> f64 {
        let plot_height = self.plot_height();
        let shown = if value.is_finite() { value } else { self.y_max };
        PAD_TOP + plot_height - (shown / self.y_max).clamp(0.0, 1.0) * plot_height
    }

    pub fn visible(&self) -> impl Iterator<Item = &Point> {
        self.data
            .points
            .iter()
            .filter(move |point| point.age <= self.end)
    }

    pub fn age_step(&self) -> u32 {
        let span = self.end - self.start;
        if span > 40 {
            10
        } else if span > 15 {
            5
        } else {
            2
        }
    }

    pub fn label_anchor(&self, x: f64) -> (&'static str, f64) {
        if x > PAD_LEFT + self.plot_width() * LABEL_FLIP_SHARE {
            ("end", x - LABEL_GAP)
        } else {
            ("start", x + LABEL_GAP)
        }
    }
}

pub fn nice_ticks(peak: f64) -> (f64, Vec<f64>) {
    if !(peak.is_finite() && peak > 0.0) {
        return (1.0, vec![0.0, 1.0]);
    }
    let raw_step = peak / TARGET_Y_TICKS;
    let magnitude = 10f64.powf(raw_step.log10().floor());
    let step = TICK_MULTIPLIERS
        .iter()
        .map(|multiplier| multiplier * magnitude)
        .find(|candidate| *candidate >= raw_step)
        .unwrap_or(10.0 * magnitude);
    let count = (peak / step).ceil().max(1.0) as usize;
    let ticks = (0..=count).map(|n| n as f64 * step).collect();
    (count as f64 * step, ticks)
}

pub fn svg_number(value: f64) -> String {
    format!("{value:.1}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::MAX_AGE;

    fn data(life_expectancy: u32) -> ChartData {
        ChartData {
            points: (30..=MAX_AGE)
                .map(|age| Point {
                    age,
                    nominal: 0.0,
                    real: if age <= 60 {
                        (age - 30) as f64 * 1_000.0
                    } else {
                        0.0
                    },
                })
                .collect(),
            retire_age: 60,
            end_age: (life_expectancy + 5).min(MAX_AGE),
            life_expectancy,
            depleted_at: Some(61.0),
        }
    }

    #[test]
    fn nice_ticks_round_up_to_a_tidy_step() {
        assert_eq!(nice_ticks(0.0), (1.0, vec![0.0, 1.0]));
        assert_eq!(nice_ticks(f64::NAN), (1.0, vec![0.0, 1.0]));
        assert_eq!(nice_ticks(-5.0), (1.0, vec![0.0, 1.0]));
        assert_eq!(
            nice_ticks(30_000.0),
            (30_000.0, vec![0.0, 10_000.0, 20_000.0, 30_000.0])
        );
        assert_eq!(
            nice_ticks(9_000.0),
            (10_000.0, vec![0.0, 2_500.0, 5_000.0, 7_500.0, 10_000.0])
        );
        assert_eq!(
            nice_ticks(1_000_001.0),
            (1_500_000.0, vec![0.0, 500_000.0, 1_000_000.0, 1_500_000.0])
        );
    }

    #[test]
    fn geometry_window_ends_at_the_end_age() {
        assert_eq!(ChartGeometry::new(data(80), 640.0).end, 85);
        assert_eq!(ChartGeometry::new(data(98), 640.0).end, MAX_AGE);
    }

    #[test]
    fn geometry_window_is_at_least_one_year() {
        let mut short = data(80);
        short.end_age = 10;
        assert_eq!(ChartGeometry::new(short, 640.0).end, 31);
    }

    #[test]
    fn geometry_maps_ages_and_values_to_the_plot() {
        let geometry = ChartGeometry::new(data(80), 640.0);
        assert_eq!(geometry.start, 30);
        assert_eq!(geometry.y_max, 30_000.0);
        assert_eq!(geometry.x(30.0), PAD_LEFT);
        assert_eq!(geometry.x(85.0), 640.0 - PAD_RIGHT);
        assert_eq!(geometry.plot_right(), 640.0 - PAD_RIGHT);
        assert_eq!(geometry.y(0.0), HEIGHT - PAD_BOTTOM);
        assert_eq!(geometry.y(30_000.0), PAD_TOP);
        assert_eq!(geometry.y(60_000.0), PAD_TOP);
        assert_eq!(geometry.y(-1.0), HEIGHT - PAD_BOTTOM);
        assert_eq!(geometry.y(f64::NAN), PAD_TOP);
    }

    #[test]
    fn geometry_keeps_a_minimum_plot_width() {
        let geometry = ChartGeometry::new(data(80), 20.0);
        assert_eq!(geometry.plot_width(), MIN_PLOT_WIDTH);
    }

    #[test]
    fn age_step_widens_with_the_span() {
        let mut geometry = ChartGeometry::new(data(80), 640.0);
        assert_eq!(geometry.age_step(), 10);
        geometry.end = 50;
        assert_eq!(geometry.age_step(), 5);
        geometry.end = 45;
        assert_eq!(geometry.age_step(), 2);
    }

    #[test]
    fn labels_flip_to_the_left_past_sixty_percent_of_the_plot() {
        let geometry = ChartGeometry::new(data(80), 640.0);
        let flip_at = PAD_LEFT + geometry.plot_width() * LABEL_FLIP_SHARE;
        assert_eq!(
            geometry.label_anchor(flip_at),
            ("start", flip_at + LABEL_GAP)
        );
        assert_eq!(
            geometry.label_anchor(flip_at + 1.0),
            ("end", flip_at + 1.0 - LABEL_GAP)
        );
    }

    #[test]
    fn svg_numbers_have_one_decimal() {
        assert_eq!(svg_number(52.0), "52.0");
        assert_eq!(svg_number(1.25), "1.2");
        assert_eq!(svg_number(-3.06), "-3.1");
    }
}
