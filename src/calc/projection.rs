use super::inputs::{validate, Inputs, Issue, MAX_AGE};

const MONTHS_PER_YEAR: u32 = 12;
const INCOME_SEARCH_START: f64 = 1_000.0;
const INCOME_SEARCH_GROWTH: f64 = 4.0;
const INCOME_SEARCH_CEILING: f64 = 1e30;
const DEPLETION_TOLERANCE: f64 = 1e-6;
const INCOME_SEARCH_STEPS: u32 = 80;

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

pub fn whole_age(age: f64) -> u32 {
    age.floor() as u32
}

impl Projection {
    pub fn lasts_until(&self) -> Option<u32> {
        self.depleted_at.map(whole_age)
    }
}

fn monthly_rate(annual_percent: f64) -> f64 {
    annual_percent / 100.0 / MONTHS_PER_YEAR as f64
}

fn inflation_factor(inflation_percent: f64, years: f64) -> f64 {
    (1.0 + inflation_percent / 100.0).powf(years)
}

fn lasts_to(depleted_at: Option<f64>, life_expectancy: u32) -> bool {
    match depleted_at {
        None => true,
        Some(age) => age >= life_expectancy as f64,
    }
}

struct Simulation {
    points: Vec<Point>,
    pot_nominal: f64,
    first_withdrawal: f64,
    depleted_at: Option<f64>,
}

