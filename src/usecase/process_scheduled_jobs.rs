use crate::entity::prelude::Job;
use crate::entity::{job, job_metadata};
use chrono::Utc;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

pub async fn process(db: &DatabaseConnection) -> Result<(), DbErr> {
    let jobs: Vec<(job::Model, Option<job_metadata::Model>)> = Job::find()
        .filter(job::Column::Time.lte(Utc::now().naive_utc()))
        .find_also_related(job_metadata::Entity)
        .all(db)
        .await?;

    for (job, maybe_metadata) in jobs {
        println!("{:?}", job);
        println!("{:?}", maybe_metadata);
    }

    Ok(())
}
