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
    pub username:   Option<String>,
    pub email:      Option<String>,
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
    pub brand:         Option<String>,
    pub model:         Option<String>,
    pub serial_number: Option<String>,
    /// Use POST /laptops/{id}/assign or /unassign to change assigned status.
    pub status:        Option<LaptopStatus>,
    pub purchase_date: Option<NaiveDate>,
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
