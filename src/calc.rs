#[cfg(test)]
mod deposit_tests;
#[cfg(test)]
mod fixtures;
pub mod inputs;
pub mod projection;

pub use inputs::{validate, Deposit, Field, Inputs, Issue, Subject, MAX_AGE};
pub use projection::{project, sustainable_income, whole_age, Point, Projection};
