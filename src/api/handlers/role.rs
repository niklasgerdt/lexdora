use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use sqlx::Row;
use crate::api::AppState;
use crate::models::{IdOut, RoleCreateReq, RoleUpdateReq, RoleOut, RolePermissionAddReq, RolePermissionOut};

pub async fn roles_create(State(st): State<AppState>, Json(req): Json<RoleCreateReq>) -> impl IntoResponse {
    let res = sqlx::query("INSERT INTO dora.roles(organization_id, name, description) VALUES ($1, $2, $3) RETURNING id")
        .bind(req.organization_id)
        .bind(req.name)
        .bind(req.description)
        .fetch_one(&st.pool)
        .await;
    match res {
        Ok(r) => {
            let id: Uuid = r.try_get("id").unwrap();
            (axum::http::StatusCode::CREATED, Json(IdOut { id })).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn roles_list(State(st): State<AppState>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, organization_id, name, description, created_at, updated_at FROM dora.roles ORDER BY name")
        .fetch_all(&st.pool)
        .await;
    match res {
        Ok(rows) => {
            let out: Vec<RoleOut> = rows.into_iter().map(|r| RoleOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                name: r.try_get("name").unwrap(),
                description: r.try_get("description").unwrap_or(None),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            }).collect();
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn roles_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, organization_id, name, description, created_at, updated_at FROM dora.roles WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await;
    match res {
        Ok(r) => {
            let out = RoleOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                name: r.try_get("name").unwrap(),
                description: r.try_get("description").unwrap_or(None),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            };
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn roles_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<RoleUpdateReq>) -> impl IntoResponse {
    let res = sqlx::query("UPDATE dora.roles SET name=COALESCE($2, name), description=COALESCE($3, description), updated_at=now() WHERE id=$1 RETURNING id")
        .bind(id)
        .bind(req.name)
        .bind(req.description)
        .fetch_one(&st.pool)
        .await;
    match res {
        Ok(r) => {
            let id: Uuid = r.try_get("id").unwrap();
            Json(IdOut { id }).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn roles_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM dora.roles WHERE id=$1")
        .bind(id)
        .execute(&st.pool)
        .await;
    match res {
        Ok(r) => {
            Json(serde_json::json!({"deleted": r.rows_affected()})).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn role_permissions_list(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("SELECT rp.role_id, rp.permission_id, p.name as permission_name, rp.assigned_at FROM dora.role_permissions rp JOIN dora.permissions p ON p.id = rp.permission_id WHERE rp.role_id=$1")
        .bind(id)
        .fetch_all(&st.pool)
        .await;
    match res {
        Ok(rows) => {
            let out: Vec<RolePermissionOut> = rows.into_iter().map(|r| RolePermissionOut {
                role_id: r.try_get("role_id").unwrap(),
                permission_id: r.try_get("permission_id").unwrap(),
                permission_name: r.try_get("permission_name").unwrap(),
                assigned_at: r.try_get("assigned_at").unwrap(),
            }).collect();
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn role_permissions_add(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<RolePermissionAddReq>) -> impl IntoResponse {
    let res = sqlx::query("INSERT INTO dora.role_permissions(role_id, permission_id) VALUES ($1, $2)")
        .bind(id)
        .bind(req.permission_id)
        .execute(&st.pool)
        .await;
    match res {
        Ok(_) => (axum::http::StatusCode::CREATED, Json(serde_json::json!({"status": "assigned"}))).into_response(),
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn role_permissions_remove(State(st): State<AppState>, Path((id, pid)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM dora.role_permissions WHERE role_id=$1 AND permission_id=$2")
        .bind(id)
        .bind(pid)
        .execute(&st.pool)
        .await;
    match res {
        Ok(r) => Json(serde_json::json!({"deleted": r.rows_affected()})).into_response(),
        Err(e) => super::handle_db_error(e),
    }
}
