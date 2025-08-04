use sea_orm::prelude::async_trait::async_trait;
use crate::domain::job::entity::job_metadata_entity::JobMetadataEntity;
use sea_orm::DbErr;

#[async_trait]
pub trait JobMetadataRepositoryPort: Send + Sync {
    async fn update_status(&self, job_metadata: JobMetadataEntity) -> Result<(), DbErr>; 
}
