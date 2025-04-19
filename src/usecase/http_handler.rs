use crate::entity::job::Model as JobModel;
use crate::entity::job_metadata;
use crate::entity::job_metadata::Model as JobMetadataModel;
use crate::entity::prelude::JobMetadata;
use crate::entity::sea_orm_active_enums::JobStatusEnum;
use chrono::Utc;
use reqwest::{Client, Error, Response};
use sea_orm::{DatabaseConnection, EntityTrait};

pub async fn handle(job: &JobModel, metadata: &JobMetadataModel, db: &DatabaseConnection) {
    if metadata.status == JobStatusEnum::Scheduled {
        match { perform_request(job).await } {
            Ok(_) => {
                update_metadata(metadata, JobStatusEnum::Completed, db, None).await;
            }
            Err(e) => {
                println!("Error: {}", e);
                update_metadata(metadata, JobStatusEnum::Failed, db, Some(e.to_string())).await;
            }
        }
    }
}

async fn update_metadata(
    metadata: &JobMetadataModel,
    status: JobStatusEnum,
    db: &DatabaseConnection,
    failure: Option<String>,
) {
    let to_update = job_metadata::ActiveModel {
        job_id: sea_orm::Set(metadata.job_id),
        status: sea_orm::Set(status),
        processed_at: sea_orm::Set(Option::from(Utc::now().naive_utc())),
        failure: sea_orm::Set(failure),
    };

    JobMetadata::update(to_update)
        .exec(db)
        .await
        .expect("Failed to update metadata");
}

fn perform_request(job: &JobModel) -> impl Future<Output = Result<Response, Error>> {
    Client::new()
        .post(&job.target)
        .json(&serde_json::json!(job.payload))
        .send()
}
