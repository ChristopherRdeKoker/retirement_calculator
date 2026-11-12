use crate::calc::{validate, Field, Inputs, Issue, Subject};
use crate::format::group_thousands;
use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKind {
    Age,
    Money,
    Percent,
}

pub struct FieldDef {
    pub id: &'static str,
    pub label: &'static str,
    pub hint: &'static str,
    pub help: &'static str,
    pub kind: FieldKind,
}

pub fn field_def(field: Field) -> FieldDef {
    match field {
        Field::CurrentAge => FieldDef {
            id: "current-age",
            label: "Current age",
            hint: "",
            help: "Your age today, in whole years. The younger you are, the more years your savings have to grow.",
            kind: FieldKind::Age,
        },
        Field::RetireAge => FieldDef {
            id: "retire-age",
            label: "Retirement age",
            hint: "",
            help: "The age you plan to stop working and start living off your savings. Each extra year of work means one more year of saving and one less year of drawing.",
            kind: FieldKind::Age,
        },
        Field::LifeExpectancy => FieldDef {
            id: "life-expectancy",
            label: "Plan until age",
            hint: "Life expectancy, up to 100",
            help: "How long your money needs to last. Plan for a long life, as running out at 85 is much worse than having some left over.",
            kind: FieldKind::Age,
        },
        Field::Savings => FieldDef {
            id: "savings",
            label: "Current savings",
            hint: "Retirement annuity, pension or provident fund",
            help: "What your retirement savings are worth today, such as a retirement annuity, pension or provident fund. Leave out money you plan to spend before you retire.",
            kind: FieldKind::Money,
        },
        Field::MonthlyContribution => FieldDef {
            id: "contribution",
            label: "Monthly contribution",
            hint: "",
            help: "How much goes into your retirement savings each month, including anything your employer adds for you.",
            kind: FieldKind::Money,
        },
        Field::ContributionIncrease => FieldDef {
            id: "increase",
            label: "Yearly contribution increase",
            hint: "In line with salary increases",
            help: "How much you raise your monthly contribution each year, for example when you get a salary increase. Raising it in line with inflation keeps what you save worth the same in today's money.",
            kind: FieldKind::Percent,
        },
        Field::ReturnPre => FieldDef {
            id: "return-pre",
            label: "Return before retirement",
            hint: "Per year, before inflation",
            help: "How much your savings grow each year before you retire, on average, before inflation. Balanced funds have often averaged around 8–10%; this is an assumption, not a promise.",
            kind: FieldKind::Percent,
        },
        Field::ReturnPost => FieldDef {
            id: "return-post",
            label: "Return after retirement",
            hint: "Usually lower: a more cautious mix",
            help: "How much your savings grow each year once you have retired, before inflation. It is usually lower, because retirees often move to a more cautious mix with fewer shares.",
            kind: FieldKind::Percent,
        },
        Field::Inflation => FieldDef {
            id: "inflation",
            label: "Inflation",
            hint: "SARB target range is 3–6%",
            help: "How fast prices rise each year, on average. It shrinks what your rand can buy, so the calculator uses it to show amounts in today's money.",
            kind: FieldKind::Percent,
        },
        Field::DesiredIncome => FieldDef {
            id: "income",
            label: "Monthly income in retirement",
            hint: "In today's money, before tax",
            help: "The monthly income you want once you retire, before tax, in today's money. The calculator raises it with inflation each year so it keeps buying the same.",
            kind: FieldKind::Money,
        },
    }
}

pub const FIELD_GROUPS: [(&str, &[Field]); 4] = [
    (
        "About you",
        &[Field::CurrentAge, Field::RetireAge, Field::LifeExpectancy],
    ),
    (
        "Saving",
        &[
            Field::Savings,
            Field::MonthlyContribution,
            Field::ContributionIncrease,
        ],
    ),
    (
        "Growth and inflation",
        &[Field::ReturnPre, Field::ReturnPost, Field::Inflation],
    ),
    ("Retirement income", &[Field::DesiredIncome]),
];

