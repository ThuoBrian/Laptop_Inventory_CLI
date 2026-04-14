use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
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

/// Valid status values: "available" | "assigned" | "in_repair" | "retired"
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Laptop {
    pub id:            Uuid,
    pub brand:         String,
    pub model:         String,
    pub serial_number: String,
    pub status:        String,
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
    /// Allowed: "available" | "in_repair" | "retired"
    /// Use POST /laptops/{id}/assign or /unassign to change assigned status.
    pub status:        Option<String>,
    pub purchase_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignLaptop {
    pub user_id: Uuid,
}
