<p align="center">
  <img width="400" height="400" alt="Tempus logo" src="https://github.com/user-attachments/assets/9c3f08d9-5fd5-457b-84d6-0a3d62c10897" />
</p>  
<p align="center">
Tempus is a minimalist, blazing-fast, and scalable scheduler designed to handle time-based job execution with maximum efficiency and simplicity.
</p>  
<br/>

## Features

- ⚡ **Reliable Job Execution**: Execute scheduled jobs with built-in retry mechanisms and failure handling
- 🌐 **Multi-Protocol Support**: Support for both HTTP webhooks and Kafka message publishing
- 🔗 **RESTful API**: Complete CRUD operations for job management via HTTP API
- 💾 **Database Persistence**: PostgreSQL integration with Sea-ORM for reliable job storage
- 📅 **Job Rescheduling**: Update job execution times dynamically via API
- 🚀 **Concurrent Processing**: Multi-threaded job processing with configurable concurrency limits  
- 🔄 **Retry Logic**: Configurable retry attempts with exponential backoff for failed jobs
- 📊 **Job Status Tracking**: Complete job lifecycle management (Scheduled, Processing, Completed, Failed, Deleted)
- 🛑 **Graceful Shutdown**: Signal handling for clean shutdown with running job completion
- ⚙️ **Configuration Management**: Environment-based configuration with sensible defaults
- 📝 **Structured Logging**: Comprehensive logging for monitoring and debugging
- 📊 **Prometheus Metrics**: Built-in metrics collection and export for monitoring and observability

## Architecture

Tempus is built using a clean hexagonal architecture with clear separation of concerns:

- **Domain Layer**: Core business logic and entities
- **Infrastructure Layer**: Database persistence, HTTP clients, and Kafka integration
- **API Layer**: RESTful endpoints for job management
- **Engine Layer**: Job processing engine with concurrent execution

## Quick Start

### Prerequisites

- Rust 2024 edition
- PostgreSQL database
- Kafka cluster (optional, for Kafka jobs)

### Installation

1. Clone the repository
2. Set up your environment variables (see Configuration section)
3. Run database migrations:
   ```bash
   cd migration && cargo run
   ```
4. Build the project:
   ```bash
   cargo build --release
   ```

### Running

**Start the scheduler engine:**
```bash
cargo run --bin tempus
```

**Start the API server:**
```bash
cargo run --bin tempus-api
```

The engine will expose metrics on `http://localhost:3001/metrics` and the API will be available on `http://localhost:3000`.

## API Usage

### Create a Job

**HTTP Job:**
```bash
curl -X POST http://localhost:3000/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "type": "http",
    "target": "https://api.example.com/webhook",
    "time": "2024-01-01T12:00:00Z",
    "payload": {
      "message": "Hello World"
    }
  }'
```

**Kafka Job:**
```bash
curl -X POST http://localhost:3000/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "type": "kafka",
    "target": "my-topic",
    "time": "2024-01-01T12:00:00Z",
    "payload": {
      "event": "user.created",
      "userId": 123
    }
  }'
```

### Reschedule a Job

```bash
curl -X PATCH http://localhost:3000/jobs/{job_id}/time \
  -H "Content-Type: application/json" \
  -d '{
    "time": "2024-01-02T12:00:00Z"
  }'
```

### Delete a Job

```bash
curl -X DELETE http://localhost:3000/jobs/{job_id}
```

## Metrics and Monitoring

Tempus provides comprehensive Prometheus metrics for monitoring job execution and system performance. All metrics are exposed by the engine on port 3001.

### Available Metrics

- **`jobs_processed_total{status}`**: Counter of processed jobs by status (success, failure, retry)
- **`jobs_duration_seconds`**: Histogram of job execution duration
- **`jobs_http_requests_total{status_code}`**: Counter of HTTP requests made by jobs
- **`jobs_kafka_messages_total`**: Counter of Kafka messages published
- **`current_processing_jobs`**: Gauge of currently processing jobs

### Accessing Metrics

```bash
# Get all metrics from the engine
curl http://localhost:3001/metrics

# Check engine health
curl http://localhost:3001/health
```

### Prometheus Configuration

Add the following to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'tempus-engine'
    static_configs:
      - targets: ['localhost:3001']
    scrape_interval: 5s
    metrics_path: /metrics
```

## Configuration

Tempus uses environment variables for configuration. You can set the following variables:

### Database Configuration
- `DATABASE_URL`: PostgreSQL connection string
- `DATABASE_MAX_CONNECTIONS`: Maximum database connections (default: 100)
- `DATABASE_MIN_CONNECTIONS`: Minimum database connections (default: 30)
- `DATABASE_CONNECT_TIMEOUT_SECS`: Connection timeout in seconds (default: 8)
- `DATABASE_ACQUIRE_TIMEOUT_SECS`: Connection acquire timeout (default: 8)
- `DATABASE_IDLE_TIMEOUT_SECS`: Connection idle timeout (default: 60)
- `DATABASE_MAX_LIFETIME_SECS`: Connection max lifetime (default: 60)

### Engine Configuration
- `ENGINE_MAX_CONCURRENT_JOBS`: Maximum concurrent job processing (default: 10)
- `ENGINE_RETRY_ATTEMPTS`: Number of retry attempts for failed jobs (default: 3)
- `ENGINE_BASE_DELAY_MINUTES`: Base delay between retries in minutes (default: 2)

### HTTP Configuration
- `HTTP_PORT`: API server port (default: 3000)
- `HTTP_POOL_IDLE_TIMEOUT_SECS`: HTTP client pool idle timeout (default: 30)
- `HTTP_REQUEST_TIMEOUT_SECS`: HTTP request timeout (default: 30)

### Kafka Configuration
- `KAFKA_BOOTSTRAP_SERVERS`: Kafka bootstrap servers (default: localhost:9092)
- `KAFKA_DEFAULT_TOPIC`: Default topic for Kafka jobs (default: tempus-events)
- `KAFKA_PRODUCER_TIMEOUT_SECS`: Producer timeout in seconds (default: 30)
- `KAFKA_PRODUCER_RETRIES`: Number of producer retries (default: 5)
- `KAFKA_BATCH_SIZE`: Producer batch size (default: 16384)
- `KAFKA_COMPRESSION_TYPE`: Compression type (default: snappy)

## Development

### Running Tests

```bash
cargo test
```

### Database Migrations

To create a new migration:
```bash
cd migration
cargo run -- generate MIGRATION_NAME
```

To run migrations:
```bash
cd migration
cargo run
```

### API Testing

The project includes Bruno API collection files in the `bruno/` directory for testing the API endpoints.

> ⚠️ *Note: Tempus is under active development. APIs and features may change as the project evolves.*
