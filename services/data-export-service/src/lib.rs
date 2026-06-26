use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DataExportError>;

#[derive(Debug, Error)]
pub enum DataExportError {
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("invalid filter")]
    InvalidFilter,
    #[error("storage error")]
    StorageError,
    #[error("internal error")]
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExportFormat {
    Csv,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportFilter {
    pub date_start: Option<DateTime<Utc>>,
    pub date_end: Option<DateTime<Utc>>,
    pub merchant_address: Option<String>,
    pub payer_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentRecord {
    pub order_id: String,
    pub merchant_address: String,
    pub payer_address: String,
    pub token_address: String,
    pub amount: i128,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct ExportService {
    records: Vec<PaymentRecord>,
}

impl ExportService {
    pub fn new(records: Vec<PaymentRecord>) -> Self {
        Self { records }
    }

    pub fn export(&self, format: ExportFormat, filter: ExportFilter) -> Result<String> {
        let filtered = self.filter_records(filter)?;
        match format {
            ExportFormat::Csv => self.export_csv(&filtered),
            ExportFormat::Json => self.export_json(&filtered),
        }
    }

    fn filter_records(&self, filter: ExportFilter) -> Result<Vec<&PaymentRecord>> {
        let mut items = Vec::new();
        for record in self.records.iter() {
            if let Some(start) = filter.date_start {
                if record.created_at < start {
                    continue;
                }
            }
            if let Some(end) = filter.date_end {
                if record.created_at > end {
                    continue;
                }
            }
            if let Some(ref merchant) = filter.merchant_address {
                if &record.merchant_address != merchant {
                    continue;
                }
            }
            if let Some(ref payer) = filter.payer_address {
                if &record.payer_address != payer {
                    continue;
                }
            }
            items.push(record);
        }
        Ok(items)
    }

    fn export_csv(&self, records: &[&PaymentRecord]) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.write_record(&[
            "order_id",
            "merchant_address",
            "payer_address",
            "token_address",
            "amount",
            "status",
            "created_at",
        ])
        .map_err(|e| DataExportError::Serialization(e.to_string()))?;

        for record in records.iter() {
            wtr.serialize(record)
                .map_err(|e| DataExportError::Serialization(e.to_string()))?;
        }

        std::string::String::from_utf8(wtr.into_inner().map_err(|e| DataExportError::Serialization(e.to_string()))?)
            .map_err(|e| DataExportError::Serialization(e.to_string()))
    }

    fn export_json(&self, records: &[&PaymentRecord]) -> Result<String> {
        serde_json::to_string_pretty(records).map_err(|e| DataExportError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_record(order_id: &str, created_at: DateTime<Utc>) -> PaymentRecord {
        PaymentRecord {
            order_id: order_id.into(),
            merchant_address: "MERCHANT_001".into(),
            payer_address: "PAYER_001".into(),
            token_address: "TOKEN_001".into(),
            amount: 500,
            status: "Completed".into(),
            created_at,
        }
    }

    #[test]
    fn export_json_filters_by_date() {
        let records = vec![
            sample_record("ORDER_001", Utc.ymd(2026, 6, 1).and_hms(0, 0, 0)),
            sample_record("ORDER_002", Utc.ymd(2026, 7, 1).and_hms(0, 0, 0)),
        ];
        let service = ExportService::new(records);
        let result = service
            .export(
                ExportFormat::Json,
                ExportFilter {
                    date_start: Some(Utc.ymd(2026, 6, 15).and_hms(0, 0, 0)),
                    date_end: None,
                    merchant_address: None,
                    payer_address: None,
                },
            )
            .unwrap();

        assert!(result.contains("ORDER_002"));
        assert!(!result.contains("ORDER_001"));
    }

    #[test]
    fn export_csv_generates_header() {
        let records = vec![sample_record("ORDER_001", Utc.ymd(2026, 6, 1).and_hms(0, 0, 0))];
        let service = ExportService::new(records);
        let result = service
            .export(
                ExportFormat::Csv,
                ExportFilter {
                    date_start: None,
                    date_end: None,
                    merchant_address: None,
                    payer_address: None,
                },
            )
            .unwrap();

        assert!(result.contains("order_id,merchant_address,payer_address"));
        assert!(result.contains("ORDER_001"));
    }
}