pub fn group_digits(value: f64) -> String {
    group_thousands(&format!("{:.0}", value.abs().round()), ' ')
}

pub fn field_text(field: Field, inputs: &Inputs) -> String {
    let value = inputs.value(field);
    match field_def(field).kind {
        FieldKind::Money => group_digits(value),
        FieldKind::Age | FieldKind::Percent => value.to_string(),
    }
}

pub fn parse(kind: FieldKind, raw: &str) -> Option<f64> {
    let cleaned: String = raw
        .chars()
        .filter(|character| {
            !character.is_whitespace() && !matches!(character, 'R' | 'r' | '%' | '\u{a0}')
        })
        .collect();
    let normal = match kind {
        FieldKind::Money => cleaned.replace(',', ""),
        _ => cleaned.replace(',', "."),
    };
    if normal.is_empty() {
        return None;
    }
    let value: f64 = normal.parse().ok()?;
    if !value.is_finite() {
        return None;
    }
    if kind == FieldKind::Age && (value.fract() != 0.0 || !(0.0..=200.0).contains(&value)) {
        return None;
    }
    Some(value)
}

fn unreadable(field: Field) -> Issue {
    let def = field_def(field);
    let message = match def.kind {
        FieldKind::Age => format!("{}: enter a whole number of years.", def.label),
        _ => format!("{}: enter a number.", def.label),
    };
    Issue {
        subject: Subject::Field(field),
        message,
    }
}

