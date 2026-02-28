use chrono::{DateTime, NaiveDateTime, Utc};
use propbot_core::{Bar, DataError, Tick};
use rust_decimal::Decimal;
use std::path::Path;
use std::str::FromStr;

/// Load OHLCV bars from a CSV file.
///
/// Expected columns (case-insensitive, flexible ordering):
/// `timestamp` (or `date`, `datetime`), `open`, `high`, `low`, `close`, `volume`
///
/// Supports common date formats.
pub fn load_bars_from_csv(path: &Path) -> Result<Vec<Bar>, DataError> {
    let instrument = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(path)
        .map_err(|e| DataError::ParseError(format!("Failed to open CSV: {}", e)))?;

    let headers = reader
        .headers()
        .map_err(|e| DataError::ParseError(format!("Failed to read headers: {}", e)))?
        .clone();

    let col_map = resolve_bar_columns(&headers)?;

    let mut bars = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| DataError::ParseError(format!("CSV record error: {}", e)))?;

        let timestamp = parse_timestamp(&record[col_map.timestamp])?;
        let open = parse_decimal(&record[col_map.open], "open")?;
        let high = parse_decimal(&record[col_map.high], "high")?;
        let low = parse_decimal(&record[col_map.low], "low")?;
        let close = parse_decimal(&record[col_map.close], "close")?;
        let volume = if let Some(vol_idx) = col_map.volume {
            parse_decimal(&record[vol_idx], "volume")?
        } else {
            Decimal::ZERO
        };

        bars.push(Bar {
            instrument: instrument.clone(),
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        });
    }

    // Sort by timestamp
    bars.sort_by_key(|b| b.timestamp);
    Ok(bars)
}

/// Load tick data from a CSV file.
///
/// Expected columns: `timestamp`, `bid`, `ask`, `last`, `volume`
pub fn load_ticks_from_csv(path: &Path) -> Result<Vec<Tick>, DataError> {
    let instrument = path
        .file_stem()
        .map(|s| {
            let name = s.to_string_lossy().to_string();
            name.strip_suffix("_ticks").unwrap_or(&name).to_string()
        })
        .unwrap_or_else(|| "unknown".to_string());

    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(path)
        .map_err(|e| DataError::ParseError(format!("Failed to open CSV: {}", e)))?;

    let headers = reader
        .headers()
        .map_err(|e| DataError::ParseError(format!("Failed to read headers: {}", e)))?
        .clone();

    let ts_col = find_column(&headers, &["timestamp", "date", "datetime", "time"])
        .ok_or_else(|| DataError::ParseError("No timestamp column found".into()))?;
    let bid_col = find_column(&headers, &["bid"])
        .ok_or_else(|| DataError::ParseError("No bid column found".into()))?;
    let ask_col = find_column(&headers, &["ask"])
        .ok_or_else(|| DataError::ParseError("No ask column found".into()))?;
    let last_col = find_column(&headers, &["last", "price"]);
    let vol_col = find_column(&headers, &["volume", "vol", "size"]);

    let mut ticks = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| DataError::ParseError(format!("CSV record error: {}", e)))?;

        let timestamp = parse_timestamp(&record[ts_col])?;
        let bid = parse_decimal(&record[bid_col], "bid")?;
        let ask = parse_decimal(&record[ask_col], "ask")?;
        let last = if let Some(idx) = last_col {
            parse_decimal(&record[idx], "last")?
        } else {
            (bid + ask) / Decimal::TWO
        };
        let volume = if let Some(idx) = vol_col {
            parse_decimal(&record[idx], "volume")?
        } else {
            Decimal::ZERO
        };

        ticks.push(Tick {
            instrument: instrument.clone(),
            timestamp,
            bid,
            ask,
            last,
            volume,
        });
    }

    ticks.sort_by_key(|t| t.timestamp);
    Ok(ticks)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

struct BarColumnMap {
    timestamp: usize,
    open: usize,
    high: usize,
    low: usize,
    close: usize,
    volume: Option<usize>,
}

fn resolve_bar_columns(headers: &csv::StringRecord) -> Result<BarColumnMap, DataError> {
    let ts = find_column(headers, &["timestamp", "date", "datetime", "time"])
        .ok_or_else(|| DataError::ParseError("No timestamp column found".into()))?;
    let open = find_column(headers, &["open", "o"])
        .ok_or_else(|| DataError::ParseError("No open column found".into()))?;
    let high = find_column(headers, &["high", "h"])
        .ok_or_else(|| DataError::ParseError("No high column found".into()))?;
    let low = find_column(headers, &["low", "l"])
        .ok_or_else(|| DataError::ParseError("No low column found".into()))?;
    let close = find_column(headers, &["close", "c"])
        .ok_or_else(|| DataError::ParseError("No close column found".into()))?;
    let volume = find_column(headers, &["volume", "vol", "v"]);

    Ok(BarColumnMap {
        timestamp: ts,
        open,
        high,
        low,
        close,
        volume,
    })
}

fn find_column(headers: &csv::StringRecord, names: &[&str]) -> Option<usize> {
    for (i, header) in headers.iter().enumerate() {
        let h = header.trim().to_lowercase();
        for name in names {
            if h == *name {
                return Some(i);
            }
        }
    }
    None
}

fn parse_decimal(s: &str, field: &str) -> Result<Decimal, DataError> {
    Decimal::from_str(s.trim())
        .map_err(|e| DataError::ParseError(format!("Failed to parse {} '{}': {}", field, s, e)))
}

fn parse_timestamp(s: &str) -> Result<DateTime<Utc>, DataError> {
    let s = s.trim();

    // Try RFC 3339 / ISO 8601 with timezone
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Common formats (without timezone, assume UTC)
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%m/%d/%Y %H:%M:%S",
        "%m/%d/%Y %H:%M",
        "%Y-%m-%d",
        "%Y%m%d %H:%M:%S",
        "%d/%m/%Y %H:%M:%S",
    ];

    for fmt in &formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(s, fmt) {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
        }
    }

    // Try date-only formats
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let naive_dt = naive_date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
    }

    // Try Unix timestamp (seconds)
    if let Ok(ts) = s.parse::<i64>() {
        if let Some(dt) = DateTime::from_timestamp(ts, 0) {
            return Ok(dt);
        }
    }

    Err(DataError::ParseError(format!(
        "Unable to parse timestamp: '{}'",
        s
    )))
}
