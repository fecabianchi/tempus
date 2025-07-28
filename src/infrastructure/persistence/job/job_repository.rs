use crate::domain::job::entity::job_entity::JobEntity;
use crate::domain::job::port::driven::job_repository_port::JobRepositoryPort;
use crate::infrastructure::persistence::job::prelude::Job;
use crate::infrastructure::persistence::job::sea_orm_active_enums::JobStatusEnum;
use crate::infrastructure::persistence::job::{job, job_metadata};
use chrono::Utc;
use sea_orm::prelude::Uuid;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, Statement, TransactionTrait,
};

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

    async fn find_and_flag_processing(&self) -> Result<Vec<JobEntity>, DbErr> {
        let txn = self.db.begin().await?;

        let sql = r#"
            UPDATE job_metadata
            SET status = 'processing'
            WHERE job_id IN (
                SELECT job.id
                FROM job
                INNER JOIN job_metadata ON job.id = job_metadata.job_id
                WHERE job.time <= NOW()
                  AND job_metadata.status = 'scheduled'
                FOR UPDATE SKIP LOCKED
                LIMIT $1
            )
            RETURNING job_id
        "#;

        let rows = txn
            .query_all(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                sql,
                vec![500.into()],
            ))
            .await?;

        let job_ids: Vec<Uuid> = rows
            .into_iter()
            .filter_map(|row| row.try_get::<Uuid>("", "job_id").ok())
            .collect();

        if job_ids.is_empty() {
            txn.commit().await?;
            return Ok(vec![]);
        }

        let jobs = Job::find()
            .filter(job::Column::Id.is_in(job_ids))
            .find_also_related(job_metadata::Entity)
            .all(&txn)
            .await?;

        txn.commit().await?;

        Ok(jobs.into_iter().map(JobEntity::from).collect())
    }
}
