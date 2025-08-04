use crate::domain::job::entity::job_metadata_entity::JobMetadataEntity;
use crate::domain::job::r#enum::job_enum::JobMetadataStatus;
use crate::domain::job::port::driven::job_metadata_repository_port::JobMetadataRepositoryPort;
use crate::infrastructure::persistence::job::job_metadata;
use crate::infrastructure::persistence::job::sea_orm_active_enums::JobStatusEnum;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

#[derive(Clone)]
pub struct JobMetadataRepository {
    db: DatabaseConnection,
}

impl JobMetadataRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl JobMetadataRepositoryPort for JobMetadataRepository {
    async fn update_status(&self, job_metadata: JobMetadataEntity) -> Result<(), DbErr> {
        let to_update = job_metadata::ActiveModel {
            job_id: sea_orm::Set(job_metadata.job_id),
            status: sea_orm::Set(to_model_status(job_metadata.status)),
            processed_at: sea_orm::Set(job_metadata.processed_at),
            failure: sea_orm::Set(job_metadata.failure),
        };

        job_metadata::Entity::update(to_update)
            .exec(&self.db)
            .await?;
            
        Ok(())
    }
}

fn to_model_status(status: JobMetadataStatus) -> JobStatusEnum {
    match status {
        JobMetadataStatus::Scheduled => JobStatusEnum::Scheduled,
        JobMetadataStatus::Processing => JobStatusEnum::Processing,
        JobMetadataStatus::Completed => JobStatusEnum::Completed,
        JobMetadataStatus::Deleted => JobStatusEnum::Deleted,
        JobMetadataStatus::Failed => JobStatusEnum::Failed,
    }
}
