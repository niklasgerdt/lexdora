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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_org_create_req_deser() {
        let json = r#"{"name": "ACME", "legal_entity_id": "123"}"#;
        let req: OrgCreateReq = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "ACME");
        assert_eq!(req.legal_entity_id, Some("123".to_string()));
    }

    #[test]
    fn test_org_update_req_deser() {
        let json = r#"{"name": "New ACME"}"#;
        let req: OrgUpdateReq = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, Some("New ACME".to_string()));
        assert_eq!(req.legal_entity_id, None);
    }

    #[test]
    fn test_user_create_req_deser() {
        let org_id = Uuid::new_v4();
        let json = format!(r#"{{"organization_id": "{}", "email": "test@example.com", "full_name": "Test User"}}"#, org_id);
        let req: UserCreateReq = serde_json::from_str(&json).unwrap();
        assert_eq!(req.organization_id, org_id);
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.full_name, Some("Test User".to_string()));
    }

    #[test]
    fn test_incident_create_req_deser() {
        let org_id = Uuid::new_v4();
        let json = format!(r#"{{"organization_id": "{}", "title": "Outage", "type_": "Network", "severity": "High", "detected_at": "2024-03-24T12:00:00Z"}}"#, org_id);
        let req: IncidentCreateReq = serde_json::from_str(&json).unwrap();
        assert_eq!(req.organization_id, org_id);
        assert_eq!(req.title, "Outage");
        assert_eq!(req.type_, "Network");
        assert_eq!(req.severity, "High");
        assert_eq!(req.detected_at, "2024-03-24T12:00:00Z");
    }

    #[test]
    fn test_tpp_create_req_deser() {
        let org_id = Uuid::new_v4();
        let json = format!(r#"{{"organization_id": "{}", "name": "Cloud Provider", "criticality": "Critical", "is_important": true}}"#, org_id);
        let req: TppCreateReq = serde_json::from_str(&json).unwrap();
        assert_eq!(req.organization_id, org_id);
        assert_eq!(req.name, "Cloud Provider");
        assert_eq!(req.criticality, Some("Critical".to_string()));
        assert_eq!(req.is_important, Some(true));
    }

    #[test]
    fn test_role_create_req_deser() {
        let org_id = Uuid::new_v4();
        let json = format!(r#"{{"organization_id": "{}", "name": "Admin", "description": "Administrator role"}}"#, org_id);
        let req: RoleCreateReq = serde_json::from_str(&json).unwrap();
        assert_eq!(req.organization_id, org_id);
        assert_eq!(req.name, "Admin");
        assert_eq!(req.description, Some("Administrator role".to_string()));
    }

    #[test]
    fn test_permission_create_req_deser() {
        let json = r#"{"name": "read_orgs", "description": "Can read organizations"}"#;
        let req: PermissionCreateReq = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "read_orgs");
        assert_eq!(req.description, Some("Can read organizations".to_string()));
    }

    #[test]
    fn test_bu_create_req_deser() {
        let org_id = Uuid::new_v4();
        let json = format!(r#"{{"organization_id": "{}", "name": "IT Department"}}"#, org_id);
        let req: BuCreateReq = serde_json::from_str(&json).unwrap();
        assert_eq!(req.organization_id, org_id);
        assert_eq!(req.name, "IT Department");
    }

    #[test]
    fn test_asset_create_req_deser() {
        let org_id = Uuid::new_v4();
        let json = format!(r#"{{"organization_id": "{}", "name": "Database Server", "criticality": "High"}}"#, org_id);
        let req: AssetCreateReq = serde_json::from_str(&json).unwrap();
        assert_eq!(req.organization_id, org_id);
        assert_eq!(req.name, "Database Server");
        assert_eq!(req.criticality, Some("High".to_string()));
    }

    #[test]
    fn test_id_out_ser() {
        let id = Uuid::new_v4();
        let out = IdOut { id };
        let json = serde_json::to_string(&out).unwrap();
        assert_eq!(json, format!(r#"{{"id":"{}"}}"#, id));
    }
}
