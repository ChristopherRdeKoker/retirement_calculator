use crate::calc::{project, sustainable_income, Inputs, Projection, MAX_AGE};

#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub inputs: Inputs,
    pub projection: Projection,
    pub affordable: Option<f64>,
}

impl Outcome {
    pub fn new(inputs: Inputs) -> Option<Outcome> {
        let projection = project(&inputs).ok()?;
        let affordable = sustainable_income(&inputs);
        Some(Outcome {
            inputs,
            projection,
            affordable,
        })
    }
}

pub fn headline(projection: &Projection) -> String {
    match projection.lasts_until() {
        Some(age) => format!("Your savings last until age {age}"),
        None => format!("Your savings last beyond age {MAX_AGE}"),
    }
}

pub fn lasts_label(projection: &Projection) -> String {
    match projection.lasts_until() {
        Some(age) => format!("Age {age}"),
        None => format!("Beyond {MAX_AGE}"),
    }
}

pub fn lasts_short(projection: &Projection) -> String {
    match projection.lasts_until() {
        Some(age) => age.to_string(),
        None => format!("{MAX_AGE}+"),
    }
}

pub fn sustainability_sentence(projection: &Projection) -> &'static str {
    if projection.sustainable {
        "Your desired income is sustainable."
    } else {
        "Your desired income is not sustainable."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn projection(depleted_at: Option<f64>) -> Projection {
        Projection {
            points: Vec::new(),
            pot_nominal: 0.0,
            pot_real: 0.0,
            first_withdrawal: 0.0,
            depleted_at,
            sustainable: depleted_at.is_none(),
        }
    }

    #[test]
    fn wording_names_the_last_whole_year() {
        let runs_out = projection(Some(84.75));
        assert_eq!(headline(&runs_out), "Your savings last until age 84");
        assert_eq!(lasts_label(&runs_out), "Age 84");
        assert_eq!(lasts_short(&runs_out), "84");
    }

    #[test]
    fn wording_says_beyond_max_age_when_savings_never_run_out() {
        let lasting = projection(None);
        assert_eq!(headline(&lasting), "Your savings last beyond age 100");
        assert_eq!(lasts_label(&lasting), "Beyond 100");
        assert_eq!(lasts_short(&lasting), "100+");
    }

    #[test]
    fn sustainability_sentence_follows_the_projection() {
        assert_eq!(
            sustainability_sentence(&projection(None)),
            "Your desired income is sustainable."
        );
        assert_eq!(
            sustainability_sentence(&projection(Some(70.0))),
            "Your desired income is not sustainable."
        );
    }

    #[test]
    fn invalid_inputs_have_no_outcome() {
        let inputs = Inputs {
            retire_age: 20,
            ..Inputs::default()
        };
        assert_eq!(Outcome::new(inputs), None);
        assert!(Outcome::new(Inputs::default()).is_some());
    }
}
