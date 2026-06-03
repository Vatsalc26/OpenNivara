use serde::{Deserialize, Serialize};
use specta::Type;

use super::location::LocationContext;
use super::model_registry::ModelContextInfo;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RelativeDateContext {
    pub today_start: String,
    pub today_end: String,
    pub tomorrow_start: String,
    pub tomorrow_end: String,
    pub yesterday_start: String,
    pub yesterday_end: String,
    pub current_week_start: String,
    pub current_week_end: String,
    pub next_week_start: String,
    pub next_week_end: String,
    pub current_month_start: String,
    pub current_month_end: String,
    pub next_month_start: String,
    pub next_month_end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RuntimeContext {
    pub now_utc: String,
    pub now_local: String,
    pub timezone: String,
    pub date_local: String,
    pub day_of_week: String,
    pub locale: Option<String>,
    pub calendar_week: Option<u32>,
    pub relative_date_context: RelativeDateContext,
    pub location: LocationContext,
    pub model: ModelContextInfo,
}
