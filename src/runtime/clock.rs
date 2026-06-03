use chrono::{Datelike, Duration, TimeZone, Utc};
use chrono_tz::Tz;

use super::context::{RelativeDateContext, RuntimeContext};
use super::location::LocationContext;
use super::model_registry;

pub fn get_runtime_context(timezone: Option<&str>) -> RuntimeContext {
    runtime_context_at(Utc::now(), timezone, LocationContext::default())
}

pub fn runtime_context_at(
    now_utc: chrono::DateTime<Utc>,
    timezone: Option<&str>,
    location: LocationContext,
) -> RuntimeContext {
    let tz: Tz = timezone
        .and_then(|name| name.parse().ok())
        .unwrap_or(Tz::UTC);
    let local = now_utc.with_timezone(&tz);
    let date = local.date_naive();
    let today_start = tz
        .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
        .single()
        .unwrap_or_else(|| now_utc.with_timezone(&tz));
    let tomorrow_start = today_start + Duration::days(1);
    let yesterday_start = today_start - Duration::days(1);
    let week_start = today_start - Duration::days(local.weekday().num_days_from_monday() as i64);
    let next_week_start = week_start + Duration::days(7);
    let month_start = tz
        .with_ymd_and_hms(date.year(), date.month(), 1, 0, 0, 0)
        .single()
        .unwrap_or(today_start);
    let (next_month_year, next_month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    let next_month_start = tz
        .with_ymd_and_hms(next_month_year, next_month, 1, 0, 0, 0)
        .single()
        .unwrap_or(month_start + Duration::days(31));

    RuntimeContext {
        now_utc: now_utc.to_rfc3339(),
        now_local: local.to_rfc3339(),
        timezone: tz.name().to_string(),
        date_local: date.to_string(),
        day_of_week: local.weekday().to_string(),
        locale: None,
        calendar_week: Some(local.iso_week().week()),
        relative_date_context: RelativeDateContext {
            today_start: today_start.to_rfc3339(),
            today_end: (tomorrow_start - Duration::seconds(1)).to_rfc3339(),
            tomorrow_start: tomorrow_start.to_rfc3339(),
            tomorrow_end: (tomorrow_start + Duration::days(1) - Duration::seconds(1)).to_rfc3339(),
            yesterday_start: yesterday_start.to_rfc3339(),
            yesterday_end: (today_start - Duration::seconds(1)).to_rfc3339(),
            current_week_start: week_start.to_rfc3339(),
            current_week_end: (next_week_start - Duration::seconds(1)).to_rfc3339(),
            next_week_start: next_week_start.to_rfc3339(),
            next_week_end: (next_week_start + Duration::days(7) - Duration::seconds(1))
                .to_rfc3339(),
            current_month_start: month_start.to_rfc3339(),
            current_month_end: (next_month_start - Duration::seconds(1)).to_rfc3339(),
            next_month_start: next_month_start.to_rfc3339(),
            next_month_end: (next_month_start + Duration::days(31) - Duration::seconds(1))
                .to_rfc3339(),
        },
        location,
        model: model_registry::get_current_model_context_info(),
    }
}

pub fn resolve_relative_time_phrase(phrase: &str, runtime: &RuntimeContext) -> Option<String> {
    let phrase = phrase.to_lowercase();
    let dates = &runtime.relative_date_context;
    if phrase.contains("tomorrow") {
        Some(format!("{}..{}", dates.tomorrow_start, dates.tomorrow_end))
    } else if phrase.contains("yesterday") {
        Some(format!(
            "{}..{}",
            dates.yesterday_start, dates.yesterday_end
        ))
    } else if phrase.contains("today") {
        Some(format!("{}..{}", dates.today_start, dates.today_end))
    } else if phrase.contains("next week") {
        Some(format!(
            "{}..{}",
            dates.next_week_start, dates.next_week_end
        ))
    } else if phrase.contains("this week") {
        Some(format!(
            "{}..{}",
            dates.current_week_start, dates.current_week_end
        ))
    } else if phrase.contains("last month") {
        Some("last_month".into())
    } else {
        None
    }
}
