use super::{group_digits, parse, FieldKind};
use crate::calc::{Deposit, Issue, Subject};

pub const SECTION_TITLE: &str = "Deposit at start of year";
pub const SECTION_HELP: &str = "Add a lump sum you plan to put into your savings, such as a bonus or an inheritance. It is added at the start of the year you choose, so it grows for that whole year.";
pub const YEAR_HELP: &str = "Which year of the projection the deposit goes in. Year 1 starts at your current age. It must be before you retire.";
pub const AMOUNT_HELP: &str =
    "How much you deposit, in rand at the time you pay it in. It is not adjusted for inflation.";
pub const NEW_ROW_YEAR: &str = "1";

#[derive(Clone, Debug, PartialEq)]
pub struct DepositText {
    pub year: String,
    pub amount: String,
}

impl DepositText {
    pub fn of(deposit: &Deposit) -> Self {
        DepositText {
            year: deposit.year.to_string(),
            amount: group_digits(deposit.amount),
        }
    }
}

pub fn read_deposits(rows: &[DepositText]) -> Result<Vec<Deposit>, Vec<Issue>> {
    let mut deposits = Vec::new();
    let mut issues = Vec::new();
    for (row, text) in rows.iter().enumerate() {
        let number = row + 1;
        let year = text.year.trim().parse::<u32>().ok();
        let amount = parse(FieldKind::Money, &text.amount);
        if year.is_none() {
            issues.push(Issue {
                subject: Subject::DepositYear(row),
                message: format!("Deposit {number}: enter the year as a whole number."),
            });
        }
        if amount.is_none() {
            issues.push(Issue {
                subject: Subject::DepositAmount(row),
                message: format!("Deposit {number}: enter the amount as a number."),
            });
        }
        if let (Some(year), Some(amount)) = (year, amount) {
            deposits.push(Deposit { year, amount });
        }
    }
    if issues.is_empty() {
        Ok(deposits)
    } else {
        Err(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(year: &str, amount: &str) -> DepositText {
        DepositText {
            year: year.to_string(),
            amount: amount.to_string(),
        }
    }

    #[test]
    fn help_texts_are_short_sentences() {
        for help in [SECTION_HELP, YEAR_HELP, AMOUNT_HELP] {
            assert!(help.len() <= 200, "{help}");
            assert!(help.ends_with('.'), "{help}");
        }
    }

    #[test]
    fn read_deposits_parses_years_and_rand_amounts() {
        let rows = [text("3", "R 100 000"), text("1", "2,500")];
        assert_eq!(
            read_deposits(&rows),
            Ok(vec![
                Deposit {
                    year: 3,
                    amount: 100_000.0
                },
                Deposit {
                    year: 1,
                    amount: 2_500.0
                },
            ])
        );
    }

    #[test]
    fn read_deposits_reports_each_unreadable_part_by_row() {
        let rows = [text("2", "1000"), text("1.5", ""), text("-1", "5")];
        let issues = read_deposits(&rows).unwrap_err();
        let subjects: Vec<Subject> = issues.iter().map(|issue| issue.subject).collect();
        assert_eq!(
            subjects,
            [
                Subject::DepositYear(1),
                Subject::DepositAmount(1),
                Subject::DepositYear(2)
            ]
        );
        assert_eq!(
            issues[0].message,
            "Deposit 2: enter the year as a whole number."
        );
        assert_eq!(
            issues[1].message,
            "Deposit 2: enter the amount as a number."
        );
    }

    #[test]
    fn deposit_text_round_trips() {
        let deposit = Deposit {
            year: 4,
            amount: 1_250_000.0,
        };
        let shown = DepositText::of(&deposit);
        assert_eq!(shown, text("4", "1 250 000"));
        assert_eq!(read_deposits(&[shown]), Ok(vec![deposit]));
    }
}
