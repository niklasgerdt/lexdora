use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use sqlx::Row;
use std::str::FromStr;
use chrono::{DateTime, Utc};
use crate::api::AppState;
use crate::models::{IdOut, IncidentCreateReq, IncidentUpdateReq, IncidentOut};

pub async fn incidents_create(State(st): State<AppState>, Json(req): Json<IncidentCreateReq>) -> impl IntoResponse {
    let detected: DateTime<Utc> = if req.detected_at.to_lowercase() == "now" {
        Utc::now()
    } else {
        match DateTime::from_str(&req.detected_at) {
            Ok(dt) => dt,
            Err(_) => return (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "invalid detected_at; use RFC3339 or 'now'"}))).into_response(),
        }
    };
    let res = sqlx::query("INSERT INTO dora.incidents(organization_id, title, description, type, severity, detected_at, is_major) VALUES ($1,$2,$3,$4::dora.incident_type,$5::dora.incident_severity,$6,$7) RETURNING id")
        .bind(req.organization_id)
        .bind(req.title)
        .bind(req.description)
        .bind(req.type_)
        .bind(req.severity)
        .bind(detected)
        .bind(req.is_major.unwrap_or(false))
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

pub async fn incidents_list(State(st): State<AppState>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, organization_id, title, description, type::text, severity::text, detected_at, is_major, created_at, updated_at FROM dora.incidents ORDER BY detected_at DESC")
        .fetch_all(&st.pool)
        .await;
    match res {
        Ok(rows) => {
            let out: Vec<IncidentOut> = rows.into_iter().map(|r| IncidentOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                title: r.try_get("title").unwrap(),
                description: r.try_get("description").unwrap_or(None),
                type_: r.try_get("type").unwrap(),
                severity: r.try_get("severity").unwrap(),
                detected_at: r.try_get("detected_at").unwrap(),
                is_major: r.try_get("is_major").unwrap(),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            }).collect();
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn incidents_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("SELECT id, organization_id, title, description, type::text, severity::text, detected_at, is_major, created_at, updated_at FROM dora.incidents WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await;
    match res {
        Ok(r) => {
            let out = IncidentOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                title: r.try_get("title").unwrap(),
                description: r.try_get("description").unwrap_or(None),
                type_: r.try_get("type").unwrap(),
                severity: r.try_get("severity").unwrap(),
                detected_at: r.try_get("detected_at").unwrap(),
                is_major: r.try_get("is_major").unwrap(),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            };
            Json(out).into_response()
        }
        Err(e) => super::handle_db_error(e),
    }
}

pub async fn incidents_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<IncidentUpdateReq>) -> impl IntoResponse {
    let res = sqlx::query("UPDATE dora.incidents SET title=COALESCE($2, title), description=COALESCE($3, description), type=COALESCE($4::dora.incident_type, type), severity=COALESCE($5::dora.incident_severity, severity), is_major=COALESCE($6, is_major), updated_at=now() WHERE id=$1 RETURNING id")
        .bind(id)
        .bind(req.title)
        .bind(req.description)
        .bind(req.type_)
        .bind(req.severity)
        .bind(req.is_major)
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

pub async fn incidents_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM dora.incidents WHERE id=$1")
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
