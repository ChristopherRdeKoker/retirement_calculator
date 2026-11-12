use crate::calc::{validate, Inputs};
use crate::scenarios::{clean_name, Scenario, MAX_SCENARIOS};

const INPUTS_KEY: &str = "retire-calc.inputs.v1";
const SCENARIOS_KEY: &str = "retire-calc.scenarios.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loaded {
    Fresh,
    Ok,
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
        Some(s) => s.set_item(key, value).is_ok(),
        None => false,
    }
}

pub fn load_inputs() -> (Inputs, Loaded) {
    let Some(raw) = read_raw(INPUTS_KEY) else {
        return (Inputs::default(), Loaded::Fresh);
    };
    match serde_json::from_str::<Inputs>(&raw) {
        Ok(inputs) if validate(&inputs).is_empty() => (inputs, Loaded::Ok),
        _ => (Inputs::default(), Loaded::Repaired),
    }
}

pub fn save_inputs(i: &Inputs) -> bool {
    match serde_json::to_string(i) {
        Ok(json) => write_raw(INPUTS_KEY, &json),
        Err(_) => false,
    }
}

pub fn load_scenarios() -> (Vec<Scenario>, Loaded) {
    let Some(raw) = read_raw(SCENARIOS_KEY) else {
        return (Vec::new(), Loaded::Fresh);
    };
    let Ok(list) = serde_json::from_str::<Vec<serde_json::Value>>(&raw) else {
        return (Vec::new(), Loaded::Repaired);
    };
    let total = list.len();
    let mut kept: Vec<Scenario> = Vec::new();
    for value in list {
        if let Ok(mut s) = serde_json::from_value::<Scenario>(value) {
            s.name = clean_name(&s.name);
            if !s.name.is_empty()
                && validate(&s.inputs).is_empty()
                && !kept
                    .iter()
                    .any(|kept_scenario| kept_scenario.name == s.name)
                && kept.len() < MAX_SCENARIOS
            {
                kept.push(s);
            }
        }
    }
    let state = if kept.len() == total {
        Loaded::Ok
    } else {
        Loaded::Repaired
    };
    (kept, state)
}

pub fn save_scenarios(list: &[Scenario]) -> bool {
    match serde_json::to_string(list) {
        Ok(json) => write_raw(SCENARIOS_KEY, &json),
        Err(_) => false,
    }
}
