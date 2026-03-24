use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::UserCmd;
use crate::models::{IdOut, UserOut};

pub async fn user_handlers(pool: &PgPool, cmd: UserCmd) -> Result<()> {
    match cmd {
        UserCmd::Create { organization_id, email, full_name } => {
            let row = sqlx::query(
                "INSERT INTO dora.users(organization_id, email, full_name) VALUES ($1, $2, $3) RETURNING id",
            )
            .bind(organization_id)
            .bind(email)
            .bind(full_name)
            .fetch_one(pool)
            .await?;
            let id: Uuid = row.try_get("id")?;
            println!("{}", serde_json::to_string(&IdOut { id })?);
        }
        UserCmd::List { organization_id } => {
            if let Some(org) = organization_id {
                let rows = sqlx::query("SELECT id, organization_id, email, full_name, is_active, created_at, updated_at FROM dora.users WHERE organization_id=$1 ORDER BY email")
                    .bind(org)
                    .fetch_all(pool)
                    .await?;
                let out: Vec<UserOut> = rows.into_iter().map(|r| UserOut{
                    id: r.try_get("id").unwrap(),
                    organization_id: r.try_get("organization_id").unwrap(),
                    email: r.try_get("email").unwrap(),
                    full_name: r.try_get("full_name").unwrap_or(None),
                    is_active: r.try_get("is_active").unwrap(),
                    created_at: r.try_get("created_at").unwrap(),
                    updated_at: r.try_get("updated_at").unwrap(),
                }).collect();
                println!("{}", serde_json::to_string(&out)?);
            } else {
                let rows = sqlx::query("SELECT id, organization_id, email, full_name, is_active, created_at, updated_at FROM dora.users ORDER BY email")
                    .fetch_all(pool)
                    .await?;
                let out: Vec<UserOut> = rows.into_iter().map(|r| UserOut{
                    id: r.try_get("id").unwrap(),
                    organization_id: r.try_get("organization_id").unwrap(),
                    email: r.try_get("email").unwrap(),
                    full_name: r.try_get("full_name").unwrap_or(None),
                    is_active: r.try_get("is_active").unwrap(),
                    created_at: r.try_get("created_at").unwrap(),
                    updated_at: r.try_get("updated_at").unwrap(),
                }).collect();
                println!("{}", serde_json::to_string(&out)?);
            }
        }
        UserCmd::Get { id } => {
            let r = sqlx::query("SELECT id, organization_id, email, full_name, is_active, created_at, updated_at FROM dora.users WHERE id=$1")
                .bind(id)
                .fetch_one(pool)
                .await?;
            let out = UserOut {
                id: r.try_get("id")?,
                organization_id: r.try_get("organization_id")?,
                email: r.try_get("email")?,
                full_name: r.try_get::<Option<String>, _>("full_name")?,
                is_active: r.try_get("is_active")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
            };
            println!("{}", serde_json::to_string(&out)?);
        }
        UserCmd::Update { id, email, full_name, is_active } => {
            let row = sqlx::query(
                "UPDATE dora.users SET email=COALESCE($2, email), full_name=COALESCE($3, full_name), is_active=COALESCE($4, is_active), updated_at=now() WHERE id=$1 RETURNING id",
            )
            .bind(id)
            .bind(email)
            .bind(full_name)
            .bind(is_active)
            .fetch_one(pool)
            .await?;
            let id: Uuid = row.try_get("id")?;
            println!("{}", serde_json::to_string(&IdOut { id })?);
        }
        UserCmd::Delete { id } => {
            let res = sqlx::query("DELETE FROM dora.users WHERE id=$1")
                .bind(id)
                .execute(pool)
                .await?;
            println!("{}", serde_json::to_string(&serde_json::json!({"deleted": res.rows_affected()}))?);
        }
    }
    Ok(())
}
