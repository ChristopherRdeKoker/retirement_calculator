use crate::calc::{validate, Inputs};
use crate::scenarios::{clean_name, Scenario, MAX_SCENARIOS};

const INPUTS_KEY: &str = "retire-calc.inputs.v1";
const SCENARIOS_KEY: &str = "retire-calc.scenarios.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loaded {
    Fresh,
    Valid,
    Repaired,
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

pub fn available() -> bool {
    storage().is_some()
}

fn read_raw(key: &str) -> Option<String> {
    storage()?.get_item(key).ok().flatten()
}

fn write_raw(key: &str, value: &str) -> bool {
    match storage() {
        Some(storage) => storage.set_item(key, value).is_ok(),
        None => false,
    }
}

pub fn parse_inputs(raw: &str) -> (Inputs, Loaded) {
    match serde_json::from_str::<Inputs>(raw) {
        Ok(inputs) if validate(&inputs).is_empty() => (inputs, Loaded::Valid),
        _ => (Inputs::default(), Loaded::Repaired),
    }
}

pub fn parse_scenarios(raw: &str) -> (Vec<Scenario>, Loaded) {
    let Ok(list) = serde_json::from_str::<Vec<serde_json::Value>>(raw) else {
        return (Vec::new(), Loaded::Repaired);
    };
    let total = list.len();
    let mut kept: Vec<Scenario> = Vec::new();
    for value in list {
        let Ok(mut scenario) = serde_json::from_value::<Scenario>(value) else {
            continue;
        };
        scenario.name = clean_name(&scenario.name);
        let is_keepable = !scenario.name.is_empty()
            && validate(&scenario.inputs).is_empty()
            && !kept.iter().any(|existing| existing.name == scenario.name)
            && kept.len() < MAX_SCENARIOS;
        if is_keepable {
            kept.push(scenario);
        }
    }
    let state = if kept.len() == total {
        Loaded::Valid
    } else {
        Loaded::Repaired
    };
    (kept, state)
}

pub fn load_inputs() -> (Inputs, Loaded) {
    match read_raw(INPUTS_KEY) {
        Some(raw) => parse_inputs(&raw),
        None => (Inputs::default(), Loaded::Fresh),
    }
}

pub fn load_scenarios() -> (Vec<Scenario>, Loaded) {
    match read_raw(SCENARIOS_KEY) {
        Some(raw) => parse_scenarios(&raw),
        None => (Vec::new(), Loaded::Fresh),
    }
}

pub fn save_inputs(inputs: &Inputs) -> bool {
    match serde_json::to_string(inputs) {
        Ok(json) => write_raw(INPUTS_KEY, &json),
        Err(_) => false,
    }
}

