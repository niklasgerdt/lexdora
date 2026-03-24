use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::OrgCmd;
use crate::models::{IdOut, OrgOut};

pub async fn org_handlers(pool: &PgPool, cmd: OrgCmd) -> Result<()> {
    match cmd {
        OrgCmd::Create { name, legal_entity_id } => {
            let row = sqlx::query(
                "INSERT INTO dora.organizations(name, legal_entity_id) VALUES ($1, $2) RETURNING id",
            )
            .bind(name)
            .bind(legal_entity_id)
            .fetch_one(pool)
            .await?;
            let id: Uuid = row.try_get("id")?;
            println!("{}", serde_json::to_string(&IdOut { id })?);
        }
        OrgCmd::List => {
            let rows = sqlx::query("SELECT id, name, legal_entity_id, created_at, updated_at FROM dora.organizations ORDER BY name")
                .fetch_all(pool)
                .await?;
            let out: Vec<OrgOut> = rows
                .into_iter()
                .map(|r| OrgOut {
                    id: r.try_get("id").unwrap(),
                    name: r.try_get("name").unwrap(),
                    legal_entity_id: r.try_get("legal_entity_id").unwrap_or(None),
                    created_at: r.try_get("created_at").unwrap(),
                    updated_at: r.try_get("updated_at").unwrap(),
                })
                .collect();
            println!("{}", serde_json::to_string(&out)?);
        }
        OrgCmd::Get { id } => {
            let r = sqlx::query("SELECT id, name, legal_entity_id, created_at, updated_at FROM dora.organizations WHERE id=$1")
                .bind(id)
                .fetch_one(pool)
                .await?;
            let out = OrgOut {
                id: r.try_get("id")?,
                name: r.try_get("name")?,
                legal_entity_id: r.try_get::<Option<String>, _>("legal_entity_id")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
            };
            println!("{}", serde_json::to_string(&out)?);
        }
        OrgCmd::Update { id, name, legal_entity_id } => {
            let row = sqlx::query(
                "UPDATE dora.organizations SET name=COALESCE($2, name), legal_entity_id=COALESCE($3, legal_entity_id), updated_at=now() WHERE id=$1 RETURNING id",
            )
            .bind(id)
            .bind(name)
            .bind(legal_entity_id)
            .fetch_one(pool)
            .await?;
            let id: Uuid = row.try_get("id")?;
            println!("{}", serde_json::to_string(&IdOut { id })?);
        }
        OrgCmd::Delete { id } => {
            let res = sqlx::query("DELETE FROM dora.organizations WHERE id=$1")
                .bind(id)
                .execute(pool)
                .await?;
            println!("{}", serde_json::to_string(&serde_json::json!({"deleted": res.rows_affected()}))?);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::db_pool;

    async fn setup_pool() -> PgPool {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        db_pool(&db_url).await.expect("Failed to create pool")
    }

    #[tokio::test]
    async fn test_org_handlers() {
        let pool = setup_pool().await;
        let name = format!("CLI Org {}", Uuid::new_v4());
        
        // Test Create
        let cmd = OrgCmd::Create { name: name.clone(), legal_entity_id: None };
        org_handlers(&pool, cmd).await.unwrap();

        // Test List
        let cmd = OrgCmd::List;
        org_handlers(&pool, cmd).await.unwrap();
    }
}
