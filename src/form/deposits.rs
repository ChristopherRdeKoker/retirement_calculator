use super::{group_digits, parse, FieldKind, FormText};
use crate::calc::{Deposit, Inputs, Issue, Subject};
use leptos::prelude::*;

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

#[derive(Clone, Copy)]
pub struct DepositRow {
    pub key: usize,
    pub year: RwSignal<String>,
    pub amount: RwSignal<String>,
}

impl FormText {
    pub fn deposits(self) -> RwSignal<Vec<DepositRow>> {
        self.deposits
    }

    pub fn deposits_open(self) -> RwSignal<bool> {
        self.deposits_open
    }

    fn row(self, text: DepositText) -> DepositRow {
        let key = self.next_key.get_value();
        self.next_key.set_value(key + 1);
        DepositRow {
            key,
            year: RwSignal::new(text.year),
            amount: RwSignal::new(text.amount),
        }
    }

    pub(super) fn rows_for(self, inputs: &Inputs) -> Vec<DepositRow> {
        inputs
            .deposits
            .iter()
            .map(|deposit| self.row(DepositText::of(deposit)))
            .collect()
    }

    pub fn add_deposit(self) {
        let row = self.row(DepositText {
            year: NEW_ROW_YEAR.to_string(),
            amount: String::new(),
        });
        self.deposits.update(|rows| rows.push(row));
    }

    pub fn remove_deposit(self, key: usize) {
        self.deposits
            .update(|rows| rows.retain(|row| row.key != key));
    }

    pub(super) fn deposit_texts(self) -> Vec<DepositText> {
        self.deposits
            .get()
            .iter()
            .map(|row| DepositText {
                year: row.year.get(),
                amount: row.amount.get(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::Field;
    use crate::form::{field_text, read_form};

    fn texts(inputs: &Inputs) -> impl Fn(Field) -> String + '_ {
        move |field| field_text(field, inputs)
    }

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
    #[test]
    fn read_form_adds_deposit_rows_and_reports_them_after_fields() {
        let defaults = Inputs::default();
        let rows = [DepositText {
            year: "3".to_string(),
            amount: "R 100 000".to_string(),
        }];
        let read = read_form(texts(&defaults), &rows).unwrap();
        assert_eq!(
            read.deposits,
            [Deposit {
                year: 3,
                amount: 100_000.0
            }]
        );
        let bad_rows = [DepositText {
            year: "3".to_string(),
            amount: String::new(),
        }];
        let text_of = |field: Field| match field {
            Field::Savings => String::new(),
            _ => field_text(field, &defaults),
        };
        let subjects: Vec<Subject> = read_form(text_of, &bad_rows)
            .unwrap_err()
            .into_iter()
            .map(|issue| issue.subject)
            .collect();
        assert_eq!(
            subjects,
            [Subject::Field(Field::Savings), Subject::DepositAmount(0)]
        );
    }

    #[test]
    fn read_form_blocks_a_deposit_after_retirement() {
        let defaults = Inputs::default();
        let rows = [DepositText {
            year: "31".to_string(),
            amount: "1 000".to_string(),
        }];
        let issues = read_form(texts(&defaults), &rows).unwrap_err();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].subject, Subject::DepositYear(0));
    }
}
