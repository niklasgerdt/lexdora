use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use sqlx::Row;
use crate::api::AppState;
use crate::models::{IdOut, AssetCreateReq, AssetUpdateReq, AssetOut};

pub async fn assets_create(State(st): State<AppState>, Json(req): Json<AssetCreateReq>) -> impl IntoResponse {
    let res = sqlx::query("INSERT INTO dora.ict_assets(organization_id, name, description, criticality, owner_id) VALUES ($1,$2,$3,$4::dora.asset_criticality,$5) RETURNING id")
        .bind(req.organization_id)
        .bind(req.name)
        .bind(req.description)
        .bind(req.criticality)
        .bind(req.owner_id)
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

pub async fn assets_list(State(st): State<AppState>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, organization_id, name, description, criticality::text, owner_id, created_at, updated_at FROM dora.ict_assets ORDER BY name")
        .fetch_all(&st.pool)
        .await;
    match res {
        Ok(rows) => {
            let out: Vec<AssetOut> = rows.into_iter().map(|r| AssetOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                name: r.try_get("name").unwrap(),
                description: r.try_get("description").unwrap_or(None),
                criticality: r.try_get("criticality").unwrap_or(None),
                owner_id: r.try_get("owner_id").unwrap_or(None),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            }).collect();
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn assets_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, organization_id, name, description, criticality::text, owner_id, created_at, updated_at FROM dora.ict_assets WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await;
    match res {
        Ok(r) => {
            let out = AssetOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                name: r.try_get("name").unwrap(),
                description: r.try_get("description").unwrap_or(None),
                criticality: r.try_get("criticality").unwrap_or(None),
                owner_id: r.try_get("owner_id").unwrap_or(None),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            };
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn assets_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<AssetUpdateReq>) -> impl IntoResponse {
    let res = sqlx::query("UPDATE dora.ict_assets SET name=COALESCE($2, name), description=COALESCE($3, description), criticality=COALESCE($4::dora.asset_criticality, criticality), owner_id=COALESCE($5, owner_id), updated_at=now() WHERE id=$1 RETURNING id")
        .bind(id)
        .bind(req.name)
        .bind(req.description)
        .bind(req.criticality)
        .bind(req.owner_id)
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

pub async fn assets_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM dora.ict_assets WHERE id=$1")
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
