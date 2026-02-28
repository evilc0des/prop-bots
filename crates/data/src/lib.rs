pub mod csv_loader;
pub mod db;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use propbot_core::{Bar, DataError, DataProvider, Tick, Timeframe};

/// A CSV-file-based data provider.
pub struct CsvDataProvider {
    pub directory: std::path::PathBuf,
}

impl CsvDataProvider {
    pub fn new(directory: impl Into<std::path::PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }
}

#[async_trait]
impl DataProvider for CsvDataProvider {
    async fn load_bars(
        &self,
        instrument: &str,
        _timeframe: Timeframe,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Bar>, DataError> {
        let file_path = self.directory.join(format!("{}.csv", instrument));
        if !file_path.exists() {
            return Err(DataError::NotFound(format!(
                "CSV file not found: {}",
                file_path.display()
            )));
        }
        let bars = csv_loader::load_bars_from_csv(&file_path)?;
        let filtered: Vec<Bar> = bars
            .into_iter()
            .filter(|b| b.timestamp >= start && b.timestamp <= end)
            .collect();
        Ok(filtered)
    }

    async fn load_ticks(
        &self,
        instrument: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Tick>, DataError> {
        let file_path = self.directory.join(format!("{}_ticks.csv", instrument));
        if !file_path.exists() {
            return Err(DataError::NotFound(format!(
                "Tick CSV file not found: {}",
                file_path.display()
            )));
        }
        let ticks = csv_loader::load_ticks_from_csv(&file_path)?;
        let filtered: Vec<Tick> = ticks
            .into_iter()
            .filter(|t| t.timestamp >= start && t.timestamp <= end)
            .collect();
        Ok(filtered)
    }

    async fn available_instruments(&self) -> Result<Vec<String>, DataError> {
        let mut instruments = Vec::new();
        let entries = std::fs::read_dir(&self.directory)
            .map_err(|e| DataError::IoError(e))?;
        for entry in entries {
            let entry = entry.map_err(|e| DataError::IoError(e))?;
            let path = entry.path();
            if path.extension().map(|e| e == "csv").unwrap_or(false) {
                if let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy().to_string();
                    if !name.ends_with("_ticks") {
                        instruments.push(name);
                    }
                }
            }
        }
        instruments.sort();
        Ok(instruments)
    }
}

/// A PostgreSQL-backed data provider.
pub struct PostgresDataProvider {
    pub pool: sqlx::PgPool,
}

impl PostgresDataProvider {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DataProvider for PostgresDataProvider {
    async fn load_bars(
        &self,
        instrument: &str,
        _timeframe: Timeframe,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Bar>, DataError> {
        db::load_bars(&self.pool, instrument, start, end)
            .await
            .map_err(|e| DataError::DatabaseError(e.to_string()))
    }

    async fn load_ticks(
        &self,
        instrument: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Tick>, DataError> {
        db::load_ticks(&self.pool, instrument, start, end)
            .await
            .map_err(|e| DataError::DatabaseError(e.to_string()))
    }

    async fn available_instruments(&self) -> Result<Vec<String>, DataError> {
        db::available_instruments(&self.pool)
            .await
            .map_err(|e| DataError::DatabaseError(e.to_string()))
    }
}
