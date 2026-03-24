use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use sqlx::Row;
use crate::api::AppState;
use crate::models::{IdOut, BuCreateReq, BuUpdateReq, BuOut};

pub async fn business_units_create(State(st): State<AppState>, Json(req): Json<BuCreateReq>) -> impl IntoResponse {
    let res = sqlx::query("INSERT INTO dora.business_units(organization_id, name, description) VALUES ($1, $2, $3) RETURNING id")
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

pub async fn business_units_list(State(st): State<AppState>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, organization_id, name, description, created_at, updated_at FROM dora.business_units ORDER BY name")
        .fetch_all(&st.pool)
        .await;
    match res {
        Ok(rows) => {
            let out: Vec<BuOut> = rows.into_iter().map(|r| BuOut {
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

pub async fn business_units_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, organization_id, name, description, created_at, updated_at FROM dora.business_units WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await;
    match res {
        Ok(r) => {
            let out = BuOut {
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

pub async fn business_units_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<BuUpdateReq>) -> impl IntoResponse {
    let res = sqlx::query("UPDATE dora.business_units SET name=COALESCE($2, name), description=COALESCE($3, description), updated_at=now() WHERE id=$1 RETURNING id")
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

pub async fn business_units_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM dora.business_units WHERE id=$1")
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
