use crate::calc::Inputs;
use serde::{Deserialize, Serialize};

pub const MAX_SCENARIOS: usize = 3;
pub const MAX_NAME: usize = 32;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub inputs: Inputs,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SaveOutcome {
    Added(String),
    Updated(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SaveError {
    Full,
}

pub fn clean_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_NAME)
        .collect()
}

fn default_name(list: &[Scenario]) -> String {
    (1..=MAX_SCENARIOS + 1)
        .map(|number| format!("Scenario {number}"))
        .find(|candidate| !list.iter().any(|scenario| &scenario.name == candidate))
        .unwrap_or_else(|| "Scenario".to_string())
}

pub fn upsert(
    list: &mut Vec<Scenario>,
    name: &str,
    inputs: Inputs,
) -> Result<SaveOutcome, SaveError> {
    let mut label = clean_name(name);
    if label.is_empty() {
        label = default_name(list);
    }
    if let Some(existing) = list.iter_mut().find(|scenario| scenario.name == label) {
        existing.inputs = inputs;
        return Ok(SaveOutcome::Updated(label));
    }
    if list.len() >= MAX_SCENARIOS {
        return Err(SaveError::Full);
    }
    list.push(Scenario {
        name: label.clone(),
        inputs,
    });
    Ok(SaveOutcome::Added(label))
}

pub fn without(list: Vec<Scenario>, name: &str) -> Vec<Scenario> {
    list.into_iter()
        .filter(|scenario| scenario.name != name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn retiring_at(retire_age: u32) -> Inputs {
        Inputs {
            retire_age,
            ..Inputs::default()
        }
    }

    fn names(list: &[Scenario]) -> Vec<&str> {
        list.iter().map(|scenario| scenario.name.as_str()).collect()
    }

    #[test]
    fn clean_name_collapses_whitespace_and_caps_length() {
        assert_eq!(clean_name("  Retire   at\t60 \n"), "Retire at 60");
        assert_eq!(clean_name("   "), "");
        assert_eq!(clean_name(&"é".repeat(40)), "é".repeat(MAX_NAME));
    }

    #[test]
    fn upsert_adds_a_new_scenario_under_its_cleaned_name() {
        let mut list = Vec::new();
        let outcome = upsert(&mut list, "  Early   exit ", retiring_at(55));
        assert_eq!(outcome, Ok(SaveOutcome::Added("Early exit".to_string())));
        assert_eq!(
            list,
            vec![Scenario {
                name: "Early exit".to_string(),
                inputs: retiring_at(55)
            }]
        );
    }

    #[test]
    fn upsert_names_an_unnamed_scenario_after_the_first_free_number() {
        let mut list = Vec::new();
        assert_eq!(
            upsert(&mut list, "", retiring_at(60)),
            Ok(SaveOutcome::Added("Scenario 1".to_string()))
        );
        assert_eq!(
            upsert(&mut list, "Scenario 3", retiring_at(61)),
            Ok(SaveOutcome::Added("Scenario 3".to_string()))
        );
        assert_eq!(
            upsert(&mut list, "   ", retiring_at(62)),
            Ok(SaveOutcome::Added("Scenario 2".to_string()))
        );
        assert_eq!(names(&list), ["Scenario 1", "Scenario 3", "Scenario 2"]);
    }

    #[test]
    fn default_name_falls_back_when_every_number_is_taken() {
        let list: Vec<Scenario> = (1..=MAX_SCENARIOS + 1)
            .map(|number| Scenario {
                name: format!("Scenario {number}"),
                inputs: Inputs::default(),
            })
            .collect();
        assert_eq!(default_name(&list), "Scenario");
    }

    #[test]
    fn upsert_updates_a_scenario_with_the_same_name_in_place() {
        let mut list = Vec::new();
        upsert(&mut list, "A", retiring_at(60)).unwrap();
        upsert(&mut list, "B", retiring_at(61)).unwrap();
        assert_eq!(
            upsert(&mut list, " A ", retiring_at(70)),
            Ok(SaveOutcome::Updated("A".to_string()))
        );
        assert_eq!(names(&list), ["A", "B"]);
        assert_eq!(list[0].inputs, retiring_at(70));
    }

    #[test]
    fn upsert_refuses_a_new_name_when_full_but_still_updates() {
        let mut list = Vec::new();
        for name in ["A", "B", "C"] {
            upsert(&mut list, name, retiring_at(60)).unwrap();
        }
        assert_eq!(
            upsert(&mut list, "D", retiring_at(61)),
            Err(SaveError::Full)
        );
        assert_eq!(names(&list), ["A", "B", "C"]);
        assert_eq!(
            upsert(&mut list, "B", retiring_at(62)),
            Ok(SaveOutcome::Updated("B".to_string()))
        );
        assert_eq!(list[1].inputs, retiring_at(62));
    }

    #[test]
    fn without_drops_only_the_named_scenario() {
        let mut list = Vec::new();
        for name in ["A", "B", "C"] {
            upsert(&mut list, name, retiring_at(60)).unwrap();
        }
        assert_eq!(names(&without(list, "B")), ["A", "C"]);
    }
}
