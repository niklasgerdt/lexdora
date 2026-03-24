use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use sqlx::Row;
use crate::api::AppState;
use crate::models::{IdOut, OrgCreateReq, OrgUpdateReq, OrgOut};

pub async fn orgs_create(State(st): State<AppState>, Json(req): Json<OrgCreateReq>) -> impl IntoResponse {
    let res = sqlx::query("INSERT INTO dora.organizations(name, legal_entity_id) VALUES ($1, $2) RETURNING id")
        .bind(req.name)
        .bind(req.legal_entity_id)
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

pub async fn orgs_list(State(st): State<AppState>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, name, legal_entity_id, created_at, updated_at FROM dora.organizations ORDER BY name")
        .fetch_all(&st.pool)
        .await;
    match res {
        Ok(rows) => {
            let out: Vec<OrgOut> = rows.into_iter().map(|r| OrgOut {
                id: r.try_get("id").unwrap(),
                name: r.try_get("name").unwrap(),
                legal_entity_id: r.try_get("legal_entity_id").unwrap_or(None),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            }).collect();
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn orgs_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, name, legal_entity_id, created_at, updated_at FROM dora.organizations WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await;
    match res {
        Ok(r) => {
            let out = OrgOut {
                id: r.try_get("id").unwrap(),
                name: r.try_get("name").unwrap(),
                legal_entity_id: r.try_get("legal_entity_id").unwrap_or(None),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            };
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn orgs_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<OrgUpdateReq>) -> impl IntoResponse {
    let res = sqlx::query("UPDATE dora.organizations SET name=COALESCE($2, name), legal_entity_id=COALESCE($3, legal_entity_id), updated_at=now() WHERE id=$1 RETURNING id")
        .bind(id)
        .bind(req.name)
        .bind(req.legal_entity_id)
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

pub async fn orgs_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM dora.organizations WHERE id=$1")
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
