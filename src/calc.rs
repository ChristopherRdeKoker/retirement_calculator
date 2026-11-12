use serde::{Deserialize, Serialize};

pub const MAX_AGE: u32 = 100;
pub const MAX_MONEY: f64 = 1e12;

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

#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub field: Field,
    pub message: String,
}

fn issue(field: Field, message: &str) -> Issue {
    Issue {
        field,
        message: message.to_string(),
    }
}

fn check_rate(issues: &mut Vec<Issue>, field: Field, value: f64, label: &str, min: f64, max: f64) {
    if !value.is_finite() || value < min || value > max {
        issues.push(issue(
            field,
            &format!("{label} must be between {min}% and {max}%."),
        ));
    }
}

fn check_money(issues: &mut Vec<Issue>, field: Field, value: f64, label: &str) {
    if !value.is_finite() || value < 0.0 {
        issues.push(issue(field, &format!("{label} can't be negative.")));
    } else if value > MAX_MONEY {
        issues.push(issue(
            field,
            &format!("{label} must be at most R 1 000 000 000 000."),
        ));
    }
}

pub fn validate(i: &Inputs) -> Vec<Issue> {
    let mut issues = Vec::new();
    if i.current_age < 16 || i.current_age > 99 {
        issues.push(issue(
            Field::CurrentAge,
            "Current age must be between 16 and 99.",
        ));
    }
    if i.retire_age <= i.current_age {
        issues.push(issue(
            Field::RetireAge,
            "Retirement age must be later than your current age.",
        ));
    } else if i.retire_age >= MAX_AGE {
        issues.push(issue(Field::RetireAge, "Retirement age must be below 100."));
    }
    if i.life_expectancy <= i.retire_age {
        issues.push(issue(
            Field::LifeExpectancy,
            "Life expectancy must be later than your retirement age.",
        ));
    } else if i.life_expectancy > MAX_AGE {
        issues.push(issue(
            Field::LifeExpectancy,
            "Life expectancy can be at most 100.",
        ));
    }
    check_money(&mut issues, Field::Savings, i.savings, "Current savings");
    check_money(
        &mut issues,
        Field::MonthlyContribution,
        i.monthly_contribution,
        "Monthly contribution",
    );
    check_money(
        &mut issues,
        Field::DesiredIncome,
        i.desired_income,
        "Desired income",
    );
    check_rate(
        &mut issues,
        Field::ContributionIncrease,
        i.contribution_increase,
        "Yearly increase",
        0.0,
        50.0,
    );
    check_rate(
        &mut issues,
        Field::ReturnPre,
        i.return_pre,
        "Return before retirement",
        -20.0,
        50.0,
    );
    check_rate(
        &mut issues,
        Field::ReturnPost,
        i.return_post,
        "Return after retirement",
        -20.0,
        50.0,
    );
    check_rate(
        &mut issues,
        Field::Inflation,
        i.inflation,
        "Inflation",
        -10.0,
        50.0,
    );
    issues
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub age: u32,
    pub nominal: f64,
    pub real: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Projection {
    pub points: Vec<Point>,
    pub pot_nominal: f64,
    pub pot_real: f64,
    pub first_withdrawal: f64,
    pub depleted_at: Option<f64>,
    pub sustainable: bool,
}

impl Projection {
    pub fn lasts_until(&self) -> Option<u32> {
        self.depleted_at.map(|a| a.floor() as u32)
    }
}

fn monthly_rate(annual_pct: f64) -> f64 {
    annual_pct / 100.0 / 12.0
}

fn inflation_factor(inflation_pct: f64, years: f64) -> f64 {
    (1.0 + inflation_pct / 100.0).powf(years)
}

struct Run {
    points: Vec<Point>,
    pot_nominal: f64,
    first_withdrawal: f64,
    depleted_at: Option<f64>,
}

fn simulate(i: &Inputs, income_today: f64, record: bool) -> Run {
    let r_pre = monthly_rate(i.return_pre);
    let r_post = monthly_rate(i.return_post);
    let growth = 1.0 + i.contribution_increase / 100.0;
    let infl = 1.0 + i.inflation / 100.0;
    let saving_years = i.retire_age - i.current_age;
    let total_years = MAX_AGE - i.current_age;

    let mut balance = i.savings;
    let mut contribution = i.monthly_contribution;
    let mut withdrawal = income_today * infl.powi(saving_years as i32);
    let first_withdrawal = withdrawal;
    let mut pot_nominal = 0.0;
    let mut depleted_at = None;
    let mut points = Vec::new();

    for year in 0..=total_years {
        let age = i.current_age + year;
        if record {
            points.push(Point {
                age,
                nominal: balance,
                real: balance / infl.powi(year as i32),
            });
        }
        if year == total_years {
            break;
        }
        if year == saving_years {
            pot_nominal = balance;
        }
        if year < saving_years {
            if year > 0 {
                contribution *= growth;
            }
            for _ in 0..12 {
                balance = balance * (1.0 + r_pre) + contribution;
            }
        } else {
            if year > saving_years {
                withdrawal *= infl;
            }
            for month in 0..12 {
                if depleted_at.is_some() {
                    break;
                }
                balance *= 1.0 + r_post;
                if balance + 1e-6 * withdrawal.max(1.0) < withdrawal {
                    depleted_at = Some(age as f64 + month as f64 / 12.0);
                    balance = 0.0;
                } else {
                    balance = (balance - withdrawal).max(0.0);
                }
            }
        }
    }
    Run {
        points,
        pot_nominal,
        first_withdrawal,
        depleted_at,
    }
}

pub fn project(i: &Inputs) -> Result<Projection, Vec<Issue>> {
    let issues = validate(i);
    if !issues.is_empty() {
        return Err(issues);
    }
    let run = simulate(i, i.desired_income, true);
    let years = (i.retire_age - i.current_age) as f64;
    let sustainable = match run.depleted_at {
        None => true,
        Some(age) => age >= i.life_expectancy as f64,
    };
    Ok(Projection {
        points: run.points,
        pot_nominal: run.pot_nominal,
        pot_real: run.pot_nominal / inflation_factor(i.inflation, years),
        first_withdrawal: run.first_withdrawal,
        depleted_at: run.depleted_at,
        sustainable,
    })
}

pub fn sustainable_income(i: &Inputs) -> Option<f64> {
    if !validate(i).is_empty() {
        return None;
    }
    let lasts = |income: f64| match simulate(i, income, false).depleted_at {
        None => true,
        Some(age) => age >= i.life_expectancy as f64,
    };
    let mut lo = 0.0_f64;
    let mut hi = 1_000.0_f64;
    while lasts(hi) {
        lo = hi;
        hi *= 4.0;
        if !hi.is_finite() || hi > 1e30 {
            return Some(lo);
        }
    }
    for _ in 0..80 {
        let mid = (lo + hi) / 2.0;
        if lasts(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(lo)
}

pub fn format_rand(value: f64) -> String {
    if !value.is_finite() {
        return "R —".to_string();
    }
    let rounded = value.abs().round();
    let digits = format!("{rounded:.0}");
    let mut grouped = String::new();
    for (n, ch) in digits.chars().enumerate() {
        if n > 0 && (digits.len() - n) % 3 == 0 {
            grouped.push('\u{a0}');
        }
        grouped.push(ch);
    }
    let sign = if value < 0.0 && rounded > 0.0 {
        "−"
    } else {
        ""
    };
    format!("{sign}R\u{a0}{grouped}")
}

pub fn format_compact(value: f64) -> String {
    let v = value.abs();
    let (scaled, suffix) = if v >= 1e12 {
        (value / 1e12, "tn")
    } else if v >= 1e9 {
        (value / 1e9, "bn")
    } else if v >= 1e6 {
        (value / 1e6, "m")
    } else if v >= 1e3 {
        (value / 1e3, "k")
    } else {
        return format!("R{value:.0}");
    };
    let text = if scaled.abs() >= 100.0 || (scaled - scaled.round()).abs() < 0.05 {
        format!("{scaled:.0}")
    } else {
        format!("{scaled:.1}")
    };
    format!("R{text}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat() -> Inputs {
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

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-6 * b.abs().max(1.0)
    }

    #[test]
    fn defaults_are_valid() {
        assert!(validate(&Inputs::default()).is_empty());
        assert!(project(&Inputs::default()).is_ok());
    }

    #[test]
    fn zero_return_sums_contributions() {
        let p = project(&flat()).unwrap();
        assert!(close(p.pot_nominal, 120_000.0));
        assert!(close(p.pot_real, 120_000.0));
    }

    #[test]
    fn zero_return_drawdown_lasts_exactly() {
        let p = project(&flat()).unwrap();
        assert_eq!(p.depleted_at, Some(50.0));
        assert_eq!(p.lasts_until(), Some(50));
        assert!(p.sustainable);
    }

    #[test]
    fn slightly_more_income_is_not_sustainable() {
        let mut i = flat();
        i.desired_income = 1_001.0;
        let p = project(&i).unwrap();
        assert_eq!(p.lasts_until(), Some(49));
        assert!(!p.sustainable);
    }

    #[test]
    fn monthly_compounding_of_lump_sum() {
        let mut i = flat();
        i.savings = 100_000.0;
        i.monthly_contribution = 0.0;
        i.return_pre = 12.0;
        let p = project(&i).unwrap();
        assert!(close(p.pot_nominal, 100_000.0 * 1.01_f64.powi(120)));
    }

    #[test]
    fn contributions_compound_monthly_at_month_end() {
        let mut i = flat();
        i.retire_age = 31;
        i.return_pre = 12.0;
        let p = project(&i).unwrap();
        let expected = 1_000.0 * (1.01_f64.powi(12) - 1.0) / 0.01;
        assert!(close(p.pot_nominal, expected));
    }

    #[test]
    fn contribution_grows_each_year() {
        let mut i = flat();
        i.retire_age = 33;
        i.contribution_increase = 10.0;
        let p = project(&i).unwrap();
        assert!(close(p.pot_nominal, 12_000.0 + 13_200.0 + 14_520.0));
    }

    #[test]
    fn income_is_inflated_to_retirement_and_keeps_rising() {
        let mut i = flat();
        i.inflation = 10.0;
        i.retire_age = 32;
        i.life_expectancy = 34;
        i.savings = 1_000_000.0;
        i.monthly_contribution = 0.0;
        let p = project(&i).unwrap();
        assert!(close(p.first_withdrawal, 1_210.0));
        assert!(close(p.pot_real, 1_000_000.0 / 1.21));
        let at_33 = p.points.iter().find(|pt| pt.age == 33).unwrap();
        let at_34 = p.points.iter().find(|pt| pt.age == 34).unwrap();
        assert!(close(at_33.nominal, 1_000_000.0 - 12.0 * 1_210.0));
        assert!(close(at_34.nominal, at_33.nominal - 12.0 * 1_331.0));
    }

    #[test]
    fn savings_beyond_100_reports_none() {
        let mut i = flat();
        i.savings = 10_000_000.0;
        i.desired_income = 100.0;
        let p = project(&i).unwrap();
        assert_eq!(p.depleted_at, None);
        assert!(p.sustainable);
        assert_eq!(p.points.last().unwrap().age, MAX_AGE);
    }

    #[test]
    fn nothing_saved_depletes_at_retirement() {
        let mut i = flat();
        i.monthly_contribution = 0.0;
        let p = project(&i).unwrap();
        assert_eq!(p.depleted_at, Some(40.0));
        assert!(!p.sustainable);
    }

    #[test]
    fn zero_income_never_depletes() {
        let mut i = flat();
        i.monthly_contribution = 0.0;
        i.desired_income = 0.0;
        let p = project(&i).unwrap();
        assert_eq!(p.depleted_at, None);
    }

    #[test]
    fn points_cover_every_year_and_are_zero_after_depletion() {
        let p = project(&flat()).unwrap();
        assert_eq!(p.points.first().unwrap().age, 30);
        assert_eq!(p.points.len(), (MAX_AGE - 30 + 1) as usize);
        assert!(p
            .points
            .iter()
            .filter(|pt| pt.age > 50)
            .all(|pt| pt.nominal == 0.0));
    }

    #[test]
    fn retirement_not_after_current_age_is_rejected() {
        let mut i = flat();
        i.retire_age = 30;
        let issues = project(&i).unwrap_err();
        assert!(issues.iter().any(|x| x.field == Field::RetireAge
            && x.message == "Retirement age must be later than your current age."));
        i.retire_age = 25;
        assert!(project(&i).is_err());
    }

    #[test]
    fn life_expectancy_must_follow_retirement() {
        let mut i = flat();
        i.life_expectancy = 40;
        assert!(validate(&i)
            .iter()
            .any(|x| x.field == Field::LifeExpectancy));
        i.life_expectancy = 101;
        assert!(validate(&i)
            .iter()
            .any(|x| x.field == Field::LifeExpectancy));
    }

    #[test]
    fn rejects_negative_nan_and_out_of_range() {
        let mut i = flat();
        i.savings = -1.0;
        i.return_pre = f64::NAN;
        i.inflation = 80.0;
        i.monthly_contribution = f64::INFINITY;
        let fields: Vec<Field> = validate(&i).into_iter().map(|x| x.field).collect();
        assert!(fields.contains(&Field::Savings));
        assert!(fields.contains(&Field::ReturnPre));
        assert!(fields.contains(&Field::Inflation));
        assert!(fields.contains(&Field::MonthlyContribution));
        assert!(sustainable_income(&i).is_none());
    }

    #[test]
    fn huge_numbers_stay_finite() {
        let i = Inputs {
            current_age: 16,
            retire_age: 99,
            life_expectancy: 100,
            savings: MAX_MONEY,
            monthly_contribution: MAX_MONEY,
            contribution_increase: 50.0,
            return_pre: 50.0,
            return_post: 50.0,
            inflation: 50.0,
            desired_income: MAX_MONEY,
        };
        let p = project(&i).unwrap();
        assert!(p.pot_nominal.is_finite() && p.pot_nominal > 1e20);
        assert!(p.pot_real.is_finite());
        assert!(p
            .points
            .iter()
            .all(|pt| pt.nominal.is_finite() && pt.real.is_finite()));
        assert!(sustainable_income(&i).unwrap().is_finite());
        assert!(format_rand(p.pot_nominal).starts_with("R\u{a0}"));
    }

    #[test]
    fn sustainable_income_matches_exact_case() {
        let s = sustainable_income(&flat()).unwrap();
        assert!((s - 1_000.0).abs() < 0.01, "{s}");
        let mut i = flat();
        i.desired_income = s;
        assert!(project(&i).unwrap().sustainable);
    }

    #[test]
    fn negative_returns_are_allowed() {
        let mut i = flat();
        i.savings = 100_000.0;
        i.return_pre = -12.0;
        let p = project(&i).unwrap();
        assert!(p.pot_nominal < 100_000.0 + 120_000.0);
    }

    #[test]
    fn rand_formatting() {
        assert_eq!(format_rand(1_234_567.0), "R\u{a0}1\u{a0}234\u{a0}567");
        assert_eq!(format_rand(999.4), "R\u{a0}999");
        assert_eq!(format_rand(0.0), "R\u{a0}0");
        assert_eq!(format_rand(-1_500.0), "−R\u{a0}1\u{a0}500");
        assert_eq!(format_rand(f64::NAN), "R —");
        assert_eq!(format_compact(2_500_000.0), "R2.5m");
        assert_eq!(format_compact(12_000.0), "R12k");
        assert_eq!(format_compact(0.0), "R0");
    }
}
