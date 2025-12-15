use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub department: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub department: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Laptop {
    pub id: Uuid,
    pub user_id: Uuid,
    pub brand: String,
    pub model: String,
    pub serial_number: String,
    pub purchase_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLaptop {
    pub user_id: Uuid,
    pub brand: String,
    pub model: String,
    pub serial_number: String,
    pub purchase_date: DateTime<Utc>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AllocateLaptop {
    pub laptop_id: Uuid,
    pub user_id: Uuid,
}
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LaptopWithUser {
    pub laptop: Laptop,
    pub user: User,
}
