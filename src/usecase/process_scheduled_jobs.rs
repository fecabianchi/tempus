use crate::entity::prelude::Job;
use crate::entity::sea_orm_active_enums::{JobStatusEnum, ScheduleTypeEnum};
use crate::entity::{job, job_metadata};
use crate::usecase::http_handler;
use chrono::Utc;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

pub async fn process(db: &DatabaseConnection) -> Result<(), DbErr> {
    let jobs: Vec<(job::Model, Option<job_metadata::Model>)> = Job::find()
        .filter(job::Column::Time.lte(Utc::now().naive_utc()))
        .find_also_related(job_metadata::Entity)
        .filter(job_metadata::Column::Status.eq(JobStatusEnum::Scheduled))
        .all(db)
        .await?;

    for (job, maybe_metadata) in jobs {
        match job.r#type {
            ScheduleTypeEnum::Http => match maybe_metadata {
                None => {
                    println!("No metadata found for job");
                }
                Some(metadata) => http_handler::handle(&job, &metadata, &db).await,
            },
        }
    }

    Ok(())
}
