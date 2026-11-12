use super::inputs::Inputs;

pub fn flat() -> Inputs {
    Inputs {
        current_age: 30,
        retire_age: 40,
        life_expectancy: 50,
        savings: 0.0,
        monthly_contribution: 1_000.0,
        contribution_increase: 0.0,
        return_pre: 0.0,
        return_post: 0.0,
        inflation: 0.0,
        desired_income: 1_000.0,
    }
}

pub fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() <= 1e-6 * expected.abs().max(1.0)
}