pub fn save_scenarios(list: &[Scenario]) -> bool {
    match serde_json::to_string(list) {
        Ok(json) => write_raw(SCENARIOS_KEY, &json),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::Deposit;

    fn scenario_json(name: &str, inputs: &Inputs) -> serde_json::Value {
        serde_json::json!({ "name": name, "inputs": inputs })
    }

    fn names(list: &[Scenario]) -> Vec<&str> {
        list.iter().map(|scenario| scenario.name.as_str()).collect()
    }

    #[test]
    fn saved_inputs_round_trip() {
        let inputs = Inputs {
            retire_age: 60,
            savings: 1_234.5,
            ..Inputs::default()
        };
        let json = serde_json::to_string(&inputs).unwrap();
        assert_eq!(parse_inputs(&json), (inputs, Loaded::Valid));
    }

    #[test]
    fn broken_or_invalid_inputs_fall_back_to_defaults() {
        assert_eq!(
            parse_inputs("{not json"),
            (Inputs::default(), Loaded::Repaired)
        );
        let invalid = Inputs {
            retire_age: 20,
            ..Inputs::default()
        };
        let json = serde_json::to_string(&invalid).unwrap();
        assert_eq!(parse_inputs(&json), (Inputs::default(), Loaded::Repaired));
        assert_eq!(
            parse_inputs(r#"{"current_age":35}"#),
            (Inputs::default(), Loaded::Repaired)
        );
    }

    #[test]
    fn valid_scenarios_load_unchanged() {
        let raw = serde_json::json!([
            scenario_json("A", &Inputs::default()),
            scenario_json("B", &Inputs::default()),
        ])
        .to_string();
        let (list, state) = parse_scenarios(&raw);
        assert_eq!(names(&list), ["A", "B"]);
        assert_eq!(state, Loaded::Valid);
    }

    #[test]
    fn scenarios_that_are_not_a_list_are_reset() {
        assert_eq!(
            parse_scenarios("\"a string\""),
            (Vec::new(), Loaded::Repaired)
        );
        assert_eq!(parse_scenarios("{not json"), (Vec::new(), Loaded::Repaired));
    }

    #[test]
    fn bad_entries_are_dropped_and_reported() {
        let invalid = Inputs {
            current_age: 500,
            ..Inputs::default()
        };
        let raw = serde_json::json!([
            scenario_json("  Keep   me ", &Inputs::default()),
            scenario_json("Bad inputs", &invalid),
            scenario_json("   ", &Inputs::default()),
            scenario_json("Keep me", &Inputs::default()),
            7,
            null,
            "x",
        ])
        .to_string();
        let (list, state) = parse_scenarios(&raw);
        assert_eq!(names(&list), ["Keep me"]);
        assert_eq!(state, Loaded::Repaired);
    }

    #[test]
    fn scenarios_past_the_limit_are_dropped() {
        let raw = serde_json::Value::Array(
            ["A", "B", "C", "D"]
                .iter()
                .map(|name| scenario_json(name, &Inputs::default()))
                .collect(),
        )
        .to_string();
        let (list, state) = parse_scenarios(&raw);
        assert_eq!(names(&list), ["A", "B", "C"]);
        assert_eq!(state, Loaded::Repaired);
    }

    const OLD_INPUTS_WITHOUT_DEPOSITS: &str = r#"{"current_age":35,"retire_age":65,"life_expectancy":90,"savings":250000.0,"monthly_contribution":5000.0,"contribution_increase":5.0,"return_pre":10.0,"return_post":7.0,"inflation":5.0,"desired_income":25000.0}"#;

    fn with_deposits(deposits: Vec<Deposit>) -> Inputs {
        Inputs {
            deposits,
            ..Inputs::default()
        }
    }

    #[test]
    fn inputs_saved_before_deposits_existed_load_with_no_deposits() {
        assert_eq!(
            parse_inputs(OLD_INPUTS_WITHOUT_DEPOSITS),
            (Inputs::default(), Loaded::Valid)
        );
        assert_eq!(
            serde_json::to_string(&Inputs::default()).unwrap(),
            OLD_INPUTS_WITHOUT_DEPOSITS
        );
    }

    #[test]
    fn scenarios_saved_before_deposits_existed_load_with_no_deposits() {
        let raw = format!(r#"[{{"name":"Old plan","inputs":{OLD_INPUTS_WITHOUT_DEPOSITS}}}]"#);
        let (list, state) = parse_scenarios(&raw);
        assert_eq!(state, Loaded::Valid);
        assert_eq!(names(&list), ["Old plan"]);
        assert!(list[0].inputs.deposits.is_empty());
        assert_eq!(list[0].inputs, Inputs::default());
    }

    #[test]
    fn scenario_deposits_round_trip() {
        let inputs = with_deposits(vec![
            Deposit {
                year: 3,
                amount: 100_000.0,
            },
            Deposit {
                year: 3,
                amount: 5_000.0,
            },
        ]);
        let raw = serde_json::json!([scenario_json("Bonus", &inputs)]).to_string();
        let (list, state) = parse_scenarios(&raw);
        assert_eq!(state, Loaded::Valid);
        assert_eq!(list[0].inputs, inputs);
        let json = serde_json::to_string(&inputs).unwrap();
        assert!(json
            .ends_with(r#""deposits":[{"year":3,"amount":100000.0},{"year":3,"amount":5000.0}]}"#));
        assert_eq!(parse_inputs(&json), (inputs, Loaded::Valid));
    }

    #[test]
    fn scenario_with_an_invalid_deposit_is_dropped() {
        let late = with_deposits(vec![Deposit {
            year: 31,
            amount: 1_000.0,
        }]);
        let raw = serde_json::json!([
            scenario_json("Late", &late),
            scenario_json("Fine", &Inputs::default()),
        ])
        .to_string();
        let (list, state) = parse_scenarios(&raw);
        assert_eq!(names(&list), ["Fine"]);
        assert_eq!(state, Loaded::Repaired);
    }
}
