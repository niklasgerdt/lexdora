use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use sqlx::Row;
use crate::api::AppState;
use crate::models::{IdOut, UserCreateReq, UserUpdateReq, UserOut, UserRoleAddReq, UserRoleOut};

pub async fn users_create(State(st): State<AppState>, Json(req): Json<UserCreateReq>) -> impl IntoResponse {
    let res = sqlx::query("INSERT INTO dora.users(organization_id, email, full_name) VALUES ($1, $2, $3) RETURNING id")
        .bind(req.organization_id)
        .bind(req.email)
        .bind(req.full_name)
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

pub async fn users_list(State(st): State<AppState>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, organization_id, email, full_name, is_active, created_at, updated_at FROM dora.users ORDER BY email")
        .fetch_all(&st.pool)
        .await;
    match res {
        Ok(rows) => {
            let out: Vec<UserOut> = rows.into_iter().map(|r| UserOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                email: r.try_get("email").unwrap(),
                full_name: r.try_get("full_name").unwrap_or(None),
                is_active: r.try_get("is_active").unwrap(),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            }).collect();
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn users_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, organization_id, email, full_name, is_active, created_at, updated_at FROM dora.users WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await;
    match res {
        Ok(r) => {
            let out = UserOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                email: r.try_get("email").unwrap(),
                full_name: r.try_get("full_name").unwrap_or(None),
                is_active: r.try_get("is_active").unwrap(),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            };
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn users_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UserUpdateReq>) -> impl IntoResponse {
    let res = sqlx::query("UPDATE dora.users SET email=COALESCE($2, email), full_name=COALESCE($3, full_name), is_active=COALESCE($4, is_active), updated_at=now() WHERE id=$1 RETURNING id")
        .bind(id)
        .bind(req.email)
        .bind(req.full_name)
        .bind(req.is_active)
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

pub async fn users_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM dora.users WHERE id=$1")
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

pub async fn user_roles_list(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("SELECT ur.user_id, ur.role_id, r.name as role_name, ur.assigned_at FROM dora.user_roles ur JOIN dora.roles r ON r.id = ur.role_id WHERE ur.user_id=$1")
        .bind(id)
        .fetch_all(&st.pool)
        .await;
    match res {
        Ok(rows) => {
            let out: Vec<UserRoleOut> = rows.into_iter().map(|r| UserRoleOut {
                user_id: r.try_get("user_id").unwrap(),
                role_id: r.try_get("role_id").unwrap(),
                role_name: r.try_get("role_name").unwrap(),
                assigned_at: r.try_get("assigned_at").unwrap(),
            }).collect();
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn user_roles_add(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UserRoleAddReq>) -> impl IntoResponse {
    let res = sqlx::query("INSERT INTO dora.user_roles(user_id, role_id) VALUES ($1, $2)")
        .bind(id)
        .bind(req.role_id)
        .execute(&st.pool)
        .await;
    match res {
        Ok(_) => (axum::http::StatusCode::CREATED, Json(serde_json::json!({"status": "assigned"}))).into_response(),
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn user_roles_remove(State(st): State<AppState>, Path((id, rid)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM dora.user_roles WHERE user_id=$1 AND role_id=$2")
        .bind(id)
        .bind(rid)
        .execute(&st.pool)
        .await;
    match res {
        Ok(r) => Json(serde_json::json!({"deleted": r.rows_affected()})).into_response(),
        Err(e) => super::handle_db_error(e),
    }
}
