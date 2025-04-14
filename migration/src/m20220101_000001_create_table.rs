use sea_orm::{DbErr, DeriveIden, DeriveMigrationName, EnumIter, Iterable};
use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::{ForeignKey, Table};
use sea_orm_migration::schema::{date_time, enumeration, pk_uuid, string, timestamps};
use sea_orm_migration::{async_trait, MigrationTrait, SchemaManager};
use std::process::id;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_type(
                Type::create()
                    .as_enum(ScheduleTypeEnum)
                    .values(ScheduleType::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(JobStatusEnum)
                    .values(JobStatus::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(timestamps(
                Table::create()
                    .table(Job::Table)
                    .if_not_exists()
                    .col(pk_uuid(Job::Id))
                    .col(date_time(Job::Time))
                    .col(string(Job::Target))
                    .col(enumeration(
                        Job::Type,
                        ScheduleTypeEnum,
                        ScheduleType::iter(),
                    ))
                    .to_owned(),
            ))
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(JobMetadata::Table)
                    .if_not_exists()
                    .col(
                        pk_uuid(JobMetadata::JobId), // makes JobId both PK and NOT NULL
                    )
                    .col(enumeration(
                        JobMetadata::Status,
                        JobStatusEnum,
                        JobStatus::iter(),
                    ))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-jobmetadata-job-id")
                            .from(JobMetadata::Table, JobMetadata::JobId)
                            .to(Job::Table, Job::Id)
                            .on_delete(sea_orm::prelude::ForeignKeyAction::Cascade)
                            .on_update(sea_orm::prelude::ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(Job::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum JobMetadata {
    Table,
    JobId,
    Status,
}

#[derive(DeriveIden)]
enum Job {
    Table,
    Id,
    Time,
    Target,
    Type,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
struct JobStatusEnum;

#[derive(DeriveIden, EnumIter)]
pub enum JobStatus {
    Scheduled,
    Processing,
    Completed,
    Deleted,
    Failed,
}

#[derive(DeriveIden)]
struct ScheduleTypeEnum;

#[derive(DeriveIden, EnumIter)]
pub enum ScheduleType {
    Http,
}
