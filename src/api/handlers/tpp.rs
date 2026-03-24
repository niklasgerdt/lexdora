use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use sqlx::Row;
use crate::api::AppState;
use crate::models::{IdOut, TppCreateReq, TppUpdateReq, TppOut};

pub async fn tpps_create(State(st): State<AppState>, Json(req): Json<TppCreateReq>) -> impl IntoResponse {
    let res = sqlx::query("INSERT INTO dora.third_party_providers(organization_id, name, country, criticality, is_important) VALUES ($1,$2,$3,$4::dora.tpp_criticality,$5) RETURNING id")
        .bind(req.organization_id)
        .bind(req.name)
        .bind(req.country)
        .bind(req.criticality)
        .bind(req.is_important.unwrap_or(false))
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

pub async fn tpps_list(State(st): State<AppState>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, organization_id, name, country, criticality::text, is_important, created_at, updated_at FROM dora.third_party_providers ORDER BY name")
        .fetch_all(&st.pool)
        .await;
    match res {
        Ok(rows) => {
            let out: Vec<TppOut> = rows.into_iter().map(|r| TppOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                name: r.try_get("name").unwrap(),
                country: r.try_get("country").unwrap_or(None),
                criticality: r.try_get("criticality").unwrap_or(None),
                is_important: r.try_get("is_important").unwrap(),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            }).collect();
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn tpps_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, organization_id, name, country, criticality::text, is_important, created_at, updated_at FROM dora.third_party_providers WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await;
    match res {
        Ok(r) => {
            let out = TppOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                name: r.try_get("name").unwrap(),
                country: r.try_get("country").unwrap_or(None),
                criticality: r.try_get("criticality").unwrap_or(None),
                is_important: r.try_get("is_important").unwrap(),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            };
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn tpps_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<TppUpdateReq>) -> impl IntoResponse {
    let res = sqlx::query("UPDATE dora.third_party_providers SET name=COALESCE($2, name), country=COALESCE($3, country), criticality=COALESCE($4::dora.tpp_criticality, criticality), is_important=COALESCE($5, is_important), updated_at=now() WHERE id=$1 RETURNING id")
        .bind(id)
        .bind(req.name)
        .bind(req.country)
        .bind(req.criticality)
        .bind(req.is_important)
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

pub async fn tpps_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM dora.third_party_providers WHERE id=$1")
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
