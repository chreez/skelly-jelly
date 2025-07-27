//! Database layer for event storage

use crate::{config::DatabaseConfig, error::Result, types::*};
use chrono::{DateTime, Utc};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    ConnectOptions, Row, SqlitePool,
};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Time-series optimized database for event storage
pub struct TimeSeriesDatabase {
    pool: SqlitePool,
    config: DatabaseConfig,
}

impl TimeSeriesDatabase {
    /// Create a new database instance
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        // Ensure database directory exists
        if let Some(parent) = Path::new(&config.path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Build connection options
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", config.path.display()))?
            .journal_mode(if config.wal_enabled {
                SqliteJournalMode::Wal
            } else {
                SqliteJournalMode::Delete
            })
            .synchronous(SqliteSynchronous::from_str(&config.synchronous_mode).unwrap_or(SqliteSynchronous::Normal))
            .busy_timeout(Duration::from_secs(5))
            .log_statements(tracing::log::LevelFilter::Debug);

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(config.pool_size)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(600))
            .test_before_acquire(true)
            .connect_with(options)
            .await?;

        info!("Database connection pool established with {} connections", config.pool_size);

        let db = Self { pool, config };
        
        // Run migrations
        db.migrate().await?;
        
        Ok(db)
    }

    /// Run database migrations
    async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");
        
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                timestamp INTEGER NOT NULL,
                session_id BLOB NOT NULL,
                event_type INTEGER NOT NULL,
                data BLOB NOT NULL,
                PRIMARY KEY (timestamp, session_id)
            ) WITHOUT ROWID;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_events_session 
            ON events(session_id, timestamp);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Screenshot metadata table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS screenshot_metadata (
                screenshot_id BLOB PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                window_title TEXT,
                app_name TEXT,
                text_density REAL,
                ui_element_count INTEGER,
                dominant_colors TEXT,
                privacy_masked INTEGER
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_screenshots_timestamp 
            ON screenshot_metadata(timestamp);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Aggregation tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS event_aggregates_minute (
                timestamp INTEGER PRIMARY KEY,
                keystroke_count INTEGER,
                mouse_clicks INTEGER,
                window_switches INTEGER,
                active_time_ms INTEGER,
                screenshot_count INTEGER
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS event_aggregates_hour (
                timestamp INTEGER PRIMARY KEY,
                keystroke_count INTEGER,
                mouse_clicks INTEGER,
                window_switches INTEGER,
                active_time_ms INTEGER,
                screenshot_count INTEGER
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS event_aggregates_day (
                timestamp INTEGER PRIMARY KEY,
                keystroke_count INTEGER,
                mouse_clicks INTEGER,
                window_switches INTEGER,
                active_time_ms INTEGER,
                screenshot_count INTEGER
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("Database migrations completed");
        Ok(())
    }

    /// Store a raw event
    pub async fn store_event(&self, session_id: &Uuid, event: &RawEvent) -> Result<()> {
        let timestamp = event.timestamp().timestamp_millis();
        let event_type = match event {
            RawEvent::Keystroke(_) => 1,
            RawEvent::MouseMove(_) => 2,
            RawEvent::MouseClick(_) => 3,
            RawEvent::WindowFocus(_) => 4,
            RawEvent::Screenshot(_) => 5,
            RawEvent::ProcessStart(_) => 6,
            RawEvent::ResourceUsage(_) => 7,
        };
        
        let data = bincode::serialize(event)?;
        
        sqlx::query(
            r#"
            INSERT INTO events (timestamp, session_id, event_type, data)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(timestamp)
        .bind(session_id.as_bytes())
        .bind(event_type)
        .bind(&data)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    /// Store multiple events in a batch
    pub async fn store_events_batch(&self, session_id: &Uuid, events: &[RawEvent]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        
        for event in events {
            let timestamp = event.timestamp().timestamp_millis();
            let event_type = match event {
                RawEvent::Keystroke(_) => 1,
                RawEvent::MouseMove(_) => 2,
                RawEvent::MouseClick(_) => 3,
                RawEvent::WindowFocus(_) => 4,
                RawEvent::Screenshot(_) => 5,
                RawEvent::ProcessStart(_) => 6,
                RawEvent::ResourceUsage(_) => 7,
            };
            
            let data = bincode::serialize(event)?;
            
            sqlx::query(
                r#"
                INSERT INTO events (timestamp, session_id, event_type, data)
                VALUES (?1, ?2, ?3, ?4)
                "#,
            )
            .bind(timestamp)
            .bind(session_id.as_bytes())
            .bind(event_type)
            .bind(&data)
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(())
    }

    /// Store screenshot metadata
    pub async fn store_screenshot_metadata(
        &self,
        id: &ScreenshotId,
        metadata: &ScreenshotMetadata,
    ) -> Result<()> {
        let timestamp = metadata.timestamp.timestamp_millis();
        let dominant_colors = serde_json::to_string(&metadata.dominant_colors)?;
        
        sqlx::query(
            r#"
            INSERT INTO screenshot_metadata (
                screenshot_id, timestamp, window_title, app_name,
                text_density, ui_element_count, dominant_colors, privacy_masked
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(id.as_bytes())
        .bind(timestamp)
        .bind(&metadata.window_title)
        .bind(&metadata.app_name)
        .bind(metadata.text_density)
        .bind(metadata.ui_element_count as i32)
        .bind(&dominant_colors)
        .bind(metadata.privacy_masked as i32)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    /// Get events for a time range
    pub async fn get_events(
        &self,
        session_id: &Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<RawEvent>> {
        let start_ts = start.timestamp_millis();
        let end_ts = end.timestamp_millis();
        
        let rows = sqlx::query(
            r#"
            SELECT data FROM events 
            WHERE session_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3
            ORDER BY timestamp
            "#,
        )
        .bind(session_id.as_bytes())
        .bind(start_ts)
        .bind(end_ts)
        .fetch_all(&self.pool)
        .await?;
        
        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let data: Vec<u8> = row.get("data");
            let event: RawEvent = bincode::deserialize(&data)?;
            events.push(event);
        }
        
        Ok(events)
    }

    /// Delete old events based on retention policy
    pub async fn cleanup_old_events(&self, retention_days: u32) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        let cutoff_ts = cutoff.timestamp_millis();
        
        let result = sqlx::query(
            r#"
            DELETE FROM events WHERE timestamp < ?1
            "#,
        )
        .bind(cutoff_ts)
        .execute(&self.pool)
        .await?;
        
        let deleted = result.rows_affected();
        if deleted > 0 {
            info!("Deleted {} old events", deleted);
        }
        
        Ok(deleted)
    }

    /// Get database size in bytes
    pub async fn get_size(&self) -> Result<u64> {
        let row = sqlx::query(
            r#"
            SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        
        let size: i64 = row.get("size");
        Ok(size as u64)
    }

    /// Vacuum the database
    pub async fn vacuum(&self) -> Result<()> {
        info!("Running database vacuum...");
        sqlx::query("VACUUM").execute(&self.pool).await?;
        info!("Database vacuum completed");
        Ok(())
    }

    /// Get connection pool for direct access
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Close the database
    pub async fn close(self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_db() -> (TimeSeriesDatabase, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let mut config = DatabaseConfig::default();
        config.path = db_path;
        config.pool_size = 1;
        
        let db = TimeSeriesDatabase::new(config).await.unwrap();
        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_database_creation() {
        let (db, _temp_dir) = create_test_db().await;
        assert!(db.get_size().await.unwrap() > 0);
    }

    #[tokio::test]
    async fn test_event_storage() {
        let (db, _temp_dir) = create_test_db().await;
        let session_id = Uuid::new_v4();
        
        let event = RawEvent::Keystroke(KeystrokeEvent {
            timestamp: Utc::now(),
            key_code: 65,
            modifiers: KeyModifiers {
                shift: false,
                ctrl: false,
                alt: false,
                meta: false,
            },
            inter_key_interval_ms: Some(100),
        });
        
        db.store_event(&session_id, &event).await.unwrap();
        
        let events = db.get_events(
            &session_id,
            Utc::now() - chrono::Duration::minutes(1),
            Utc::now() + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();
        
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_batch_storage() {
        let (db, _temp_dir) = create_test_db().await;
        let session_id = Uuid::new_v4();
        
        let events = vec![
            RawEvent::Keystroke(KeystrokeEvent {
                timestamp: Utc::now(),
                key_code: 65,
                modifiers: KeyModifiers::default(),
                inter_key_interval_ms: Some(100),
            }),
            RawEvent::MouseClick(MouseClickEvent {
                timestamp: Utc::now(),
                x: 100,
                y: 200,
                button: MouseButton::Left,
                click_type: ClickType::Single,
            }),
        ];
        
        db.store_events_batch(&session_id, &events).await.unwrap();
        
        let stored_events = db.get_events(
            &session_id,
            Utc::now() - chrono::Duration::minutes(1),
            Utc::now() + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();
        
        assert_eq!(stored_events.len(), 2);
    }
}