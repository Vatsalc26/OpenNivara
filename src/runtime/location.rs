use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LocationContext {
    pub status: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub accuracy_meters: Option<f64>,
    pub source: String,
    pub captured_at: Option<String>,
    pub freshness_seconds: Option<u32>,
    pub timezone_hint: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub label: Option<String>,
    pub permission_state: String,
    pub privacy_level: String,
}

impl Default for LocationContext {
    fn default() -> Self {
        Self {
            status: "unknown".into(),
            latitude: None,
            longitude: None,
            accuracy_meters: None,
            source: "unknown".into(),
            captured_at: None,
            freshness_seconds: None,
            timezone_hint: None,
            city: None,
            region: None,
            country: None,
            label: None,
            permission_state: "unavailable".into(),
            privacy_level: "disabled".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateSavedPlace {
    pub label: String,
    pub place_type: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub timezone: Option<String>,
    pub details_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SavedPlace {
    pub id: String,
    pub label: String,
    pub place_type: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub timezone: Option<String>,
    pub details_json: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LocationObservationInput {
    pub status: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub accuracy_meters: Option<f64>,
    pub source: String,
    pub label: Option<String>,
    pub details_json: String,
    pub privacy_level: String,
}

pub fn save_place(conn: &Connection, input: &CreateSavedPlace) -> anyhow::Result<SavedPlace> {
    let now = Utc::now().to_rfc3339();
    let place = SavedPlace {
        id: format!("place_{}", Uuid::new_v4()),
        label: input.label.clone(),
        place_type: input.place_type.clone(),
        latitude: input.latitude,
        longitude: input.longitude,
        address: input.address.clone(),
        city: input.city.clone(),
        region: input.region.clone(),
        country: input.country.clone(),
        timezone: input.timezone.clone(),
        details_json: input.details_json.clone(),
        created_at: now.clone(),
        updated_at: now,
        deleted_at: None,
    };
    conn.execute(
        "INSERT INTO saved_places (
            id, label, place_type, latitude, longitude, address, city, region, country,
            timezone, details_json, created_at, updated_at, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, NULL)",
        params![
            &place.id,
            &place.label,
            &place.place_type,
            place.latitude,
            place.longitude,
            &place.address,
            &place.city,
            &place.region,
            &place.country,
            &place.timezone,
            &place.details_json,
            &place.created_at,
            &place.updated_at,
        ],
    )?;
    get_saved_place(conn, &place.id)?.ok_or_else(|| anyhow::anyhow!("created place missing"))
}

pub fn list_saved_places(conn: &Connection) -> anyhow::Result<Vec<SavedPlace>> {
    let mut stmt = conn.prepare(
        "SELECT id, label, place_type, latitude, longitude, address, city, region, country,
            timezone, details_json, created_at, updated_at, deleted_at
         FROM saved_places
         WHERE deleted_at IS NULL
         ORDER BY place_type = 'home' DESC, updated_at DESC",
    )?;
    let rows = stmt.query_map([], saved_place_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn delete_saved_place(conn: &Connection, place_id: &str) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE saved_places SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![place_id, now],
    )?;
    Ok(())
}

pub fn record_location_observation(
    conn: &Connection,
    input: &LocationObservationInput,
) -> anyhow::Result<LocationContext> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO location_observations (
            id, status, latitude, longitude, accuracy_meters, source, captured_at,
            freshness_seconds, label, details_json, privacy_level
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8, ?9, ?10)",
        params![
            format!("loc_{}", Uuid::new_v4()),
            &input.status,
            input.latitude,
            input.longitude,
            input.accuracy_meters,
            &input.source,
            &now,
            &input.label,
            &input.details_json,
            &input.privacy_level,
        ],
    )?;
    get_location_context(conn, true)
}

pub fn update_manual_location(
    conn: &Connection,
    input: &LocationObservationInput,
) -> anyhow::Result<LocationContext> {
    let mut manual = input.clone();
    manual.source = "manual".into();
    record_location_observation(conn, &manual)
}

pub fn get_location_context(
    conn: &Connection,
    allow_exact: bool,
) -> anyhow::Result<LocationContext> {
    if !allow_exact {
        return Ok(LocationContext {
            permission_state: "denied".into(),
            privacy_level: "disabled".into(),
            ..LocationContext::default()
        });
    }

    if let Some(mut context) = latest_location_observation(conn)? {
        enrich_location_from_place(conn, &mut context)?;
        return Ok(context);
    }

    if let Some(place) = list_saved_places(conn)?.into_iter().next() {
        return Ok(LocationContext {
            status: "saved_place".into(),
            latitude: place.latitude,
            longitude: place.longitude,
            accuracy_meters: None,
            source: "saved_place".into(),
            captured_at: Some(place.updated_at),
            freshness_seconds: None,
            timezone_hint: place.timezone,
            city: place.city,
            region: place.region,
            country: place.country,
            label: Some(place.label),
            permission_state: "granted".into(),
            privacy_level: "approximate".into(),
        });
    }

    Ok(LocationContext {
        permission_state: "granted".into(),
        privacy_level: "approximate".into(),
        ..LocationContext::default()
    })
}

pub fn location_is_fresh(context: &LocationContext, max_age_seconds: u32) -> bool {
    context
        .freshness_seconds
        .map(|age| age <= max_age_seconds)
        .unwrap_or_else(|| context.captured_at.is_some() && context.status == "saved_place")
}

fn get_saved_place(conn: &Connection, place_id: &str) -> anyhow::Result<Option<SavedPlace>> {
    conn.query_row(
        "SELECT id, label, place_type, latitude, longitude, address, city, region, country,
            timezone, details_json, created_at, updated_at, deleted_at
         FROM saved_places WHERE id = ?1",
        [place_id],
        saved_place_from_row,
    )
    .optional()
    .map_err(Into::into)
}

fn latest_location_observation(conn: &Connection) -> anyhow::Result<Option<LocationContext>> {
    conn.query_row(
        "SELECT status, latitude, longitude, accuracy_meters, source, captured_at,
            label, privacy_level
         FROM location_observations
         ORDER BY captured_at DESC
         LIMIT 1",
        [],
        |row| {
            let captured_at: String = row.get(5)?;
            let freshness_seconds =
                chrono::DateTime::parse_from_rfc3339(&captured_at)
                    .ok()
                    .map(|captured| {
                        Utc::now()
                            .signed_duration_since(captured.with_timezone(&Utc))
                            .num_seconds()
                            .max(0) as u32
                    });
            Ok(LocationContext {
                status: row.get(0)?,
                latitude: row.get(1)?,
                longitude: row.get(2)?,
                accuracy_meters: row.get(3)?,
                source: row.get(4)?,
                captured_at: Some(captured_at),
                freshness_seconds,
                timezone_hint: None,
                city: None,
                region: None,
                country: None,
                label: row.get(6)?,
                permission_state: "granted".into(),
                privacy_level: row.get(7)?,
            })
        },
    )
    .optional()
    .map_err(Into::into)
}

fn enrich_location_from_place(
    conn: &Connection,
    context: &mut LocationContext,
) -> anyhow::Result<()> {
    let Some(label) = context.label.as_deref() else {
        return Ok(());
    };
    let place = conn
        .query_row(
            "SELECT id, label, place_type, latitude, longitude, address, city, region, country,
                timezone, details_json, created_at, updated_at, deleted_at
             FROM saved_places
             WHERE deleted_at IS NULL AND lower(label) = lower(?1)
             ORDER BY updated_at DESC
             LIMIT 1",
            [label],
            saved_place_from_row,
        )
        .optional()?;
    if let Some(place) = place {
        context.timezone_hint = place.timezone;
        context.city = place.city;
        context.region = place.region;
        context.country = place.country;
        context.label = Some(place.label);
    }
    Ok(())
}

fn saved_place_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SavedPlace> {
    Ok(SavedPlace {
        id: row.get(0)?,
        label: row.get(1)?,
        place_type: row.get(2)?,
        latitude: row.get(3)?,
        longitude: row.get(4)?,
        address: row.get(5)?,
        city: row.get(6)?,
        region: row.get(7)?,
        country: row.get(8)?,
        timezone: row.get(9)?,
        details_json: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
        deleted_at: row.get(13)?,
    })
}