pub fn read_form(text_of: impl Fn(Field) -> String) -> Result<Inputs, Vec<Issue>> {
    let mut inputs = Inputs::default();
    let mut issues = Vec::new();
    for field in Field::ALL {
        match parse(field_def(field).kind, &text_of(field)) {
            Some(value) => inputs.set_value(field, value),
            None => issues.push(unreadable(field)),
        }
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    let issues = validate(&inputs);
    if issues.is_empty() {
        Ok(inputs)
    } else {
        Err(issues)
    }
}

#[derive(Clone, Copy)]
pub struct FormText([RwSignal<String>; Field::ALL.len()]);

impl FormText {
    pub fn new(inputs: &Inputs) -> Self {
        FormText(Field::ALL.map(|field| RwSignal::new(field_text(field, inputs))))
    }

    pub fn signal(self, field: Field) -> RwSignal<String> {
        self.0[field.index()]
    }

    pub fn fill(self, inputs: &Inputs) {
        for field in Field::ALL {
            self.signal(field).set(field_text(field, inputs));
        }
    }

    pub fn read(self) -> Result<Inputs, Vec<Issue>> {
        read_form(|field| self.signal(field).get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(inputs: &Inputs) -> impl Fn(Field) -> String + '_ {
        move |field| field_text(field, inputs)
    }

    #[test]
    fn every_field_appears_in_exactly_one_group() {
        let grouped: Vec<Field> = FIELD_GROUPS
            .iter()
            .flat_map(|(_, fields)| fields.iter().copied())
            .collect();
        assert_eq!(grouped, Field::ALL);
    }

    #[test]
    fn every_field_has_short_help() {
        for field in Field::ALL {
            let help = field_def(field).help;
            assert!(!help.trim().is_empty(), "{field:?} has no help");
            assert!(help.len() <= 200, "{field:?} help is too long");
            assert!(help.ends_with('.'), "{field:?} help is not a sentence");
        }
    }

    #[test]
    fn field_ids_are_unique() {
        let mut ids: Vec<&str> = Field::ALL.map(|field| field_def(field).id).to_vec();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), Field::ALL.len());
    }

    #[test]
    fn parse_reads_money_with_separators_and_prefix() {
        assert_eq!(parse(FieldKind::Money, "R 1 234 567"), Some(1_234_567.0));
        assert_eq!(parse(FieldKind::Money, "1,234,567"), Some(1_234_567.0));
        assert_eq!(parse(FieldKind::Money, "1\u{a0}000"), Some(1_000.0));
        assert_eq!(parse(FieldKind::Money, "12.5"), Some(12.5));
        assert_eq!(parse(FieldKind::Money, "-5"), Some(-5.0));
    }

    #[test]
    fn parse_reads_percent_with_comma_decimal() {
        assert_eq!(parse(FieldKind::Percent, "7,5"), Some(7.5));
        assert_eq!(parse(FieldKind::Percent, "7.5 %"), Some(7.5));
        assert_eq!(parse(FieldKind::Percent, "-3"), Some(-3.0));
    }

    #[test]
    fn parse_accepts_only_whole_ages_up_to_200() {
        assert_eq!(parse(FieldKind::Age, "35"), Some(35.0));
        assert_eq!(parse(FieldKind::Age, "200"), Some(200.0));
        assert_eq!(parse(FieldKind::Age, "201"), None);
        assert_eq!(parse(FieldKind::Age, "30.5"), None);
        assert_eq!(parse(FieldKind::Age, "-1"), None);
    }

    #[test]
    fn parse_rejects_empty_and_non_numbers() {
        assert_eq!(parse(FieldKind::Money, ""), None);
        assert_eq!(parse(FieldKind::Money, "   "), None);
        assert_eq!(parse(FieldKind::Money, "abc"), None);
        assert_eq!(parse(FieldKind::Percent, "inf"), None);
        assert_eq!(parse(FieldKind::Percent, "NaN"), None);
    }

    #[test]
    fn parse_strips_r_and_percent_anywhere() {
        assert_eq!(parse(FieldKind::Money, "1r2"), Some(12.0));
        assert_eq!(parse(FieldKind::Percent, "1%2"), Some(12.0));
    }

    #[test]
    fn group_digits_uses_plain_spaces_and_drops_the_sign() {
        assert_eq!(group_digits(1_234_567.0), "1 234 567");
        assert_eq!(group_digits(999.6), "1 000");
        assert_eq!(group_digits(0.0), "0");
        assert_eq!(group_digits(-2_500.0), "2 500");
    }

    #[test]
    fn field_text_lists_fields_in_form_order() {
        let defaults = Inputs::default();
        assert_eq!(
            Field::ALL.map(|field| field_text(field, &defaults)),
            ["35", "65", "90", "250 000", "5 000", "5", "10", "7", "5", "25 000"].map(String::from)
        );
    }

    #[test]
    fn read_form_round_trips_field_text() {
        let inputs = Inputs {
            current_age: 41,
            retire_age: 63,
            life_expectancy: 95,
            savings: 1_234_567.0,
            monthly_contribution: 8_000.0,
            contribution_increase: 4.5,
            return_pre: 9.25,
            return_post: -2.0,
            inflation: 5.5,
            desired_income: 30_000.0,
            deposits: Vec::new(),
        };
        assert_eq!(read_form(texts(&inputs)), Ok(inputs));
        assert_eq!(read_form(texts(&Inputs::default())), Ok(Inputs::default()));
    }

    #[test]
    fn read_form_reports_unreadable_fields_in_order() {
        let defaults = Inputs::default();
        let text_of = |field: Field| match field {
            Field::CurrentAge => "thirty".to_string(),
            Field::Savings => String::new(),
            _ => field_text(field, &defaults),
        };
        assert_eq!(
            read_form(text_of),
            Err(vec![
                Issue {
                    subject: Subject::Field(Field::CurrentAge),
                    message: "Current age: enter a whole number of years.".to_string(),
                },
                Issue {
                    subject: Subject::Field(Field::Savings),
                    message: "Current savings: enter a number.".to_string(),
                },
            ])
        );
    }

    #[test]
    fn read_form_passes_on_validation_issues() {
        let defaults = Inputs::default();
        let text_of = |field: Field| match field {
            Field::RetireAge => "30".to_string(),
            _ => field_text(field, &defaults),
        };
        let issues = read_form(text_of).unwrap_err();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].subject, Subject::Field(Field::RetireAge));
    }
}
