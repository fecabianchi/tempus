use crate::domain::job::r#enum::job_enum::JobType;
use crate::infrastructure::persistence::job::job_metadata::Model;
use crate::infrastructure::persistence::job::sea_orm_active_enums::ScheduleTypeEnum;
use crate::infrastructure::persistence::job::{job, job_metadata};
use chrono::NaiveDateTime;
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveEnum, JsonValue};

#[derive(Debug)]
pub struct JobEntity {
    pub id: Uuid,
    pub time: NaiveDateTime,
    pub target: String,
    pub r#type: JobType,
    pub payload: JsonValue,
    pub metadata: Option<JobMetadata>,
}

#[derive(Debug)]
pub struct JobMetadata {
    pub status: String,
    pub failure: Option<String>,
    pub processed_at: Option<NaiveDateTime>,
}

impl From<(job::Model, Option<Model>)> for JobEntity {
    fn from(tuple: (job::Model, Option<job_metadata::Model>)) -> Self {
        let (job_model, job_metadata_model) = tuple;

        JobEntity {
            id: job_model.id,
            time: job_model.time,
            target: job_model.target,
            r#type: match job_model.r#type {
                ScheduleTypeEnum::Http => JobType::Http,
            },
            payload: job_model.payload,
            metadata: match job_metadata_model {
                None => None,
                Some(job_metadata) => Some(JobMetadata {
                    status: job_metadata.status.to_value().to_string(),
                    failure: job_metadata.failure,
                    processed_at: job_metadata.processed_at,
                }),
            },
        }
    }
}
