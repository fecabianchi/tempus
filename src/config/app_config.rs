use crate::error::{Result, TempusError};
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub engine: EngineConfig,
    pub http: HttpConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EngineConfig {
    pub max_concurrent_jobs: usize,
    pub retry_attempts: i32,
    pub base_delay_minutes: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpConfig {
    pub port: u16,
    pub pool_idle_timeout_secs: u64,
    pub request_timeout_secs: u64,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config = Config::builder()
            .set_default("database.max_connections", 100)?
            .set_default("database.min_connections", 30)?
            .set_default("database.connect_timeout_secs", 8)?
            .set_default("database.acquire_timeout_secs", 8)?
            .set_default("database.idle_timeout_secs", 60)?
            .set_default("database.max_lifetime_secs", 60)?
            .set_default("engine.max_concurrent_jobs", 10)?
            .set_default("engine.retry_attempts", 3)?
            .set_default("engine.base_delay_minutes", 2)?
            .set_default("http.pool_idle_timeout_secs", 30)?
            .set_default("http.request_timeout_secs", 30)?
            .set_default("http.port", 3000)?
            .add_source(Environment::default().separator("_"))
            .build()
            .map_err(|e| TempusError::Config(e.to_string()))?;

        let app_config: AppConfig = config
            .try_deserialize()
            .map_err(|e| TempusError::Config(e.to_string()))?;

        app_config.validate()?;
        Ok(app_config)
    }

    fn validate(&self) -> Result<()> {
        if self.database.url.is_empty() {
            return Err(TempusError::Validation(
                "Database URL cannot be empty".to_string(),
            ));
        }

        if self.database.max_connections < self.database.min_connections {
            return Err(TempusError::Validation(
                "Max connections cannot be less than min connections".to_string(),
            ));
        }

        if self.engine.max_concurrent_jobs == 0 {
            return Err(TempusError::Validation(
                "Max concurrent jobs must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

impl From<ConfigError> for TempusError {
    fn from(err: ConfigError) -> Self {
        TempusError::Config(err.to_string())
    }
}

impl DatabaseConfig {
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connect_timeout_secs)
    }

    pub fn acquire_timeout(&self) -> Duration {
        Duration::from_secs(self.acquire_timeout_secs)
    }

    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }

    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_secs)
    }
}

impl HttpConfig {
    pub fn pool_idle_timeout(&self) -> Duration {
        Duration::from_secs(self.pool_idle_timeout_secs)
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }
}
