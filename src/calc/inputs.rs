use crate::format::group_thousands;
use serde::{Deserialize, Serialize};

pub const MAX_AGE: u32 = 100;
pub const MAX_MONEY: f64 = 1e12;
const MIN_CURRENT_AGE: u32 = 16;
const MAX_CURRENT_AGE: u32 = 99;
const MAX_RATE: f64 = 50.0;
const MIN_CONTRIBUTION_INCREASE: f64 = 0.0;
const MIN_RETURN: f64 = -20.0;
const MIN_INFLATION: f64 = -10.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Inputs {
    pub current_age: u32,
    pub retire_age: u32,
    pub life_expectancy: u32,
    pub savings: f64,
    pub monthly_contribution: f64,
    pub contribution_increase: f64,
    pub return_pre: f64,
    pub return_post: f64,
    pub inflation: f64,
    pub desired_income: f64,
}

impl Default for Inputs {
    fn default() -> Self {
        Inputs {
            current_age: 35,
            retire_age: 65,
            life_expectancy: 90,
            savings: 250_000.0,
            monthly_contribution: 5_000.0,
            contribution_increase: 5.0,
            return_pre: 10.0,
            return_post: 7.0,
            inflation: 5.0,
            desired_income: 25_000.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    CurrentAge,
    RetireAge,
    LifeExpectancy,
    Savings,
    MonthlyContribution,
    ContributionIncrease,
    ReturnPre,
    ReturnPost,
    Inflation,
    DesiredIncome,
}

impl Field {
    pub const ALL: [Field; 10] = [
        Field::CurrentAge,
        Field::RetireAge,
        Field::LifeExpectancy,
        Field::Savings,
        Field::MonthlyContribution,
        Field::ContributionIncrease,
        Field::ReturnPre,
        Field::ReturnPost,
        Field::Inflation,
        Field::DesiredIncome,
    ];

    pub fn index(self) -> usize {
        self as usize
    }
}

impl Inputs {
    pub fn value(&self, field: Field) -> f64 {
        match field {
            Field::CurrentAge => self.current_age as f64,
            Field::RetireAge => self.retire_age as f64,
            Field::LifeExpectancy => self.life_expectancy as f64,
            Field::Savings => self.savings,
            Field::MonthlyContribution => self.monthly_contribution,
            Field::ContributionIncrease => self.contribution_increase,
            Field::ReturnPre => self.return_pre,
            Field::ReturnPost => self.return_post,
            Field::Inflation => self.inflation,
            Field::DesiredIncome => self.desired_income,
        }
    }

    pub fn set_value(&mut self, field: Field, value: f64) {
        match field {
            Field::CurrentAge => self.current_age = value as u32,
            Field::RetireAge => self.retire_age = value as u32,
            Field::LifeExpectancy => self.life_expectancy = value as u32,
            Field::Savings => self.savings = value,
            Field::MonthlyContribution => self.monthly_contribution = value,
            Field::ContributionIncrease => self.contribution_increase = value,
            Field::ReturnPre => self.return_pre = value,
            Field::ReturnPost => self.return_post = value,
            Field::Inflation => self.inflation = value,
            Field::DesiredIncome => self.desired_income = value,
        }
    }

    pub fn display_end_age(&self) -> u32 {
        (self.life_expectancy + 5).min(MAX_AGE)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub field: Field,
    pub message: String,
}

fn issue(field: Field, message: String) -> Issue {
    Issue { field, message }
}

fn check_rate(issues: &mut Vec<Issue>, field: Field, value: f64, label: &str, min: f64) {
    if !value.is_finite() || value < min || value > MAX_RATE {
        issues.push(issue(
            field,
            format!("{label} must be between {min}% and {MAX_RATE}%."),
        ));
    }
}

fn check_money(issues: &mut Vec<Issue>, field: Field, value: f64, label: &str) {
    if !value.is_finite() || value < 0.0 {
        issues.push(issue(field, format!("{label} can't be negative.")));
    } else if value > MAX_MONEY {
        let limit = group_thousands(&format!("{MAX_MONEY:.0}"), ' ');
        issues.push(issue(field, format!("{label} must be at most R {limit}.")));
    }
}

fn check_ages(issues: &mut Vec<Issue>, inputs: &Inputs) {
    if !(MIN_CURRENT_AGE..=MAX_CURRENT_AGE).contains(&inputs.current_age) {
        issues.push(issue(
            Field::CurrentAge,
            format!("Current age must be between {MIN_CURRENT_AGE} and {MAX_CURRENT_AGE}."),
        ));
    }
    if inputs.retire_age <= inputs.current_age {
        issues.push(issue(
            Field::RetireAge,
            "Retirement age must be later than your current age.".to_string(),
        ));
    } else if inputs.retire_age >= MAX_AGE {
        issues.push(issue(
            Field::RetireAge,
            format!("Retirement age must be below {MAX_AGE}."),
        ));
    }
    if inputs.life_expectancy <= inputs.retire_age {
        issues.push(issue(
            Field::LifeExpectancy,
            "Life expectancy must be later than your retirement age.".to_string(),
        ));
    } else if inputs.life_expectancy > MAX_AGE {
        issues.push(issue(
            Field::LifeExpectancy,
            format!("Life expectancy can be at most {MAX_AGE}."),
        ));
    }
}

pub fn validate(inputs: &Inputs) -> Vec<Issue> {
    let mut issues = Vec::new();
    check_ages(&mut issues, inputs);
    check_money(
        &mut issues,
        Field::Savings,
        inputs.savings,
        "Current savings",
    );
    check_money(
        &mut issues,
        Field::MonthlyContribution,
        inputs.monthly_contribution,
        "Monthly contribution",
    );
    check_money(
        &mut issues,
        Field::DesiredIncome,
        inputs.desired_income,
        "Desired income",
    );
    check_rate(
        &mut issues,
        Field::ContributionIncrease,
        inputs.contribution_increase,
        "Yearly increase",
        MIN_CONTRIBUTION_INCREASE,
    );
    check_rate(
        &mut issues,
        Field::ReturnPre,
        inputs.return_pre,
        "Return before retirement",
        MIN_RETURN,
    );
    check_rate(
        &mut issues,
        Field::ReturnPost,
        inputs.return_post,
        "Return after retirement",
        MIN_RETURN,
    );
    check_rate(
        &mut issues,
        Field::Inflation,
        inputs.inflation,
        "Inflation",
        MIN_INFLATION,
    );
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::fixtures::flat;

    #[test]
    fn field_index_follows_the_order_of_all() {
        for (position, field) in Field::ALL.into_iter().enumerate() {
            assert_eq!(field.index(), position);
        }
    }

    #[test]
    fn set_value_then_value_reads_back_every_field() {
        let mut inputs = Inputs::default();
        for (position, field) in Field::ALL.into_iter().enumerate() {
            inputs.set_value(field, 40.0 + position as f64);
        }
        for (position, field) in Field::ALL.into_iter().enumerate() {
            assert_eq!(inputs.value(field), 40.0 + position as f64);
        }
        assert_eq!(inputs.current_age, 40);
        assert_eq!(inputs.desired_income, 49.0);
    }

    #[test]
    fn life_expectancy_must_follow_retirement() {
        let mut inputs = flat();
        inputs.life_expectancy = 40;
        assert!(validate(&inputs)
            .iter()
            .any(|issue| issue.field == Field::LifeExpectancy));
        inputs.life_expectancy = 101;
        assert!(validate(&inputs)
            .iter()
            .any(|issue| issue.field == Field::LifeExpectancy));
    }

    #[test]
    fn rejects_negative_nan_and_out_of_range() {
        let mut inputs = flat();
        inputs.savings = -1.0;
        inputs.return_pre = f64::NAN;
        inputs.inflation = 80.0;
        inputs.monthly_contribution = f64::INFINITY;
        let fields: Vec<Field> = validate(&inputs)
            .into_iter()
            .map(|issue| issue.field)
            .collect();
        assert!(fields.contains(&Field::Savings));
        assert!(fields.contains(&Field::ReturnPre));
        assert!(fields.contains(&Field::Inflation));
        assert!(fields.contains(&Field::MonthlyContribution));
    }

    #[test]
    fn validation_messages_are_exact() {
        let mut inputs = flat();
        inputs.current_age = 15;
        inputs.savings = MAX_MONEY * 2.0;
        inputs.monthly_contribution = -1.0;
        inputs.inflation = -11.0;
        let messages: Vec<String> = validate(&inputs)
            .into_iter()
            .map(|issue| issue.message)
            .collect();
        assert_eq!(
            messages,
            [
                "Current age must be between 16 and 99.",
                "Current savings must be at most R 1 000 000 000 000.",
                "Monthly contribution can't be negative.",
                "Inflation must be between -10% and 50%.",
            ]
        );
        let mut inputs = flat();
        inputs.retire_age = MAX_AGE;
        inputs.life_expectancy = MAX_AGE + 1;
        inputs.return_pre = 51.0;
        let messages: Vec<String> = validate(&inputs)
            .into_iter()
            .map(|issue| issue.message)
            .collect();
        assert_eq!(
            messages,
            [
                "Retirement age must be below 100.",
                "Life expectancy can be at most 100.",
                "Return before retirement must be between -20% and 50%.",
            ]
        );
    }

    #[test]
    fn defaults_are_valid() {
        assert!(validate(&Inputs::default()).is_empty());
    }

    #[test]
    fn display_end_age_is_five_years_past_life_expectancy_up_to_max_age() {
        let mut inputs = flat();
        assert_eq!(inputs.display_end_age(), 55);
        inputs.life_expectancy = 96;
        assert_eq!(inputs.display_end_age(), MAX_AGE);
    }
}
