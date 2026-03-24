use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct IdOut {
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct OrgCreateReq {
    pub name: String,
    #[serde(default)]
    pub legal_entity_id: Option<String>,
}

#[derive(Deserialize)]
pub struct OrgUpdateReq {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub legal_entity_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UserCreateReq {
    pub organization_id: Uuid,
    pub email: String,
    #[serde(default)]
    pub full_name: Option<String>,
}

#[derive(Deserialize)]
pub struct UserUpdateReq {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Serialize)]
pub struct UserOut {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct IncidentCreateReq {
    pub organization_id: Uuid,
    pub title: String,
    pub type_: String,
    pub severity: String,
    pub detected_at: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_major: Option<bool>,
}

#[derive(Deserialize)]
pub struct IncidentUpdateReq {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub type_: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_major: Option<bool>,
}

#[derive(Serialize)]
pub struct IncidentOut {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub type_: String,
    pub severity: String,
    pub detected_at: DateTime<Utc>,
    pub is_major: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct TppCreateReq {
    pub organization_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub criticality: Option<String>,
    #[serde(default)]
    pub is_important: Option<bool>,
}

#[derive(Deserialize)]
pub struct TppUpdateReq {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub criticality: Option<String>,
    #[serde(default)]
    pub is_important: Option<bool>,
}

#[derive(Serialize)]
pub struct TppOut {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub country: Option<String>,
    pub criticality: Option<String>,
    pub is_important: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct RoleCreateReq {
    pub organization_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct RoleUpdateReq {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct RoleOut {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct PermissionCreateReq {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct PermissionUpdateReq {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct PermissionOut {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct RolePermissionOut {
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub permission_name: String,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct RolePermissionAddReq {
    pub permission_id: Uuid,
}

#[derive(Serialize)]
pub struct UserRoleOut {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub role_name: String,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct UserRoleAddReq {
    pub role_id: Uuid,
}

#[derive(Deserialize)]
pub struct BuCreateReq {
    pub organization_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct BuUpdateReq {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct BuOut {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct AssetCreateReq {
    pub organization_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, value_name = "asset_criticality")]
    pub criticality: Option<String>,
    #[serde(default)]
    pub owner_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct AssetUpdateReq {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub criticality: Option<String>,
    #[serde(default)]
    pub owner_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct AssetOut {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub criticality: Option<String>,
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct OrgOut {
    pub id: Uuid,
    pub name: String,
    pub legal_entity_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
