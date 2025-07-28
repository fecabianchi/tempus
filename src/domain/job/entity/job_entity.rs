use crate::domain::job::entity::job_metadata_entity::JobMetadataEntity;
use crate::domain::job::r#enum::job_enum::{JobMetadataStatus, JobType};
use crate::infrastructure::persistence::job::job_metadata::Model;
use crate::infrastructure::persistence::job::sea_orm_active_enums::{
    JobStatusEnum, ScheduleTypeEnum,
};
use crate::infrastructure::persistence::job::{job, job_metadata};
use chrono::NaiveDateTime;
use sea_orm::JsonValue;
use sea_orm::prelude::Uuid;

#[derive(Debug, Clone)]
pub struct JobEntity {
    pub id: Uuid,
    pub time: NaiveDateTime,
    pub target: String,
    pub r#type: JobType,
    pub payload: JsonValue,
    pub metadata: Option<JobMetadataEntity>,
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
                Some(job_metadata) => Some(JobMetadataEntity {
                    job_id: job_metadata.job_id,
                    status: match job_metadata.status {
                        JobStatusEnum::Scheduled => JobMetadataStatus::Scheduled,
                        JobStatusEnum::Processing => JobMetadataStatus::Processing,
                        JobStatusEnum::Completed => JobMetadataStatus::Completed,
                        JobStatusEnum::Deleted => JobMetadataStatus::Deleted,
                        JobStatusEnum::Failed => JobMetadataStatus::Failed,
                    },
                    failure: job_metadata.failure,
                    processed_at: job_metadata.processed_at,
                }),
            },
        }
    }
}
