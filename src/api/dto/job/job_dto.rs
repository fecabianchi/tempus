use chrono::NaiveDateTime;
use sea_orm::JsonValue;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateJobRequest {
    #[validate(length(min = 1))]
    pub target: String,
    pub time: NaiveDateTime,
    #[serde(rename = "type")]
    pub job_type: String,
    pub payload: JsonValue,
}

#[derive(Debug, Serialize)]
pub struct CreateJobResponse {
    pub id: Uuid,
    pub message: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateJobTimeRequest {
    pub time: NaiveDateTime,
}
