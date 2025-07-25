use crate::domain::job::entity::job_entity::JobEntity;
use crate::domain::job::port::driven::job_repository_port::JobRepositoryPort;
use crate::infrastructure::persistence::job::prelude::Job;
use crate::infrastructure::persistence::job::sea_orm_active_enums::JobStatusEnum;
use crate::infrastructure::persistence::job::{job, job_metadata};
use chrono::Utc;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

pub struct JobRepository {
    db: DatabaseConnection,
}

impl JobRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl JobRepositoryPort for JobRepository {
    async fn find_all(&self) -> Result<Vec<JobEntity>, DbErr> {
        let rows = Job::find()
            .filter(job::Column::Time.lte(Utc::now().naive_utc()))
            .find_also_related(job_metadata::Entity)
            .filter(job_metadata::Column::Status.eq(JobStatusEnum::Scheduled))
            .all(&self.db)
            .await?;

        let jobs: Vec<JobEntity> = rows.into_iter().map(JobEntity::from).collect();

        Ok(jobs)
    }
}
