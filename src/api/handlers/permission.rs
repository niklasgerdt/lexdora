use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use sqlx::Row;
use crate::api::AppState;
use crate::models::{IdOut, PermissionCreateReq, PermissionUpdateReq, PermissionOut};

pub async fn permissions_create(State(st): State<AppState>, Json(req): Json<PermissionCreateReq>) -> impl IntoResponse {
    let res = sqlx::query("INSERT INTO dora.permissions(name, description) VALUES ($1, $2) RETURNING id")
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

pub async fn permissions_list(State(st): State<AppState>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, name, description, created_at, updated_at FROM dora.permissions ORDER BY name")
        .fetch_all(&st.pool)
        .await;
    match res {
        Ok(rows) => {
            let out: Vec<PermissionOut> = rows.into_iter().map(|r| PermissionOut {
                id: r.try_get("id").unwrap(),
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

pub async fn permissions_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, name, description, created_at, updated_at FROM dora.permissions WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await;
    match res {
        Ok(r) => {
            let out = PermissionOut {
                id: r.try_get("id").unwrap(),
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

pub async fn permissions_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<PermissionUpdateReq>) -> impl IntoResponse {
    let res = sqlx::query("UPDATE dora.permissions SET name=COALESCE($2, name), description=COALESCE($3, description), updated_at=now() WHERE id=$1 RETURNING id")
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

pub async fn permissions_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM dora.permissions WHERE id=$1")
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
