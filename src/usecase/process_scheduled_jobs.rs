use crate::entity::job;
use crate::entity::prelude::Job;
use chrono::{Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

pub async fn process(db: &DatabaseConnection) -> Result<(), DbErr> {
    let jobs: Vec<job::Model> = Job::find()
        .filter(job::Column::Time.lte(Utc::now().naive_utc()))
        .all(db)
        .await?;

    println!("{:?}", jobs);

    Ok(())
}
