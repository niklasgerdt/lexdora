use std::str::FromStr;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::IncidentCmd;
use crate::models::{IdOut, IncidentOut};

pub async fn incident_handlers(pool: &PgPool, cmd: IncidentCmd) -> Result<()> {
    match cmd {
        IncidentCmd::Create { organization_id, title, type_, severity, detected_at, description, is_major } => {
            let detected: DateTime<Utc> = if detected_at.to_lowercase() == "now" {
                Utc::now()
            } else {
                DateTime::from_str(&detected_at)
                    .with_context(|| "Invalid detected_at; use RFC3339 or 'now'")?
            };
            let row = sqlx::query(
                "INSERT INTO dora.incidents(organization_id, title, description, type, severity, detected_at, is_major) VALUES ($1,$2,$3,$4::dora.incident_type,$5::dora.incident_severity,$6,$7) RETURNING id",
            )
            .bind(organization_id)
            .bind(title)
            .bind(description)
            .bind(type_)
            .bind(severity)
            .bind(detected)
            .bind(is_major.unwrap_or(false))
            .fetch_one(pool)
            .await?;
            let id: Uuid = row.try_get("id")?;
            println!("{}", serde_json::to_string(&IdOut { id })?);
        }
        IncidentCmd::List { organization_id } => {
            let query = if let Some(org) = organization_id {
                sqlx::query("SELECT id, organization_id, title, description, type::text, severity::text, detected_at, is_major, created_at, updated_at FROM dora.incidents WHERE organization_id=$1 ORDER BY detected_at DESC")
                    .bind(org)
            } else {
                sqlx::query("SELECT id, organization_id, title, description, type::text, severity::text, detected_at, is_major, created_at, updated_at FROM dora.incidents ORDER BY detected_at DESC")
            };
            let rows = query.fetch_all(pool).await?;
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
            println!("{}", serde_json::to_string(&out)?);
        }
        IncidentCmd::Get { id } => {
            let r = sqlx::query("SELECT id, organization_id, title, description, type::text, severity::text, detected_at, is_major, created_at, updated_at FROM dora.incidents WHERE id=$1")
                .bind(id)
                .fetch_one(pool)
                .await?;
            let out = IncidentOut {
                id: r.try_get("id")?,
                organization_id: r.try_get("organization_id")?,
                title: r.try_get("title")?,
                description: r.try_get::<Option<String>, _>("description")?,
                type_: r.try_get("type")?,
                severity: r.try_get("severity")?,
                detected_at: r.try_get("detected_at")?,
                is_major: r.try_get("is_major")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
            };
            println!("{}", serde_json::to_string(&out)?);
        }
        IncidentCmd::Update { id, title, type_, severity, description, is_major } => {
            let row = sqlx::query(
                "UPDATE dora.incidents SET title=COALESCE($2, title), description=COALESCE($3, description), type=COALESCE($4::dora.incident_type, type), severity=COALESCE($5::dora.incident_severity, severity), is_major=COALESCE($6, is_major), updated_at=now() WHERE id=$1 RETURNING id",
            )
            .bind(id)
            .bind(title)
            .bind(description)
            .bind(type_)
            .bind(severity)
            .bind(is_major)
            .fetch_one(pool)
            .await?;
            let id: Uuid = row.try_get("id")?;
            println!("{}", serde_json::to_string(&IdOut { id })?);
        }
        IncidentCmd::Delete { id } => {
            let res = sqlx::query("DELETE FROM dora.incidents WHERE id=$1")
                .bind(id)
                .execute(pool)
                .await?;
            println!("{}", serde_json::to_string(&serde_json::json!({"deleted": res.rows_affected()}))?);
        }
    }
    Ok(())
}
