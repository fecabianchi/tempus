use chrono::NaiveDateTime;
use crate::domain::job::entity::job_entity::JobEntity;
use sea_orm::DbErr;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::prelude::Uuid;

#[async_trait]
pub trait JobRepositoryPort: Send + Sync {
    async fn find_all(&self) -> Result<Vec<JobEntity>, DbErr>;
    async fn find_and_flag_processing(&self) -> Result<Vec<JobEntity>, DbErr>;
    async fn increment_retry(&self, job_id: Uuid) -> Result<(), DbErr>;
    async fn update_time(&self, job_id: Uuid, time: NaiveDateTime) -> Result<(), DbErr>;
}
