use crate::domain::job::entity::job_entity::JobEntity;
use sea_orm::DbErr;

pub trait JobRepositoryPort {
    async fn find_all(&self) -> Result<Vec<JobEntity>, DbErr>;
}
