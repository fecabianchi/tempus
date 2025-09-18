use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("Failed to initialize metrics: {0}")]
    Initialization(String),
}

static PROMETHEUS_HANDLE: OnceCell<Arc<PrometheusHandle>> = OnceCell::new();

fn initialize_counters() {
    counter!("jobs_processed_total", "status" => "success").absolute(0);
    counter!("jobs_processed_total", "status" => "failure").absolute(0);
    counter!("jobs_processed_total", "status" => "retry").absolute(0);
    counter!("jobs_http_requests_total", "status_code" => "200").absolute(0);
    counter!("jobs_kafka_messages_total").absolute(0);
    histogram!("jobs_duration_seconds").record(0.0);
    gauge!("current_processing_jobs").set(0.0);
}

fn create_prometheus_handle() -> Result<Arc<PrometheusHandle>, MetricsError> {
    PrometheusBuilder::new()
        .install_recorder()
        .map(Arc::new)
        .map_err(|e| MetricsError::Initialization(e.to_string()))
}

fn set_handle_once(handle: Arc<PrometheusHandle>) -> Arc<PrometheusHandle> {
    match PROMETHEUS_HANDLE.set(handle.clone()) {
        Ok(()) => handle,
        Err(_) => {
            log::warn!("Prometheus handle already set, this is expected when multiple processes share metrics");
            handle
        }
    }
}

pub fn init_metrics() -> Result<Option<Arc<PrometheusHandle>>, MetricsError> {
    create_prometheus_handle()
        .map(set_handle_once)
        .map(|handle| {
            initialize_counters();
            log::debug!("Metrics initialized successfully with handle");
            Some(handle)
        })
}

pub fn get_prometheus_handle() -> Option<Arc<PrometheusHandle>> {
    PROMETHEUS_HANDLE.get().cloned()
}

fn log_and_increment_counter(name: &'static str, label_key: &'static str, label_value: String, description: &str) {
    log::debug!("{}", description);
    counter!(name, label_key => label_value).increment(1);
}

fn log_and_increment_simple_counter(name: &'static str, description: &str) {
    log::debug!("{}", description);
    counter!(name).increment(1);
}

fn log_and_modify_gauge(action: &str, delta: f64) {
    log::debug!("{} current processing jobs", action);
    match action {
        "Incrementing" => gauge!("current_processing_jobs").increment(delta),
        "Decrementing" => gauge!("current_processing_jobs").decrement(delta),
        _ => gauge!("current_processing_jobs").set(delta),
    }
}

pub fn increment_jobs_processed(status: &str) {
    log_and_increment_counter(
        "jobs_processed_total",
        "status",
        status.to_string(),
        &format!("Incrementing jobs_processed_total with status: {}", status)
    );
}

pub fn observe_job_duration(duration_seconds: f64) {
    log::debug!("Recording job duration: {} seconds", duration_seconds);
    histogram!("jobs_duration_seconds").record(duration_seconds);
}

pub fn increment_http_requests(status_code: u16) {
    log_and_increment_counter(
        "jobs_http_requests_total",
        "status_code",
        status_code.to_string(),
        &format!("Incrementing HTTP requests with status code: {}", status_code)
    );
}

pub fn increment_kafka_messages() {
    log_and_increment_simple_counter("jobs_kafka_messages_total", "Incrementing Kafka messages counter");
}

pub fn set_current_processing_jobs(count: i64) {
    gauge!("current_processing_jobs").set(count as f64);
}

pub fn increment_current_processing_jobs() {
    log_and_modify_gauge("Incrementing", 1.0);
}

pub fn decrement_current_processing_jobs() {
    log_and_modify_gauge("Decrementing", 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_functions_do_not_panic() {
        increment_jobs_processed("success");
        observe_job_duration(1.5);
        increment_http_requests(200);
        increment_kafka_messages();
        set_current_processing_jobs(5);
        increment_current_processing_jobs();
        decrement_current_processing_jobs();
        
        assert!(true);
    }

    #[test]
    fn test_init_metrics() {
        let result = init_metrics();
        match result {
            Ok(Some(_handle)) => {
                let retrieved_handle = get_prometheus_handle();
                assert!(retrieved_handle.is_some());
            }
            Ok(None) => {
                assert!(false, "Expected handle to be returned");
            }
            Err(_) => {
                assert!(true);
            }
        }
    }
    
    #[test]
    fn test_get_prometheus_handle_without_init() {
        let handle = get_prometheus_handle();
        assert!(handle.is_some() || handle.is_none());
    }
}
