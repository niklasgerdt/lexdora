use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::TppCmd;
use crate::models::{IdOut, TppOut};

pub async fn tpp_handlers(pool: &PgPool, cmd: TppCmd) -> Result<()> {
    match cmd {
        TppCmd::Create { organization_id, name, country, criticality, is_important } => {
            let row = sqlx::query(
                "INSERT INTO dora.third_party_providers(organization_id, name, country, criticality, is_important) VALUES ($1,$2,$3,$4::dora.tpp_criticality,$5) RETURNING id",
            )
            .bind(organization_id)
            .bind(name)
            .bind(country)
            .bind(criticality)
            .bind(is_important.unwrap_or(false))
            .fetch_one(pool)
            .await?;
            let id: Uuid = row.try_get("id")?;
            println!("{}", serde_json::to_string(&IdOut { id })?);
        }
        TppCmd::List { organization_id } => {
            let query = if let Some(org) = organization_id {
                sqlx::query("SELECT id, organization_id, name, country, criticality::text, is_important, created_at, updated_at FROM dora.third_party_providers WHERE organization_id=$1 ORDER BY name")
                    .bind(org)
            } else {
                sqlx::query("SELECT id, organization_id, name, country, criticality::text, is_important, created_at, updated_at FROM dora.third_party_providers ORDER BY name")
            };
            let rows = query.fetch_all(pool).await?;
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
            println!("{}", serde_json::to_string(&out)?);
        }
        TppCmd::Get { id } => {
            let r = sqlx::query("SELECT id, organization_id, name, country, criticality::text, is_important, created_at, updated_at FROM dora.third_party_providers WHERE id=$1")
                .bind(id)
                .fetch_one(pool)
                .await?;
            let out = TppOut {
                id: r.try_get("id")?,
                organization_id: r.try_get("organization_id")?,
                name: r.try_get("name")?,
                country: r.try_get::<Option<String>, _>("country")?,
                criticality: r.try_get::<Option<String>, _>("criticality")?,
                is_important: r.try_get("is_important")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
            };
            println!("{}", serde_json::to_string(&out)?);
        }
        TppCmd::Update { id, name, country, criticality, is_important } => {
            let row = sqlx::query(
                "UPDATE dora.third_party_providers SET name=COALESCE($2, name), country=COALESCE($3, country), criticality=COALESCE($4::dora.tpp_criticality, criticality), is_important=COALESCE($5, is_important), updated_at=now() WHERE id=$1 RETURNING id",
            )
            .bind(id)
            .bind(name)
            .bind(country)
            .bind(criticality)
            .bind(is_important)
            .fetch_one(pool)
            .await?;
            let id: Uuid = row.try_get("id")?;
            println!("{}", serde_json::to_string(&IdOut { id })?);
        }
        TppCmd::Delete { id } => {
            let res = sqlx::query("DELETE FROM dora.third_party_providers WHERE id=$1")
                .bind(id)
                .execute(pool)
                .await?;
            println!("{}", serde_json::to_string(&serde_json::json!({"deleted": res.rows_affected()}))?);
        }
    }
    Ok(())
}
