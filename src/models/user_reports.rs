use crate::common::error::{AppError, ServiceResult};
use crate::entities::user_reports;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::str::FromStr;

#[derive(Serialize)]
pub struct UserReport {
    pub report_id: i64,
    pub reason: String,
    pub time: DateTime<Utc>,
    pub from_uid: i64,
    pub to_uid: i64,
}

impl TryFrom<user_reports::UserReport> for UserReport {
    type Error = AppError;

    fn try_from(report: user_reports::UserReport) -> ServiceResult<Self> {
        let timestamp = i64::from_str(&report.time)?;
        let time = DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| anyhow!("timestamp out of range"))?;
        Ok(Self {
            time,
            report_id: report.id,
            reason: report.reason,
            from_uid: report.from_uid,
            to_uid: report.to_uid,
        })
    }
}
