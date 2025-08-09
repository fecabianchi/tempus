use crate::config::app_config::AppConfig;
use crate::error::{Result, TempusError};
use log::{error, info};
use once_cell::sync::Lazy;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use sea_orm::JsonValue;
use std::sync::Arc;

static KAFKA_PRODUCER: Lazy<Arc<FutureProducer>> = Lazy::new(|| {
    let config = AppConfig::load().expect("Failed to load configuration");
    let producer = create_kafka_producer(&config).expect("Failed to create Kafka producer");
    Arc::new(producer)
});

pub struct KafkaPublisher {
    producer: Arc<FutureProducer>,
    config: AppConfig,
}

fn create_kafka_producer(config: &AppConfig) -> Result<FutureProducer> {
    let mut client_config = ClientConfig::new();
    
    info!("Creating Kafka producer with bootstrap servers: {}", config.kafka.bootstrap_servers);
    
    client_config
        .set("bootstrap.servers", &config.kafka.bootstrap_servers)
        .set("message.timeout.ms", (config.kafka.producer_timeout_secs * 1000).to_string())
        .set("retries", config.kafka.producer_retries.to_string())
        .set("batch.size", config.kafka.batch_size.to_string())
        .set("compression.type", &config.kafka.compression_type)
        .set("acks", "all")
        .set("enable.idempotence", "true");

    client_config
        .create()
        .map_err(|e| TempusError::Kafka(e.to_string()))
}

impl KafkaPublisher {
    pub fn new(config: &AppConfig) -> Result<Self> {
        Ok(Self {
            producer: KAFKA_PRODUCER.clone(),
            config: config.clone(),
        })
    }

    pub fn get_lazy_producer() -> Arc<FutureProducer> {
        KAFKA_PRODUCER.clone()
    }

    pub async fn publish_message(&self, topic: &str, payload: JsonValue) -> Result<()> {
        let payload_str = payload.to_string();
        
        info!("Publishing message to Kafka topic: {}", topic);
        
        let record: FutureRecord<(), _> = FutureRecord::to(topic)
            .payload(&payload_str)
            .timestamp(chrono::Utc::now().timestamp_millis());

        match self.producer
            .send(record, Timeout::After(self.config.kafka.producer_timeout()))
            .await
        {
            Ok((partition, offset)) => {
                info!("Message successfully published to topic: {}, partition: {}, offset: {}", 
                      topic, partition, offset);
                Ok(())
            }
            Err((kafka_error, _)) => {
                error!("Failed to publish message to Kafka: {}", kafka_error);
                Err(TempusError::Kafka(kafka_error.to_string()))
            }
        }
    }

    pub async fn publish_to_default_topic(&self, payload: JsonValue) -> Result<()> {
        self.publish_message(&self.config.kafka.default_topic, payload).await
    }
}

pub async fn publish_kafka_message(target: String, payload: JsonValue) -> Result<()> {
    let config = AppConfig::load()?;
    let producer = KAFKA_PRODUCER.clone();

    let topic = if target.is_empty() {
        config.kafka.default_topic.clone()
    } else {
        target
    };

    let timeout = config.kafka.producer_timeout();
    let payload_str = payload.to_string();
    
    info!("Publishing message to Kafka topic: {}", topic);
    
    let record: FutureRecord<(), _> = FutureRecord::to(&topic)
        .payload(&payload_str)
        .timestamp(chrono::Utc::now().timestamp_millis());

    match producer
        .send(record, Timeout::After(timeout))
        .await
    {
        Ok((partition, offset)) => {
            info!("Message successfully published to topic: {}, partition: {}, offset: {}", 
                  topic, partition, offset);
            Ok(())
        }
        Err((kafka_error, _)) => {
            error!("Failed to publish message to Kafka: {}", kafka_error);
            Err(TempusError::Kafka(kafka_error.to_string()))
        }
    }
}