#[cfg(test)]
mod fixtures;
pub mod inputs;
pub mod projection;

pub use inputs::{validate, Field, Inputs, Issue, MAX_AGE};
pub use projection::{project, sustainable_income, whole_age, Point, Projection};
