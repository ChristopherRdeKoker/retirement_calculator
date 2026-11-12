use super::fixtures::{close, flat};
use super::inputs::{validate, Deposit, Inputs, Subject};
use super::projection::{project, sustainable_income, Projection};

const DEFAULTS_POT: f64 = 22_775_983.101_253;
const DEFAULTS_POT_TODAY: f64 = 5_269_848.860_597;
const DEFAULTS_FIRST_WITHDRAWAL: f64 = 108_048.559_379;
const DEFAULTS_RUN_OUT_AT_87_AND_8_MONTHS: f64 = 87.0 + 8.0 / 12.0;
const DEFAULTS_BALANCE_AT_45: f64 = 1_914_812.676_162;
const DEFAULTS_SUSTAINABLE_INCOME: f64 = 23_201.343_509;
const HUNDRED_K_AT_1_PERCENT_A_MONTH_FOR_12_MONTHS: f64 = 112_682.503_013_197;
const HUNDRED_K_AT_1_PERCENT_A_MONTH_FOR_36_MONTHS: f64 = 143_076.878_359_158;

fn deposit(year: u32, amount: f64) -> Deposit {
    Deposit { year, amount }
}

fn growing_lump(retire_age: u32, deposits: Vec<Deposit>) -> Inputs {
    Inputs {
        retire_age,
        monthly_contribution: 0.0,
        return_pre: 12.0,
        deposits,
        ..flat()
    }
}

fn balance_at(projection: &Projection, age: u32) -> f64 {
    projection
        .points
        .iter()
        .find(|point| point.age == age)
        .unwrap()
        .nominal
}

#[test]
fn no_deposits_give_the_same_results_as_before_deposits_existed() {
    let inputs = Inputs::default();
    assert!(inputs.deposits.is_empty());
    let projection = project(&inputs).unwrap();
    assert!(close(projection.pot_nominal, DEFAULTS_POT));
    assert!(close(projection.pot_real, DEFAULTS_POT_TODAY));
    assert!(close(
        projection.first_withdrawal,
        DEFAULTS_FIRST_WITHDRAWAL
    ));
    assert!(close(
        projection.depleted_at.unwrap(),
        DEFAULTS_RUN_OUT_AT_87_AND_8_MONTHS
    ));
    assert!(!projection.sustainable);
    assert!(close(balance_at(&projection, 45), DEFAULTS_BALANCE_AT_45));
    let income = sustainable_income(&inputs).unwrap();
    assert!(close(income, DEFAULTS_SUSTAINABLE_INCOME), "{income}");
}

#[test]
fn deposit_in_year_one_earns_the_whole_first_year() {
    let projection = project(&growing_lump(31, vec![deposit(1, 100_000.0)])).unwrap();
    assert!(
        close(
            projection.pot_nominal,
            HUNDRED_K_AT_1_PERCENT_A_MONTH_FOR_12_MONTHS
        ),
        "{}",
        projection.pot_nominal
    );
    assert!(close(balance_at(&projection, 30), 100_000.0));
}

#[test]
fn deposit_in_year_three_grows_for_one_year_and_ends_below_year_one() {
    let early = project(&growing_lump(33, vec![deposit(1, 100_000.0)])).unwrap();
    let late = project(&growing_lump(33, vec![deposit(3, 100_000.0)])).unwrap();
    assert!(close(
        early.pot_nominal,
        HUNDRED_K_AT_1_PERCENT_A_MONTH_FOR_36_MONTHS
    ));
    assert!(close(
        late.pot_nominal,
        HUNDRED_K_AT_1_PERCENT_A_MONTH_FOR_12_MONTHS
    ));
    assert!(early.pot_nominal > late.pot_nominal);
    assert_eq!(balance_at(&late, 31), 0.0);
    assert!(close(balance_at(&late, 32), 100_000.0));
}

#[test]
fn two_deposits_in_one_year_equal_one_deposit_of_their_sum() {
    let split = growing_lump(35, vec![deposit(3, 40_000.0), deposit(3, 60_000.0)]);
    let single = growing_lump(35, vec![deposit(3, 100_000.0)]);
    assert_eq!(project(&split).unwrap(), project(&single).unwrap());
    assert_eq!(sustainable_income(&split), sustainable_income(&single));
}

#[test]
fn at_zero_return_a_deposit_simply_adds_its_amount() {
    let inputs = Inputs {
        deposits: vec![deposit(5, 25_000.0)],
        ..flat()
    };
    let projection = project(&inputs).unwrap();
    assert!(close(projection.pot_nominal, 120_000.0 + 25_000.0));
    assert!(close(balance_at(&projection, 33), 36_000.0));
    assert!(close(balance_at(&projection, 34), 48_000.0 + 25_000.0));
    assert!(close(projection.depleted_at.unwrap(), 40.0 + 145.0 / 12.0));
}

#[test]
fn sustainable_income_includes_deposits() {
    let inputs = Inputs {
        deposits: vec![deposit(1, 12_000.0)],
        ..flat()
    };
    let income = sustainable_income(&inputs).unwrap();
    assert!((income - 1_100.0).abs() < 0.01, "{income}");
}

fn subjects(inputs: &Inputs) -> Vec<Subject> {
    validate(inputs)
        .into_iter()
        .map(|issue| issue.subject)
        .collect()
}

#[test]
fn deposit_year_must_be_from_one_up_to_the_last_year_before_retirement() {
    let with = |year: u32| Inputs {
        deposits: vec![deposit(year, 1_000.0)],
        ..flat()
    };
    assert_eq!(subjects(&with(0)), [Subject::DepositYear(0)]);
    assert_eq!(subjects(&with(11)), [Subject::DepositYear(0)]);
    assert_eq!(
        validate(&with(11))[0].message,
        "Deposit 1: the year must be between 1 and 10, before you retire."
    );
    assert!(subjects(&with(10)).is_empty());
    assert!(subjects(&with(1)).is_empty());
    assert!(close(project(&with(10)).unwrap().pot_nominal, 121_000.0));
    assert!(project(&with(0)).is_err());
    assert!(project(&with(11)).is_err());
    assert!(sustainable_income(&with(11)).is_none());
}

#[test]
fn deposit_amount_must_be_more_than_zero_and_finite() {
    let with = |amount: f64| Inputs {
        deposits: vec![deposit(2, 5_000.0), deposit(3, amount)],
        ..flat()
    };
    for amount in [0.0, -1.0, f64::NAN, f64::INFINITY, 2e12] {
        assert_eq!(
            subjects(&with(amount)),
            [Subject::DepositAmount(1)],
            "{amount}"
        );
        assert!(project(&with(amount)).is_err());
    }
    assert_eq!(
        validate(&with(0.0))[0].message,
        "Deposit 2: the amount must be more than R 0."
    );
    assert!(subjects(&with(0.01)).is_empty());
}

#[test]
fn bad_ages_do_not_add_a_confusing_deposit_year_issue() {
    let inputs = Inputs {
        retire_age: 30,
        deposits: vec![deposit(4, 1_000.0)],
        ..flat()
    };
    assert!(!subjects(&inputs).contains(&Subject::DepositYear(0)));
}
