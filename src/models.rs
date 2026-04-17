use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ── Users ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id:         Uuid,
    pub username:   String,
    pub email:      String,
    pub department: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub username:   String,
    pub email:      String,
    pub department: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    #[serde(default, deserialize_with = "deserialize_optional_trimmed")]
    pub username:   Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_trimmed")]
    pub email:      Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_trimmed")]
    pub department: Option<String>,
}

// ── Laptops ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
pub enum LaptopStatus {
    Available,
    Assigned,
    InRepair,
    Retired,
}

impl fmt::Display for LaptopStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LaptopStatus::Available => write!(f, "available"),
            LaptopStatus::Assigned => write!(f, "assigned"),
            LaptopStatus::InRepair => write!(f, "in_repair"),
            LaptopStatus::Retired => write!(f, "retired"),
        }
    }
}

impl std::str::FromStr for LaptopStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "available" => Ok(LaptopStatus::Available),
            "assigned" => Ok(LaptopStatus::Assigned),
            "in_repair" => Ok(LaptopStatus::InRepair),
            "retired" => Ok(LaptopStatus::Retired),
            _ => Err(format!(
                "Invalid status '{}'. Must be one of: available, assigned, in_repair, retired",
                s
            )),
        }
    }
}

impl Serialize for LaptopStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for LaptopStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<LaptopStatus>().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Laptop {
    pub id:            Uuid,
    pub brand:         String,
    pub model:         String,
    pub serial_number: String,
    pub status:        LaptopStatus,
    pub assigned_to:   Option<Uuid>,
    pub purchase_date: NaiveDate,
    pub created_at:    DateTime<Utc>,
    pub updated_at:    DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLaptop {
    pub brand:         String,
    pub model:         String,
    pub serial_number: String,
    pub purchase_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLaptop {
    #[serde(default, deserialize_with = "deserialize_optional_trimmed")]
    pub brand:         Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_trimmed")]
    pub model:         Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_trimmed")]
    pub serial_number: Option<String>,
    /// Use POST /laptops/{id}/assign or /unassign to change assigned status.
    #[serde(default, deserialize_with = "deserialize_optional_status")]
    pub status:        Option<LaptopStatus>,
    #[serde(default, deserialize_with = "deserialize_optional_date")]
    pub purchase_date: Option<NaiveDate>,
}

fn deserialize_optional_trimmed<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(s.and_then(|v| {
        let t = v.trim();
        if t.is_empty() { None } else { Some(t.to_string()) }
    }))
}

fn deserialize_optional_status<'de, D>(deserializer: D) -> Result<Option<LaptopStatus>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        None => Ok(None),
        Some(v) if v.trim().is_empty() => Ok(None),
        Some(v) => v.trim().parse::<LaptopStatus>().map(Some).map_err(serde::de::Error::custom),
    }
}

fn deserialize_optional_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        None => Ok(None),
        Some(v) if v.trim().is_empty() => Ok(None),
        Some(v) => NaiveDate::parse_from_str(v.trim(), "%Y-%m-%d")
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignLaptop {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LaptopWithAssignee {
    pub id:              Uuid,
    pub brand:           String,
    pub model:           String,
    pub serial_number:   String,
    pub status:          LaptopStatus,
    pub assigned_to:     Option<Uuid>,
    pub assignee_name:   Option<String>,
    pub purchase_date:   NaiveDate,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

// ── Pagination ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: i64, per_page: i64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Self { data, total, page, per_page, total_pages }
    }
}

pub const DEFAULT_PAGE: i64 = 1;
pub const DEFAULT_PER_PAGE: i64 = 50;
pub const MAX_PER_PAGE: i64 = 100;

pub fn clamp_page(page: Option<i64>) -> i64 {
    page.unwrap_or(DEFAULT_PAGE).max(1)
}

pub fn clamp_per_page(per_page: Option<i64>) -> i64 {
    per_page.unwrap_or(DEFAULT_PER_PAGE).clamp(1, MAX_PER_PAGE)
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct LaptopListQuery {
    pub status: Option<LaptopStatus>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