fn simulate(inputs: &Inputs, income_today: f64, record: bool) -> Simulation {
    let monthly_return_pre = monthly_rate(inputs.return_pre);
    let monthly_return_post = monthly_rate(inputs.return_post);
    let contribution_growth = 1.0 + inputs.contribution_increase / 100.0;
    let yearly_inflation = 1.0 + inputs.inflation / 100.0;
    let saving_years = inputs.retire_age - inputs.current_age;
    let total_years = MAX_AGE - inputs.current_age;

    let mut balance = inputs.savings;
    let mut contribution = inputs.monthly_contribution;
    let mut withdrawal = income_today * yearly_inflation.powi(saving_years as i32);
    let first_withdrawal = withdrawal;
    let mut pot_nominal = 0.0;
    let mut depleted_at = None;
    let mut points = Vec::new();

    for year in 0..=total_years {
        let age = inputs.current_age + year;
        if record {
            points.push(Point {
                age,
                nominal: balance,
                real: balance / yearly_inflation.powi(year as i32),
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
                contribution *= contribution_growth;
            }
            for _ in 0..MONTHS_PER_YEAR {
                balance = balance * (1.0 + monthly_return_pre) + contribution;
            }
        } else {
            if year > saving_years {
                withdrawal *= yearly_inflation;
            }
            for month in 0..MONTHS_PER_YEAR {
                if depleted_at.is_some() {
                    break;
                }
                balance *= 1.0 + monthly_return_post;
                if balance + DEPLETION_TOLERANCE * withdrawal.max(1.0) < withdrawal {
                    depleted_at = Some(age as f64 + month as f64 / MONTHS_PER_YEAR as f64);
                    balance = 0.0;
                } else {
                    balance = (balance - withdrawal).max(0.0);
                }
            }
        }
    }
    Simulation {
        points,
        pot_nominal,
        first_withdrawal,
        depleted_at,
    }
}

pub fn project(inputs: &Inputs) -> Result<Projection, Vec<Issue>> {
    let issues = validate(inputs);
    if !issues.is_empty() {
        return Err(issues);
    }
    let simulation = simulate(inputs, inputs.desired_income, true);
    let saving_years = (inputs.retire_age - inputs.current_age) as f64;
    Ok(Projection {
        points: simulation.points,
        pot_nominal: simulation.pot_nominal,
        pot_real: simulation.pot_nominal / inflation_factor(inputs.inflation, saving_years),
        first_withdrawal: simulation.first_withdrawal,
        depleted_at: simulation.depleted_at,
        sustainable: lasts_to(simulation.depleted_at, inputs.life_expectancy),
    })
}

pub fn sustainable_income(inputs: &Inputs) -> Option<f64> {
    if !validate(inputs).is_empty() {
        return None;
    }
    let lasts = |income: f64| {
        lasts_to(
            simulate(inputs, income, false).depleted_at,
            inputs.life_expectancy,
        )
    };
    let mut low = 0.0_f64;
    let mut high = INCOME_SEARCH_START;
    while lasts(high) {
        low = high;
        high *= INCOME_SEARCH_GROWTH;
        if !high.is_finite() || high > INCOME_SEARCH_CEILING {
            return Some(low);
        }
    }
    for _ in 0..INCOME_SEARCH_STEPS {
        let middle = (low + high) / 2.0;
        if lasts(middle) {
            low = middle;
        } else {
            high = middle;
        }
    }
    Some(low)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::fixtures::{close, flat};
    use crate::calc::inputs::{Field, MAX_MONEY};
    use crate::format::format_rand;

    #[test]
    fn invalid_inputs_have_no_sustainable_income() {
        let mut inputs = flat();
        inputs.savings = -1.0;
        assert!(sustainable_income(&inputs).is_none());
    }

    #[test]
    fn defaults_project() {
        assert!(project(&Inputs::default()).is_ok());
    }

    #[test]
    fn lasts_to_counts_the_planned_age_itself() {
        assert!(lasts_to(None, 90));
        assert!(lasts_to(Some(90.0), 90));
        assert!(!lasts_to(Some(89.99), 90));
    }

    #[test]
    fn zero_return_sums_contributions() {
        let projection = project(&flat()).unwrap();
        assert!(close(projection.pot_nominal, 120_000.0));
        assert!(close(projection.pot_real, 120_000.0));
    }

    #[test]
    fn zero_return_drawdown_lasts_exactly() {
        let projection = project(&flat()).unwrap();
        assert_eq!(projection.depleted_at, Some(50.0));
        assert_eq!(projection.lasts_until(), Some(50));
        assert!(projection.sustainable);
    }

    #[test]
    fn slightly_more_income_is_not_sustainable() {
        let mut inputs = flat();
        inputs.desired_income = 1_001.0;
        let projection = project(&inputs).unwrap();
        assert_eq!(projection.lasts_until(), Some(49));
        assert!(!projection.sustainable);
    }

    #[test]
    fn monthly_compounding_of_lump_sum() {
        let mut inputs = flat();
        inputs.savings = 100_000.0;
        inputs.monthly_contribution = 0.0;
        inputs.return_pre = 12.0;
        let projection = project(&inputs).unwrap();
        assert!(close(
            projection.pot_nominal,
            100_000.0 * 1.01_f64.powi(120)
        ));
    }

    #[test]
    fn contributions_compound_monthly_at_month_end() {
        let mut inputs = flat();
        inputs.retire_age = 31;
        inputs.return_pre = 12.0;
        let projection = project(&inputs).unwrap();
        let expected = 1_000.0 * (1.01_f64.powi(12) - 1.0) / 0.01;
        assert!(close(projection.pot_nominal, expected));
    }

    #[test]
    fn contribution_grows_each_year() {
        let mut inputs = flat();
        inputs.retire_age = 33;
        inputs.contribution_increase = 10.0;
        let projection = project(&inputs).unwrap();
        assert!(close(
            projection.pot_nominal,
            12_000.0 + 13_200.0 + 14_520.0
        ));
    }

    #[test]
    fn income_is_inflated_to_retirement_and_keeps_rising() {
        let mut inputs = flat();
        inputs.inflation = 10.0;
        inputs.retire_age = 32;
        inputs.life_expectancy = 34;
        inputs.savings = 1_000_000.0;
        inputs.monthly_contribution = 0.0;
        let projection = project(&inputs).unwrap();
        assert!(close(projection.first_withdrawal, 1_210.0));
        assert!(close(projection.pot_real, 1_000_000.0 / 1.21));
        let at_33 = projection
            .points
            .iter()
            .find(|point| point.age == 33)
            .unwrap();
        let at_34 = projection
            .points
            .iter()
            .find(|point| point.age == 34)
            .unwrap();
        assert!(close(at_33.nominal, 1_000_000.0 - 12.0 * 1_210.0));
        assert!(close(at_34.nominal, at_33.nominal - 12.0 * 1_331.0));
    }

    #[test]
    fn savings_beyond_100_reports_none() {
        let mut inputs = flat();
        inputs.savings = 10_000_000.0;
        inputs.desired_income = 100.0;
        let projection = project(&inputs).unwrap();
        assert_eq!(projection.depleted_at, None);
        assert!(projection.sustainable);
        assert_eq!(projection.points.last().unwrap().age, MAX_AGE);
    }

    #[test]
    fn nothing_saved_depletes_at_retirement() {
        let mut inputs = flat();
        inputs.monthly_contribution = 0.0;
        let projection = project(&inputs).unwrap();
        assert_eq!(projection.depleted_at, Some(40.0));
        assert!(!projection.sustainable);
    }

    #[test]
    fn zero_income_never_depletes() {
        let mut inputs = flat();
        inputs.monthly_contribution = 0.0;
        inputs.desired_income = 0.0;
        let projection = project(&inputs).unwrap();
        assert_eq!(projection.depleted_at, None);
    }

    #[test]
    fn points_cover_every_year_and_are_zero_after_depletion() {
        let projection = project(&flat()).unwrap();
        assert_eq!(projection.points.first().unwrap().age, 30);
        assert_eq!(projection.points.len(), (MAX_AGE - 30 + 1) as usize);
        assert!(projection
            .points
            .iter()
            .filter(|point| point.age > 50)
            .all(|point| point.nominal == 0.0));
    }

    #[test]
    fn retirement_not_after_current_age_is_rejected() {
        let mut inputs = flat();
        inputs.retire_age = 30;
        let issues = project(&inputs).unwrap_err();
        assert!(issues.iter().any(|issue| issue.field == Field::RetireAge
            && issue.message == "Retirement age must be later than your current age."));
        inputs.retire_age = 25;
        assert!(project(&inputs).is_err());
    }

    #[test]
    fn huge_numbers_stay_finite() {
        let inputs = Inputs {
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
        let projection = project(&inputs).unwrap();
        assert!(projection.pot_nominal.is_finite() && projection.pot_nominal > 1e20);
        assert!(projection.pot_real.is_finite());
        assert!(projection
            .points
            .iter()
            .all(|point| point.nominal.is_finite() && point.real.is_finite()));
        assert!(sustainable_income(&inputs).unwrap().is_finite());
        assert!(format_rand(projection.pot_nominal).starts_with("R\u{a0}"));
    }

    #[test]
    fn sustainable_income_matches_exact_case() {
        let income = sustainable_income(&flat()).unwrap();
        assert!((income - 1_000.0).abs() < 0.01, "{income}");
        let mut inputs = flat();
        inputs.desired_income = income;
        assert!(project(&inputs).unwrap().sustainable);
    }

    #[test]
    fn negative_returns_are_allowed() {
        let mut inputs = flat();
        inputs.savings = 100_000.0;
        inputs.return_pre = -12.0;
        let projection = project(&inputs).unwrap();
        assert!(projection.pot_nominal < 100_000.0 + 120_000.0);
    }

    const LUMP_50K_PLUS_2K_A_MONTH_AT_6_PERCENT_FOR_5_YEARS: f64 = 206_982.568_647;
    const SAVINGS_10K_PLUS_500_TIMES_240_MONTHS: f64 = 130_000.0;
    const CONTRIBUTION_12K_A_YEAR_RISING_20_PERCENT_FOR_5_YEARS: f64 =
        12_000.0 * (1.0 + 1.2 + 1.44 + 1.728 + 2.0736);
    const MILLION_DEFLATED_BY_6_PERCENT_FOR_10_YEARS: f64 = 558_394.776_915;
    const MILLION_PAID_OUT_OVER_120_MONTHS_AT_HALF_PERCENT_A_MONTH: f64 = 11_102.050_194;
    const RETIRE_40_PLUS_6_YEARS_AND_8_MONTHS: f64 = 40.0 + 6.0 + 8.0 / 12.0;

    fn lump_sum(savings: f64) -> Inputs {
        Inputs {
            savings,
            monthly_contribution: 0.0,
            ..flat()
        }
    }

    fn depletion_rank(projection: &Projection) -> f64 {
        projection.depleted_at.unwrap_or(f64::INFINITY)
    }

    #[test]
    fn lump_sum_and_contributions_match_the_closed_form() {
        let mut inputs = flat();
        inputs.current_age = 30;
        inputs.retire_age = 35;
        inputs.savings = 50_000.0;
        inputs.monthly_contribution = 2_000.0;
        inputs.return_pre = 6.0;
        let monthly = 0.005_f64;
        let closed_form = 50_000.0 * (1.0 + monthly).powi(60)
            + 2_000.0 * ((1.0 + monthly).powi(60) - 1.0) / monthly;
        let projection = project(&inputs).unwrap();
        assert!(close(
            closed_form,
            LUMP_50K_PLUS_2K_A_MONTH_AT_6_PERCENT_FOR_5_YEARS
        ));
        assert!(
            close(projection.pot_nominal, closed_form),
            "{}",
            projection.pot_nominal
        );
    }

    #[test]
    fn zero_return_adds_savings_and_every_contribution() {
        let mut inputs = flat();
        inputs.retire_age = 50;
        inputs.life_expectancy = 60;
        inputs.savings = 10_000.0;
        inputs.monthly_contribution = 500.0;
        let projection = project(&inputs).unwrap();
        assert!(close(
            projection.pot_nominal,
            SAVINGS_10K_PLUS_500_TIMES_240_MONTHS
        ));
    }

    #[test]
    fn zero_contributions_at_zero_return_keep_the_savings_unchanged() {
        let projection = project(&lump_sum(100_000.0)).unwrap();
        assert!(close(projection.pot_nominal, 100_000.0));
        assert!(projection
            .points
            .iter()
            .filter(|point| point.age <= 40)
            .all(|point| close(point.nominal, 100_000.0)));
    }

    #[test]
    fn contribution_rises_twenty_percent_each_year_for_five_years() {
        let mut inputs = flat();
        inputs.retire_age = 35;
        inputs.monthly_contribution = 1_000.0;
        inputs.contribution_increase = 20.0;
        let projection = project(&inputs).unwrap();
        assert!(close(projection.pot_nominal, 89_299.2));
        assert!(close(
            projection.pot_nominal,
            CONTRIBUTION_12K_A_YEAR_RISING_20_PERCENT_FOR_5_YEARS
        ));
    }

    #[test]
    fn pot_in_todays_money_is_deflated_over_the_saving_years() {
        let mut inputs = lump_sum(1_000_000.0);
        inputs.inflation = 6.0;
        let projection = project(&inputs).unwrap();
        assert!(close(projection.pot_nominal, 1_000_000.0));
        assert!(close(
            projection.pot_real,
            MILLION_DEFLATED_BY_6_PERCENT_FOR_10_YEARS
        ));
        let at_40 = projection
            .points
            .iter()
            .find(|point| point.age == 40)
            .unwrap();
        assert!(close(
            at_40.real,
            MILLION_DEFLATED_BY_6_PERCENT_FOR_10_YEARS
        ));
    }

    #[test]
    fn withdrawals_rise_with_inflation_every_year_of_retirement() {
        let mut inputs = lump_sum(1_000_000.0);
        inputs.retire_age = 31;
        inputs.life_expectancy = 34;
        inputs.inflation = 10.0;
        inputs.desired_income = 1_000.0;
        let projection = project(&inputs).unwrap();
        assert!(close(projection.first_withdrawal, 1_100.0));
        let balance = |age: u32| {
            projection
                .points
                .iter()
                .find(|point| point.age == age)
                .unwrap()
                .nominal
        };
        assert!(close(balance(32), 1_000_000.0 - 12.0 * 1_100.0));
        assert!(close(balance(33), balance(32) - 12.0 * 1_210.0));
        assert!(close(balance(34), balance(33) - 12.0 * 1_331.0));
    }

    #[test]
    fn pot_of_120k_paying_1500_a_month_runs_out_after_80_payments() {
        let mut inputs = lump_sum(120_000.0);
        inputs.desired_income = 1_500.0;
        let projection = project(&inputs).unwrap();
        assert!(close(
            projection.depleted_at.unwrap(),
            RETIRE_40_PLUS_6_YEARS_AND_8_MONTHS
        ));
        assert_eq!(projection.lasts_until(), Some(46));
        assert!(!projection.sustainable);
    }

    #[test]
    fn running_out_exactly_at_the_planned_age_counts_as_sustainable() {
        let mut inputs = lump_sum(120_000.0);
        inputs.desired_income = 1_000.0;
        let exact = project(&inputs).unwrap();
        assert_eq!(exact.depleted_at, Some(50.0));
        assert!(exact.sustainable);
        inputs.life_expectancy = 51;
        assert!(!project(&inputs).unwrap().sustainable);
    }

    #[test]
    fn sustainable_income_matches_the_annuity_formula() {
        let mut inputs = lump_sum(1_000_000.0);
        inputs.return_post = 6.0;
        let monthly = 0.005_f64;
        let annuity = 1_000_000.0 * monthly / (1.0 - (1.0 + monthly).powi(-120));
        assert!(close(
            annuity,
            MILLION_PAID_OUT_OVER_120_MONTHS_AT_HALF_PERCENT_A_MONTH
        ));
        let income = sustainable_income(&inputs).unwrap();
        assert!((income - annuity).abs() < 0.01, "{income} vs {annuity}");
        inputs.desired_income = income;
        assert!(project(&inputs).unwrap().sustainable);
        inputs.desired_income = income + 1.0;
        assert!(!project(&inputs).unwrap().sustainable);
    }

    #[test]
    fn sustainable_income_is_under_one_cent_with_nothing_saved() {
        let income = sustainable_income(&lump_sum(0.0)).unwrap();
        assert!((0.0..0.01).contains(&income), "{income}");
    }

    fn grid() -> Vec<Inputs> {
        let mut cases = Vec::new();
        for savings in [0.0, 100_000.0, 1_000_000.0] {
            for return_pre in [-5.0, 0.0, 6.0, 12.0] {
                for return_post in [-5.0, 0.0, 4.0, 8.0] {
                    for desired_income in [2_000.0, 10_000.0, 40_000.0] {
                        for inflation in [0.0, 5.0] {
                            cases.push(Inputs {
                                current_age: 40,
                                retire_age: 60,
                                life_expectancy: 90,
                                savings,
                                monthly_contribution: 3_000.0,
                                contribution_increase: 5.0,
                                return_pre,
                                return_post,
                                inflation,
                                desired_income,
                            });
                        }
                    }
                }
            }
        }
        cases
    }

    #[test]
    fn higher_return_after_retirement_never_runs_out_sooner() {
        for base in grid() {
            let mut better = base;
            better.return_post += 2.0;
            let lower = project(&base).unwrap();
            let higher = project(&better).unwrap();
            assert!(
                depletion_rank(&higher) >= depletion_rank(&lower),
                "{base:?}"
            );
            assert!(!lower.sustainable || higher.sustainable, "{base:?}");
        }
    }

    #[test]
    fn higher_return_before_retirement_never_shrinks_the_pot() {
        for base in grid() {
            let mut better = base;
            better.return_pre += 2.0;
            let lower = project(&base).unwrap();
            let higher = project(&better).unwrap();
            assert!(higher.pot_nominal >= lower.pot_nominal, "{base:?}");
            assert!(
                depletion_rank(&higher) >= depletion_rank(&lower),
                "{base:?}"
            );
        }
    }

    #[test]
    fn more_savings_never_runs_out_sooner() {
        for base in grid() {
            let mut richer = base;
            richer.savings += 50_000.0;
            let poorer = project(&base).unwrap();
            let wealthier = project(&richer).unwrap();
            assert!(wealthier.pot_nominal >= poorer.pot_nominal, "{base:?}");
            assert!(
                depletion_rank(&wealthier) >= depletion_rank(&poorer),
                "{base:?}"
            );
        }
    }

    #[test]
    fn more_income_never_lasts_longer() {
        for base in grid() {
            let mut hungrier = base;
            hungrier.desired_income += 1_000.0;
            let modest = project(&base).unwrap();
            let lavish = project(&hungrier).unwrap();
            assert!(
                depletion_rank(&lavish) <= depletion_rank(&modest),
                "{base:?}"
            );
        }
    }

    #[test]
    fn sustainable_income_is_the_boundary_on_the_grid() {
        for base in grid().into_iter().step_by(7) {
            let income = sustainable_income(&base).unwrap();
            let mut at = base;
            at.desired_income = income;
            assert!(project(&at).unwrap().sustainable, "{base:?}");
            let mut above = base;
            above.desired_income = income * 1.001 + 1.0;
            assert!(!project(&above).unwrap().sustainable, "{base:?}");
        }
    }
}
