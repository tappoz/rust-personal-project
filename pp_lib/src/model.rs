use chrono::{DateTime, Utc};
// TODO custom serialize/deserialize: https://serde.rs/custom-date-format.html
use chrono::serde::ts_seconds_option;
use serde::{Deserialize, Serialize};

// derive "Debug", otherwise when calling unwrap:
// ^^^^^^ method cannot be called on `Result<Work, pp_lib::model::Error>` due to unsatisfied trait bounds

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Work {
    pub id: i32,
    pub work_code: String,
    pub add_up_to: i32,
    pub done: bool,
    #[serde(with = "ts_seconds_option")]
    pub created_on: Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option")]
    pub updated_on: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct WorkDemand {
    pub add_up_to: i32,
    pub done: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Error {
    pub http_code: u16,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Event {
    pub id: i32,
    pub work_code: String,
    pub variable: String,
    pub value: String,
    #[serde(with = "ts_seconds_option")]
    pub created_on: Option<DateTime<Utc>>,
}

pub const VAR_COMPUTE_START: &'static str = "compute/start";
pub const VAR_COMPUTE_STOP: &'static str = "compute/stop";
pub const VAR_COMPUTE_RESULT: &'static str = "compute/result";
