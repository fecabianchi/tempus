use crate::domain::job::r#enum::job_enum::JobMetadataStatus;
use chrono::NaiveDateTime;
use sea_orm::prelude::Uuid;

#[derive(Debug, Clone)]
pub struct JobMetadataEntity {
    pub job_id: Uuid,
    pub status: JobMetadataStatus,
    pub failure: Option<String>,
    pub processed_at: Option<NaiveDateTime>,
}
