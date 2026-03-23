use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "dora-cli", version, about = "Simple CRUD CLI for DORA schema (PostgreSQL)")]
struct Cli {
    /// Database URL, e.g. postgres://user:pass@localhost:5432/db
    #[arg(long)]
    database_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Org { #[command(subcommand)] cmd: OrgCmd },
    User { #[command(subcommand)] cmd: UserCmd },
    Incident { #[command(subcommand)] cmd: IncidentCmd },
    Tpp { #[command(subcommand)] cmd: TppCmd },
    /// Start HTTP REST server
    Serve { #[arg(long)] bind: Option<String> },
}

#[derive(Subcommand, Debug)]
enum OrgCmd {
    Create { name: String, #[arg(long)] legal_entity_id: Option<String> },
    List,
    Get { id: Uuid },
    Update { id: Uuid, #[arg(long)] name: Option<String>, #[arg(long)] legal_entity_id: Option<String> },
    Delete { id: Uuid },
}

#[derive(Subcommand, Debug)]
enum UserCmd {
    Create { organization_id: Uuid, email: String, #[arg(long)] full_name: Option<String> },
    List { #[arg(long)] organization_id: Option<Uuid> },
    Get { id: Uuid },
    Update { id: Uuid, #[arg(long)] email: Option<String>, #[arg(long)] full_name: Option<String>, #[arg(long)] is_active: Option<bool> },
    Delete { id: Uuid },
}

#[derive(Subcommand, Debug)]
enum IncidentCmd {
    Create {
        organization_id: Uuid,
        title: String,
        #[arg(long, value_name = "incident_type")] type_: String,
        #[arg(long, value_name = "incident_severity")] severity: String,
        #[arg(long, value_name = "RFC3339|now")] detected_at: String,
        #[arg(long)] description: Option<String>,
        #[arg(long)] is_major: Option<bool>,
    },
    List { #[arg(long)] organization_id: Option<Uuid> },
    Get { id: Uuid },
    Update {
        id: Uuid,
        #[arg(long)] title: Option<String>,
        #[arg(long, value_name = "incident_type")] type_: Option<String>,
        #[arg(long, value_name = "incident_severity")] severity: Option<String>,
        #[arg(long)] description: Option<String>,
        #[arg(long)] is_major: Option<bool>,
    },
    Delete { id: Uuid },
}

#[derive(Subcommand, Debug)]
enum TppCmd {
    Create { organization_id: Uuid, name: String, #[arg(long)] country: Option<String>, #[arg(long)] criticality: Option<String>, #[arg(long)] is_important: Option<bool> },
    List { #[arg(long)] organization_id: Option<Uuid> },
    Get { id: Uuid },
    Update { id: Uuid, #[arg(long)] name: Option<String>, #[arg(long)] country: Option<String>, #[arg(long)] criticality: Option<String>, #[arg(long)] is_important: Option<bool> },
    Delete { id: Uuid },
}

#[derive(Serialize)]
struct IdOut { id: Uuid }

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let cli = Cli::parse();
    let database_url = if let Some(url) = cli.database_url {
        url
    } else {
        std::env::var("DATABASE_URL").context("DATABASE_URL not provided (flag --database-url or env var)")?
    };
    let pool = db_pool(&database_url).await?;

    match cli.command {
        Commands::Org { cmd } => org_handlers(&pool, cmd).await?,
        Commands::User { cmd } => user_handlers(&pool, cmd).await?,
        Commands::Incident { cmd } => incident_handlers(&pool, cmd).await?,
        Commands::Tpp { cmd } => tpp_handlers(&pool, cmd).await?,
        Commands::Serve { bind } => {
            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .init();
            let addr: SocketAddr = bind
                .unwrap_or_else(|| "0.0.0.0:8080".to_string())
                .parse()
                .with_context(|| "Invalid --bind address, use host:port")?;
            let app = build_router(pool.clone());
            info!(%addr, "Starting server");
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await
                .context("Server error")?;
        }
    }

    Ok(())
}

async fn db_pool(url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .with_context(|| "Failed to connect to DATABASE_URL")?;
    Ok(pool)
}

async fn org_handlers(pool: &PgPool, cmd: OrgCmd) -> Result<()> {
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
            #[derive(Serialize)]
            struct OrgOut { id: Uuid, name: String, legal_entity_id: Option<String>, created_at: DateTime<Utc>, updated_at: DateTime<Utc> }
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
            #[derive(Serialize)]
            struct OrgOut { id: Uuid, name: String, legal_entity_id: Option<String>, created_at: DateTime<Utc>, updated_at: DateTime<Utc> }
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

// ===== REST server =====

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

fn build_router(pool: PgPool) -> Router {
    let state = AppState { pool };
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let api = Router::new()
        .route("/healthz", get(healthz))
        // organizations
        .route("/orgs", get(orgs_list).post(orgs_create))
        .route("/orgs/:id", get(orgs_get).patch(orgs_update).delete(orgs_delete))
        // users
        .route("/users", get(users_list).post(users_create))
        .route("/users/:id", get(users_get).patch(users_update).delete(users_delete))
        // incidents
        .route("/incidents", get(incidents_list).post(incidents_create))
        .route(
            "/incidents/:id",
            get(incidents_get).patch(incidents_update).delete(incidents_delete),
        )
        // business units
        .route("/business-units", get(business_units_list).post(business_units_create))
        .route(
            "/business-units/:id",
            get(business_units_get).patch(business_units_update).delete(business_units_delete),
        )
        // roles
        .route("/roles", get(roles_list).post(roles_create))
        .route(
            "/roles/:id",
            get(roles_get).patch(roles_update).delete(roles_delete),
        )
        .route(
            "/roles/:id/permissions",
            get(role_permissions_list).post(role_permissions_add),
        )
        .route(
            "/roles/:id/permissions/:pid",
            delete(role_permissions_remove),
        )
        // permissions
        .route("/permissions", get(permissions_list).post(permissions_create))
        .route(
            "/permissions/:id",
            get(permissions_get).patch(permissions_update).delete(permissions_delete),
        )
        // user-role assignments
        .route(
            "/users/:id/roles",
            get(user_roles_list).post(user_roles_add),
        )
        .route(
            "/users/:id/roles/:rid",
            delete(user_roles_remove),
        )
        // third parties
        .route("/tpps", get(tpps_list).post(tpps_create))
        .route("/tpps/:id", get(tpps_get).patch(tpps_update).delete(tpps_delete))
        // ict assets
        .route("/assets", get(assets_list).post(assets_create))
        .route("/assets/:id", get(assets_get).patch(assets_update).delete(assets_delete))
        .with_state(state.clone());

    // Static site (Rust WASM generated files expected under web/ with pkg from wasm-pack)
    let static_dir = ServeDir::new("web").not_found_service(ServeFile::new("web/index.html"));

    Router::new()
        .merge(api)
        .nest_service("/", static_dir)
        .layer(cors)
        .with_state(state)
}

async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = async {
        signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {}, }
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

// ========== Organizations ==========
#[derive(Deserialize)]
struct OrgCreateReq {
    name: String,
    #[serde(default)]
    legal_entity_id: Option<String>,
}
#[derive(Deserialize)]
struct OrgUpdateReq {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    legal_entity_id: Option<String>,
}

#[derive(Serialize)]
struct OrgOut {
    id: Uuid,
    name: String,
    legal_entity_id: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

async fn orgs_create(State(st): State<AppState>, Json(req): Json<OrgCreateReq>) -> impl IntoResponse {
    match sqlx::query("INSERT INTO dora.organizations(name, legal_entity_id) VALUES ($1, $2) RETURNING id")
        .bind(req.name)
        .bind(req.legal_entity_id)
        .fetch_one(&st.pool)
        .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::CREATED, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(e) => db_error(e),
    }
}

// ========== Business Units ==========
#[derive(Deserialize)]
struct BuCreateReq {
    organization_id: Uuid,
    name: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
struct BuUpdateReq {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Serialize)]
struct BuOut {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct BuListParams { organization_id: Option<Uuid> }

async fn business_units_create(State(st): State<AppState>, Json(req): Json<BuCreateReq>) -> impl IntoResponse {
    match sqlx::query(
        "INSERT INTO dora.business_units(organization_id, name, description) VALUES ($1,$2,$3) RETURNING id",
    )
    .bind(req.organization_id)
    .bind(req.name)
    .bind(req.description)
    .fetch_one(&st.pool)
    .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::CREATED, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn business_units_list(State(st): State<AppState>, Query(p): Query<BuListParams>) -> impl IntoResponse {
    let query = if let Some(org) = p.organization_id {
        sqlx::query("SELECT id, organization_id, name, description FROM dora.business_units WHERE organization_id=$1 ORDER BY name").bind(org)
    } else {
        sqlx::query("SELECT id, organization_id, name, description FROM dora.business_units ORDER BY name")
    };
    match query.fetch_all(&st.pool).await {
        Ok(rows) => {
            let out: Vec<BuOut> = rows
                .into_iter()
                .map(|r| BuOut {
                    id: r.try_get("id").unwrap(),
                    organization_id: r.try_get("organization_id").unwrap(),
                    name: r.try_get("name").unwrap(),
                    description: r.try_get("description").unwrap_or(None),
                })
                .collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn business_units_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("SELECT id, organization_id, name, description FROM dora.business_units WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await
    {
        Ok(r) => {
            let out = BuOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                name: r.try_get("name").unwrap(),
                description: r.try_get("description").unwrap_or(None),
            };
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn business_units_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<BuUpdateReq>) -> impl IntoResponse {
    match sqlx::query(
        "UPDATE dora.business_units SET name=COALESCE($2,name), description=COALESCE($3,description), updated_at=now() WHERE id=$1 RETURNING id",
    )
    .bind(id)
    .bind(req.name)
    .bind(req.description)
    .fetch_one(&st.pool)
    .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::OK, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn business_units_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM dora.business_units WHERE id=$1").bind(id).execute(&st.pool).await {
        Ok(res) => (StatusCode::OK, Json(serde_json::json!({"deleted": res.rows_affected()}))).into_response(),
        Err(e) => db_error(e),
    }
}

// ========== Roles ==========
#[derive(Deserialize)]
struct RoleCreateReq {
    name: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
struct RoleUpdateReq {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Serialize)]
struct RoleOut {
    id: Uuid,
    name: String,
    description: Option<String>,
}

async fn roles_create(State(st): State<AppState>, Json(req): Json<RoleCreateReq>) -> impl IntoResponse {
    match sqlx::query("INSERT INTO dora.roles(name, description) VALUES ($1,$2) RETURNING id")
        .bind(req.name)
        .bind(req.description)
        .fetch_one(&st.pool)
        .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::CREATED, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn roles_list(State(st): State<AppState>) -> impl IntoResponse {
    match sqlx::query("SELECT id, name, description FROM dora.roles ORDER BY name").fetch_all(&st.pool).await {
        Ok(rows) => {
            let out: Vec<RoleOut> = rows
                .into_iter()
                .map(|r| RoleOut {
                    id: r.try_get("id").unwrap(),
                    name: r.try_get("name").unwrap(),
                    description: r.try_get("description").unwrap_or(None),
                })
                .collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn roles_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("SELECT id, name, description FROM dora.roles WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await
    {
        Ok(r) => {
            let out = RoleOut {
                id: r.try_get("id").unwrap(),
                name: r.try_get("name").unwrap(),
                description: r.try_get("description").unwrap_or(None),
            };
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn roles_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<RoleUpdateReq>) -> impl IntoResponse {
    match sqlx::query("UPDATE dora.roles SET name=COALESCE($2,name), description=COALESCE($3,description) WHERE id=$1 RETURNING id")
        .bind(id)
        .bind(req.name)
        .bind(req.description)
        .fetch_one(&st.pool)
        .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::OK, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn roles_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM dora.roles WHERE id=$1").bind(id).execute(&st.pool).await {
        Ok(res) => (StatusCode::OK, Json(serde_json::json!({"deleted": res.rows_affected()}))).into_response(),
        Err(e) => db_error(e),
    }
}

// ========== Permissions ==========
#[derive(Deserialize)]
struct PermissionCreateReq {
    name: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
struct PermissionUpdateReq {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Serialize)]
struct PermissionOut {
    id: Uuid,
    name: String,
    description: Option<String>,
}

async fn permissions_create(State(st): State<AppState>, Json(req): Json<PermissionCreateReq>) -> impl IntoResponse {
    match sqlx::query("INSERT INTO dora.permissions(name, description) VALUES ($1,$2) RETURNING id")
        .bind(req.name)
        .bind(req.description)
        .fetch_one(&st.pool)
        .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::CREATED, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn permissions_list(State(st): State<AppState>) -> impl IntoResponse {
    match sqlx::query("SELECT id, name, description FROM dora.permissions ORDER BY name").fetch_all(&st.pool).await {
        Ok(rows) => {
            let out: Vec<PermissionOut> = rows
                .into_iter()
                .map(|r| PermissionOut {
                    id: r.try_get("id").unwrap(),
                    name: r.try_get("name").unwrap(),
                    description: r.try_get("description").unwrap_or(None),
                })
                .collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn permissions_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("SELECT id, name, description FROM dora.permissions WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await
    {
        Ok(r) => {
            let out = PermissionOut {
                id: r.try_get("id").unwrap(),
                name: r.try_get("name").unwrap(),
                description: r.try_get("description").unwrap_or(None),
            };
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn permissions_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<PermissionUpdateReq>) -> impl IntoResponse {
    match sqlx::query("UPDATE dora.permissions SET name=COALESCE($2,name), description=COALESCE($3,description) WHERE id=$1 RETURNING id")
        .bind(id)
        .bind(req.name)
        .bind(req.description)
        .fetch_one(&st.pool)
        .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::OK, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn permissions_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM dora.permissions WHERE id=$1").bind(id).execute(&st.pool).await {
        Ok(res) => (StatusCode::OK, Json(serde_json::json!({"deleted": res.rows_affected()}))).into_response(),
        Err(e) => db_error(e),
    }
}

// ========== Role ↔ Permission assignments ==========
#[derive(Deserialize)]
struct RolePermissionAddReq { permission_id: Uuid }

async fn role_permissions_list(State(st): State<AppState>, Path(role_id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query(
        "SELECT p.id, p.name, p.description FROM dora.role_permissions rp JOIN dora.permissions p ON p.id = rp.permission_id WHERE rp.role_id = $1 ORDER BY p.name",
    )
    .bind(role_id)
    .fetch_all(&st.pool)
    .await
    {
        Ok(rows) => {
            let out: Vec<PermissionOut> = rows
                .into_iter()
                .map(|r| PermissionOut {
                    id: r.try_get("id").unwrap(),
                    name: r.try_get("name").unwrap(),
                    description: r.try_get("description").unwrap_or(None),
                })
                .collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn role_permissions_add(
    State(st): State<AppState>,
    Path(role_id): Path<Uuid>,
    Json(req): Json<RolePermissionAddReq>,
) -> impl IntoResponse {
    match sqlx::query(
        "INSERT INTO dora.role_permissions(role_id, permission_id) VALUES ($1,$2) ON CONFLICT DO NOTHING RETURNING role_id, permission_id",
    )
    .bind(role_id)
    .bind(req.permission_id)
    .fetch_optional(&st.pool)
    .await
    {
        Ok(Some(_)) => (StatusCode::CREATED, Json(serde_json::json!({"role_id": role_id, "permission_id": req.permission_id}))).into_response(),
        Ok(None) => (StatusCode::OK, Json(serde_json::json!({"role_id": role_id, "permission_id": req.permission_id, "existing": true}))).into_response(),
        Err(e) => db_error(e),
    }
}

async fn role_permissions_remove(State(st): State<AppState>, Path((role_id, pid)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM dora.role_permissions WHERE role_id=$1 AND permission_id=$2")
        .bind(role_id)
        .bind(pid)
        .execute(&st.pool)
        .await
    {
        Ok(res) => (StatusCode::OK, Json(serde_json::json!({"deleted": res.rows_affected()}))).into_response(),
        Err(e) => db_error(e),
    }
}

// ========== User ↔ Role assignments ==========
#[derive(Deserialize)]
struct UserRoleAddReq { role_id: Uuid }

async fn user_roles_list(State(st): State<AppState>, Path(user_id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query(
        "SELECT r.id, r.name, r.description FROM dora.user_roles ur JOIN dora.roles r ON r.id = ur.role_id WHERE ur.user_id = $1 ORDER BY r.name",
    )
    .bind(user_id)
    .fetch_all(&st.pool)
    .await
    {
        Ok(rows) => {
            let out: Vec<RoleOut> = rows
                .into_iter()
                .map(|r| RoleOut {
                    id: r.try_get("id").unwrap(),
                    name: r.try_get("name").unwrap(),
                    description: r.try_get("description").unwrap_or(None),
                })
                .collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn user_roles_add(
    State(st): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UserRoleAddReq>,
) -> impl IntoResponse {
    match sqlx::query(
        "INSERT INTO dora.user_roles(user_id, role_id) VALUES ($1,$2) ON CONFLICT DO NOTHING RETURNING user_id, role_id",
    )
    .bind(user_id)
    .bind(req.role_id)
    .fetch_optional(&st.pool)
    .await
    {
        Ok(Some(_)) => (StatusCode::CREATED, Json(serde_json::json!({"user_id": user_id, "role_id": req.role_id}))).into_response(),
        Ok(None) => (StatusCode::OK, Json(serde_json::json!({"user_id": user_id, "role_id": req.role_id, "existing": true}))).into_response(),
        Err(e) => db_error(e),
    }
}

async fn user_roles_remove(State(st): State<AppState>, Path((user_id, rid)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM dora.user_roles WHERE user_id=$1 AND role_id=$2")
        .bind(user_id)
        .bind(rid)
        .execute(&st.pool)
        .await
    {
        Ok(res) => (StatusCode::OK, Json(serde_json::json!({"deleted": res.rows_affected()}))).into_response(),
        Err(e) => db_error(e),
    }
}

async fn orgs_list(State(st): State<AppState>) -> impl IntoResponse {
    match sqlx::query("SELECT id, name, legal_entity_id, created_at, updated_at FROM dora.organizations ORDER BY name")
        .fetch_all(&st.pool)
        .await
    {
        Ok(rows) => {
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
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn orgs_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("SELECT id, name, legal_entity_id, created_at, updated_at FROM dora.organizations WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await
    {
        Ok(r) => {
            let out = OrgOut {
                id: r.try_get("id").unwrap(),
                name: r.try_get("name").unwrap(),
                legal_entity_id: r.try_get("legal_entity_id").unwrap_or(None),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            };
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn orgs_update(
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<OrgUpdateReq>,
) -> impl IntoResponse {
    match sqlx::query(
        "UPDATE dora.organizations SET name=COALESCE($2, name), legal_entity_id=COALESCE($3, legal_entity_id), updated_at=now() WHERE id=$1 RETURNING id",
    )
    .bind(id)
    .bind(req.name)
    .bind(req.legal_entity_id)
    .fetch_one(&st.pool)
    .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::OK, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn orgs_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM dora.organizations WHERE id=$1")
        .bind(id)
        .execute(&st.pool)
        .await
    {
        Ok(res) => (StatusCode::OK, Json(serde_json::json!({"deleted": res.rows_affected()}))).into_response(),
        Err(e) => db_error(e),
    }
}

// ========== Users ==========
#[derive(Deserialize)]
struct UserCreateReq {
    organization_id: Uuid,
    email: String,
    #[serde(default)]
    full_name: Option<String>,
}
#[derive(Deserialize)]
struct UserUpdateReq {
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    full_name: Option<String>,
    #[serde(default)]
    is_active: Option<bool>,
}

#[derive(Serialize)]
struct UserOut {
    id: Uuid,
    organization_id: Uuid,
    email: String,
    full_name: Option<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct UsersListParams {
    organization_id: Option<Uuid>,
}

async fn users_create(State(st): State<AppState>, Json(req): Json<UserCreateReq>) -> impl IntoResponse {
    match sqlx::query("INSERT INTO dora.users(organization_id, email, full_name) VALUES ($1, $2, $3) RETURNING id")
        .bind(req.organization_id)
        .bind(req.email)
        .bind(req.full_name)
        .fetch_one(&st.pool)
        .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::CREATED, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn users_list(State(st): State<AppState>, Query(p): Query<UsersListParams>) -> impl IntoResponse {
    let query = if let Some(org) = p.organization_id {
        sqlx::query("SELECT id, organization_id, email, full_name, is_active, created_at, updated_at FROM dora.users WHERE organization_id=$1 ORDER BY email").bind(org)
    } else {
        sqlx::query("SELECT id, organization_id, email, full_name, is_active, created_at, updated_at FROM dora.users ORDER BY email")
    };
    match query.fetch_all(&st.pool).await {
        Ok(rows) => {
            let out: Vec<UserOut> = rows
                .into_iter()
                .map(|r| UserOut {
                    id: r.try_get("id").unwrap(),
                    organization_id: r.try_get("organization_id").unwrap(),
                    email: r.try_get("email").unwrap(),
                    full_name: r.try_get("full_name").unwrap_or(None),
                    is_active: r.try_get("is_active").unwrap(),
                    created_at: r.try_get("created_at").unwrap(),
                    updated_at: r.try_get("updated_at").unwrap(),
                })
                .collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn users_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("SELECT id, organization_id, email, full_name, is_active, created_at, updated_at FROM dora.users WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await
    {
        Ok(r) => {
            let out = UserOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                email: r.try_get("email").unwrap(),
                full_name: r.try_get("full_name").unwrap_or(None),
                is_active: r.try_get("is_active").unwrap(),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            };
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn users_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UserUpdateReq>) -> impl IntoResponse {
    match sqlx::query(
        "UPDATE dora.users SET email=COALESCE($2, email), full_name=COALESCE($3, full_name), is_active=COALESCE($4, is_active), updated_at=now() WHERE id=$1 RETURNING id",
    )
    .bind(id)
    .bind(req.email)
    .bind(req.full_name)
    .bind(req.is_active)
    .fetch_one(&st.pool)
    .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::OK, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn users_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM dora.users WHERE id=$1").bind(id).execute(&st.pool).await {
        Ok(res) => (StatusCode::OK, Json(serde_json::json!({"deleted": res.rows_affected()}))).into_response(),
        Err(e) => db_error(e),
    }
}

// ========== Incidents ==========
#[derive(Deserialize)]
struct IncidentCreateReq {
    organization_id: Uuid,
    title: String,
    #[serde(rename = "type_")]
    type_: String,
    severity: String,
    detected_at: String, // RFC3339 | "now"
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    is_major: Option<bool>,
}
#[derive(Deserialize)]
struct IncidentUpdateReq {
    #[serde(default)]
    title: Option<String>,
    #[serde(default, rename = "type_")]
    type_: Option<String>,
    #[serde(default)]
    severity: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    is_major: Option<bool>,
}

#[derive(Serialize)]
struct IncidentOut {
    id: Uuid,
    organization_id: Uuid,
    title: String,
    description: Option<String>,
    #[serde(rename = "type_")]
    type_: String,
    severity: String,
    detected_at: DateTime<Utc>,
    status: String,
    is_major: bool,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct IncidentListParams {
    organization_id: Option<Uuid>,
}

async fn incidents_create(State(st): State<AppState>, Json(req): Json<IncidentCreateReq>) -> impl IntoResponse {
    let detected: DateTime<Utc> = if req.detected_at.to_lowercase() == "now" {
        Utc::now()
    } else {
        match DateTime::from_str(&req.detected_at) {
            Ok(dt) => dt,
            Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Invalid detected_at; use RFC3339 or 'now'"}))).into_response(),
        }
    };
    match sqlx::query(
        "INSERT INTO dora.incidents(organization_id, title, description, type, severity, detected_at, is_major) VALUES ($1,$2,$3,$4::dora.incident_type,$5::dora.incident_severity,$6,$7) RETURNING id",
    )
    .bind(req.organization_id)
    .bind(req.title)
    .bind(req.description)
    .bind(req.type_)
    .bind(req.severity)
    .bind(detected)
    .bind(req.is_major.unwrap_or(false))
    .fetch_one(&st.pool)
    .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::CREATED, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn incidents_list(State(st): State<AppState>, Query(p): Query<IncidentListParams>) -> impl IntoResponse {
    let query = if let Some(org) = p.organization_id {
        sqlx::query("SELECT id, organization_id, title, description, type::text AS type, severity::text AS severity, detected_at, status::text AS status, is_major, created_by, created_at, updated_at FROM dora.incidents WHERE organization_id=$1 ORDER BY detected_at DESC").bind(org)
    } else {
        sqlx::query("SELECT id, organization_id, title, description, type::text AS type, severity::text AS severity, detected_at, status::text AS status, is_major, created_by, created_at, updated_at FROM dora.incidents ORDER BY detected_at DESC")
    };
    match query.fetch_all(&st.pool).await {
        Ok(rows) => {
            let out: Vec<IncidentOut> = rows
                .into_iter()
                .map(|r| IncidentOut {
                    id: r.try_get("id").unwrap(),
                    organization_id: r.try_get("organization_id").unwrap(),
                    title: r.try_get("title").unwrap(),
                    description: r.try_get("description").unwrap_or(None),
                    type_: r.try_get::<String, _>("type").unwrap(),
                    severity: r.try_get::<String, _>("severity").unwrap(),
                    detected_at: r.try_get("detected_at").unwrap(),
                    status: r.try_get::<String, _>("status").unwrap(),
                    is_major: r.try_get("is_major").unwrap(),
                    created_by: r.try_get("created_by").unwrap_or(None),
                    created_at: r.try_get("created_at").unwrap(),
                    updated_at: r.try_get("updated_at").unwrap(),
                })
                .collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn incidents_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("SELECT id, organization_id, title, description, type::text AS type, severity::text AS severity, detected_at, status::text AS status, is_major, created_by, created_at, updated_at FROM dora.incidents WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await
    {
        Ok(r) => {
            let out = IncidentOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                title: r.try_get("title").unwrap(),
                description: r.try_get("description").unwrap_or(None),
                type_: r.try_get::<String, _>("type").unwrap(),
                severity: r.try_get::<String, _>("severity").unwrap(),
                detected_at: r.try_get("detected_at").unwrap(),
                status: r.try_get::<String, _>("status").unwrap(),
                is_major: r.try_get("is_major").unwrap(),
                created_by: r.try_get("created_by").unwrap_or(None),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            };
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn incidents_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<IncidentUpdateReq>) -> impl IntoResponse {
    match sqlx::query(
        "UPDATE dora.incidents SET title=COALESCE($2, title), description=COALESCE($3, description), type=COALESCE(($4)::dora.incident_type, type), severity=COALESCE(($5)::dora.incident_severity, severity), is_major=COALESCE($6, is_major), updated_at=now() WHERE id=$1 RETURNING id",
    )
    .bind(id)
    .bind(req.title)
    .bind(req.description)
    .bind(req.type_)
    .bind(req.severity)
    .bind(req.is_major)
    .fetch_one(&st.pool)
    .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::OK, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn incidents_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM dora.incidents WHERE id=$1").bind(id).execute(&st.pool).await {
        Ok(res) => (StatusCode::OK, Json(serde_json::json!({"deleted": res.rows_affected()}))).into_response(),
        Err(e) => db_error(e),
    }
}

// ========== Third Parties ==========
#[derive(Deserialize)]
struct TppCreateReq {
    organization_id: Uuid,
    name: String,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    criticality: Option<String>,
    #[serde(default)]
    is_important: Option<bool>,
}
#[derive(Deserialize)]
struct TppUpdateReq {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    criticality: Option<String>,
    #[serde(default)]
    is_important: Option<bool>,
}

#[derive(Serialize)]
struct TppOut {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    country: Option<String>,
    criticality: String,
    is_important: bool,
}

#[derive(Deserialize)]
struct TppListParams {
    organization_id: Option<Uuid>,
}

async fn tpps_create(State(st): State<AppState>, Json(req): Json<TppCreateReq>) -> impl IntoResponse {
    match sqlx::query(
        "INSERT INTO dora.third_parties(organization_id, name, country, criticality, is_important) VALUES ($1,$2,$3, COALESCE($4, 'non_critical')::dora.criticality_level, COALESCE($5,false)) RETURNING id",
    )
    .bind(req.organization_id)
    .bind(req.name)
    .bind(req.country)
    .bind(req.criticality)
    .bind(req.is_important)
    .fetch_one(&st.pool)
    .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::CREATED, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn tpps_list(State(st): State<AppState>, Query(p): Query<TppListParams>) -> impl IntoResponse {
    let query = if let Some(org) = p.organization_id {
        sqlx::query("SELECT id, organization_id, name, country, criticality::text AS criticality, is_important FROM dora.third_parties WHERE organization_id=$1 ORDER BY name").bind(org)
    } else {
        sqlx::query("SELECT id, organization_id, name, country, criticality::text AS criticality, is_important FROM dora.third_parties ORDER BY name")
    };
    match query.fetch_all(&st.pool).await {
        Ok(rows) => {
            let out: Vec<TppOut> = rows
                .into_iter()
                .map(|r| TppOut {
                    id: r.try_get("id").unwrap(),
                    organization_id: r.try_get("organization_id").unwrap(),
                    name: r.try_get("name").unwrap(),
                    country: r.try_get("country").unwrap_or(None),
                    criticality: r.try_get::<String, _>("criticality").unwrap(),
                    is_important: r.try_get("is_important").unwrap(),
                })
                .collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => {
            // Gracefully handle case where the third_parties table is missing (e.g., DB not initialized yet)
            if let sqlx::Error::Database(db) = &e {
                if db.code().as_deref() == Some("42P01") {
                    // undefined_table
                    info!(code=?db.code(), "third_parties table not found; returning empty list. Apply schema/dora_schema.sql to create it.");
                    let empty: Vec<TppOut> = Vec::new();
                    return (StatusCode::OK, Json(empty)).into_response();
                }
            }
            db_error(e)
        }
    }
}

async fn tpps_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("SELECT id, organization_id, name, country, criticality::text AS criticality, is_important FROM dora.third_parties WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await
    {
        Ok(r) => {
            let out = TppOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                name: r.try_get("name").unwrap(),
                country: r.try_get("country").unwrap_or(None),
                criticality: r.try_get::<String, _>("criticality").unwrap(),
                is_important: r.try_get("is_important").unwrap(),
            };
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn tpps_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<TppUpdateReq>) -> impl IntoResponse {
    match sqlx::query(
        "UPDATE dora.third_parties SET name=COALESCE($2, name), country=COALESCE($3, country), criticality=COALESCE(($4)::dora.criticality_level, criticality), is_important=COALESCE($5, is_important) WHERE id=$1 RETURNING id",
    )
    .bind(id)
    .bind(req.name)
    .bind(req.country)
    .bind(req.criticality)
    .bind(req.is_important)
    .fetch_one(&st.pool)
    .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::OK, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn tpps_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM dora.third_parties WHERE id=$1").bind(id).execute(&st.pool).await {
        Ok(res) => (StatusCode::OK, Json(serde_json::json!({"deleted": res.rows_affected()}))).into_response(),
        Err(e) => db_error(e),
    }
}

// ========== ICT Assets ==========
#[derive(Deserialize)]
struct AssetCreateReq {
    organization_id: Uuid,
    name: String,
    asset_type: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    criticality: Option<String>,
    #[serde(default)]
    is_important: Option<bool>,
}

#[derive(Deserialize)]
struct AssetUpdateReq {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    asset_type: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    criticality: Option<String>,
    #[serde(default)]
    is_important: Option<bool>,
}

#[derive(Serialize)]
struct AssetOut {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    asset_type: String,
    description: Option<String>,
    criticality: String,
    is_important: bool,
}

#[derive(Deserialize)]
struct AssetsListParams { organization_id: Option<Uuid> }

async fn assets_create(State(st): State<AppState>, Json(req): Json<AssetCreateReq>) -> impl IntoResponse {
    match sqlx::query(
        "INSERT INTO dora.ict_assets(organization_id, name, asset_type, description, criticality, is_important) VALUES ($1,$2,$3,$4, COALESCE($5,'non_critical')::dora.criticality_level, COALESCE($6,false)) RETURNING id",
    )
    .bind(req.organization_id)
    .bind(req.name)
    .bind(req.asset_type)
    .bind(req.description)
    .bind(req.criticality)
    .bind(req.is_important)
    .fetch_one(&st.pool)
    .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::CREATED, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn assets_list(State(st): State<AppState>, Query(p): Query<AssetsListParams>) -> impl IntoResponse {
    let query = if let Some(org) = p.organization_id {
        sqlx::query("SELECT id, organization_id, name, asset_type, description, criticality::text AS criticality, is_important FROM dora.ict_assets WHERE organization_id=$1 ORDER BY name").bind(org)
    } else {
        sqlx::query("SELECT id, organization_id, name, asset_type, description, criticality::text AS criticality, is_important FROM dora.ict_assets ORDER BY name")
    };
    match query.fetch_all(&st.pool).await {
        Ok(rows) => {
            let out: Vec<AssetOut> = rows
                .into_iter()
                .map(|r| AssetOut {
                    id: r.try_get("id").unwrap(),
                    organization_id: r.try_get("organization_id").unwrap(),
                    name: r.try_get("name").unwrap(),
                    asset_type: r.try_get("asset_type").unwrap(),
                    description: r.try_get("description").unwrap_or(None),
                    criticality: r.try_get::<String,_>("criticality").unwrap(),
                    is_important: r.try_get("is_important").unwrap(),
                })
                .collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => db_error(e),
    }
}

async fn assets_get(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("SELECT id, organization_id, name, asset_type, description, criticality::text AS criticality, is_important FROM dora.ict_assets WHERE id=$1")
        .bind(id)
        .fetch_one(&st.pool)
        .await
    {
        Ok(r) => {
            let out = AssetOut {
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                name: r.try_get("name").unwrap(),
                asset_type: r.try_get("asset_type").unwrap(),
                description: r.try_get("description").unwrap_or(None),
                criticality: r.try_get::<String,_>("criticality").unwrap(),
                is_important: r.try_get("is_important").unwrap(),
            };
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn assets_update(State(st): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<AssetUpdateReq>) -> impl IntoResponse {
    match sqlx::query(
        "UPDATE dora.ict_assets SET name=COALESCE($2,name), asset_type=COALESCE($3,asset_type), description=COALESCE($4,description), criticality=COALESCE(($5)::dora.criticality_level, criticality), is_important=COALESCE($6, is_important), updated_at=now() WHERE id=$1 RETURNING id",
    )
    .bind(id)
    .bind(req.name)
    .bind(req.asset_type)
    .bind(req.description)
    .bind(req.criticality)
    .bind(req.is_important)
    .fetch_one(&st.pool)
    .await
    {
        Ok(row) => {
            let id: Uuid = row.try_get("id").unwrap();
            (StatusCode::OK, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => db_error(e),
    }
}

async fn assets_delete(State(st): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM dora.ict_assets WHERE id=$1").bind(id).execute(&st.pool).await {
        Ok(res) => (StatusCode::OK, Json(serde_json::json!({"deleted": res.rows_affected()}))).into_response(),
        Err(e) => db_error(e),
    }
}

// ===== error mapping helper =====
fn db_error(e: sqlx::Error) -> axum::response::Response {
    match &e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND.into_response(),
        sqlx::Error::Database(db)
            if db.code().as_deref() == Some("23505") =>
        {
            // unique_violation
            (StatusCode::CONFLICT, Json(serde_json::json!({"error": db.message()}))).into_response()
        }
        sqlx::Error::Database(db)
            if db.code().as_deref() == Some("23503") =>
        {
            // foreign_key_violation (e.g., invalid organization_id)
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": db.message()}))).into_response()
        }
        sqlx::Error::Database(db)
            if db.code().as_deref() == Some("42P01") =>
        {
            // undefined_table (likely schema not applied yet)
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": format!("Database table missing: {}. Apply schema/dora_schema.sql.", db.message())
            }))).into_response()
        }
        sqlx::Error::Database(db)
            if {
                let code_opt = db.code();
                matches!(code_opt.as_deref(), Some("22P02" | "42804"))
            } =>
        {
            // invalid_text_representation or datatype_mismatch (enum casts)
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": db.message()}))).into_response()
        }
        other => {
            error!(?other, "DB error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error":"internal error"}))).into_response()
        }
    }
}

async fn user_handlers(pool: &PgPool, cmd: UserCmd) -> Result<()> {
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
                #[derive(Serialize)]
                struct UserOut { id: Uuid, organization_id: Uuid, email: String, full_name: Option<String>, is_active: bool, created_at: DateTime<Utc>, updated_at: DateTime<Utc> }
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
                #[derive(Serialize)]
                struct UserOut { id: Uuid, organization_id: Uuid, email: String, full_name: Option<String>, is_active: bool, created_at: DateTime<Utc>, updated_at: DateTime<Utc> }
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
            #[derive(Serialize)]
            struct UserOut { id: Uuid, organization_id: Uuid, email: String, full_name: Option<String>, is_active: bool, created_at: DateTime<Utc>, updated_at: DateTime<Utc> }
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

async fn incident_handlers(pool: &PgPool, cmd: IncidentCmd) -> Result<()> {
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
            let rows = if let Some(org) = organization_id {
                sqlx::query("SELECT id, organization_id, title, description, type::text AS type, severity::text AS severity, detected_at, status::text AS status, is_major, created_by, created_at, updated_at FROM dora.incidents WHERE organization_id=$1 ORDER BY detected_at DESC")
                    .bind(org)
                    .fetch_all(pool)
                    .await?
            } else {
                sqlx::query("SELECT id, organization_id, title, description, type::text AS type, severity::text AS severity, detected_at, status::text AS status, is_major, created_by, created_at, updated_at FROM dora.incidents ORDER BY detected_at DESC")
                    .fetch_all(pool)
                    .await?
            };
            #[derive(Serialize)]
            struct IncidentOut { id: Uuid, organization_id: Uuid, title: String, description: Option<String>, type_: String, severity: String, detected_at: DateTime<Utc>, status: String, is_major: bool, created_by: Option<Uuid>, created_at: DateTime<Utc>, updated_at: DateTime<Utc> }
            let out: Vec<IncidentOut> = rows.into_iter().map(|r| IncidentOut{
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                title: r.try_get("title").unwrap(),
                description: r.try_get("description").unwrap_or(None),
                type_: r.try_get::<String, _>("type").unwrap(),
                severity: r.try_get::<String, _>("severity").unwrap(),
                detected_at: r.try_get("detected_at").unwrap(),
                status: r.try_get::<String, _>("status").unwrap(),
                is_major: r.try_get("is_major").unwrap(),
                created_by: r.try_get("created_by").unwrap_or(None),
                created_at: r.try_get("created_at").unwrap(),
                updated_at: r.try_get("updated_at").unwrap(),
            }).collect();
            println!("{}", serde_json::to_string(&out)?);
        }
        IncidentCmd::Get { id } => {
            let r = sqlx::query("SELECT id, organization_id, title, description, type::text AS type, severity::text AS severity, detected_at, status::text AS status, is_major, created_by, created_at, updated_at FROM dora.incidents WHERE id=$1")
                .bind(id)
                .fetch_one(pool)
                .await?;
            #[derive(Serialize)]
            struct IncidentOut { id: Uuid, organization_id: Uuid, title: String, description: Option<String>, type_: String, severity: String, detected_at: DateTime<Utc>, status: String, is_major: bool, created_by: Option<Uuid>, created_at: DateTime<Utc>, updated_at: DateTime<Utc> }
            let out = IncidentOut{
                id: r.try_get("id")?,
                organization_id: r.try_get("organization_id")?,
                title: r.try_get("title")?,
                description: r.try_get::<Option<String>, _>("description")?,
                type_: r.try_get::<String, _>("type")?,
                severity: r.try_get::<String, _>("severity")?,
                detected_at: r.try_get("detected_at")?,
                status: r.try_get::<String, _>("status")?,
                is_major: r.try_get("is_major")?,
                created_by: r.try_get::<Option<Uuid>, _>("created_by")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
            };
            println!("{}", serde_json::to_string(&out)?);
        }
        IncidentCmd::Update { id, title, type_, severity, description, is_major } => {
            let row = sqlx::query(
                "UPDATE dora.incidents SET title=COALESCE($2, title), description=COALESCE($3, description), type=COALESCE(($4)::dora.incident_type, type), severity=COALESCE(($5)::dora.incident_severity, severity), is_major=COALESCE($6, is_major), updated_at=now() WHERE id=$1 RETURNING id",
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

async fn tpp_handlers(pool: &PgPool, cmd: TppCmd) -> Result<()> {
    match cmd {
        TppCmd::Create { organization_id, name, country, criticality, is_important } => {
            let row = sqlx::query(
                "INSERT INTO dora.third_parties(organization_id, name, country, criticality, is_important) VALUES ($1,$2,$3, COALESCE($4, 'non_critical')::dora.criticality_level, COALESCE($5,false)) RETURNING id",
            )
            .bind(organization_id)
            .bind(name)
            .bind(country)
            .bind(criticality)
            .bind(is_important)
            .fetch_one(pool)
            .await?;
            let id: Uuid = row.try_get("id")?;
            println!("{}", serde_json::to_string(&IdOut { id })?);
        }
        TppCmd::List { organization_id } => {
            let rows = if let Some(org) = organization_id {
                sqlx::query("SELECT id, organization_id, name, country, criticality::text AS criticality, is_important FROM dora.third_parties WHERE organization_id=$1 ORDER BY name")
                    .bind(org)
                    .fetch_all(pool)
                    .await?
            } else {
                sqlx::query("SELECT id, organization_id, name, country, criticality::text AS criticality, is_important FROM dora.third_parties ORDER BY name")
                    .fetch_all(pool)
                    .await?
            };
            #[derive(Serialize)]
            struct TppOut { id: Uuid, organization_id: Uuid, name: String, country: Option<String>, criticality: String, is_important: bool }
            let out: Vec<TppOut> = rows.into_iter().map(|r| TppOut{
                id: r.try_get("id").unwrap(),
                organization_id: r.try_get("organization_id").unwrap(),
                name: r.try_get("name").unwrap(),
                country: r.try_get("country").unwrap_or(None),
                criticality: r.try_get::<String, _>("criticality").unwrap(),
                is_important: r.try_get("is_important").unwrap(),
            }).collect();
            println!("{}", serde_json::to_string(&out)?);
        }
        TppCmd::Get { id } => {
            let r = sqlx::query("SELECT id, organization_id, name, country, criticality::text AS criticality, is_important FROM dora.third_parties WHERE id=$1")
                .bind(id)
                .fetch_one(pool)
                .await?;
            #[derive(Serialize)]
            struct TppOut { id: Uuid, organization_id: Uuid, name: String, country: Option<String>, criticality: String, is_important: bool }
            let out = TppOut{
                id: r.try_get("id")?,
                organization_id: r.try_get("organization_id")?,
                name: r.try_get("name")?,
                country: r.try_get::<Option<String>, _>("country")?,
                criticality: r.try_get::<String, _>("criticality")?,
                is_important: r.try_get("is_important")?,
            };
            println!("{}", serde_json::to_string(&out)?);
        }
        TppCmd::Update { id, name, country, criticality, is_important } => {
            let row = sqlx::query(
                "UPDATE dora.third_parties SET name=COALESCE($2, name), country=COALESCE($3, country), criticality=COALESCE(($4)::dora.criticality_level, criticality), is_important=COALESCE($5, is_important) WHERE id=$1 RETURNING id",
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
            let res = sqlx::query("DELETE FROM dora.third_parties WHERE id=$1")
                .bind(id)
                .execute(pool)
                .await?;
            println!("{}", serde_json::to_string(&serde_json::json!({"deleted": res.rows_affected()}))?);
        }
    }
    Ok(())
}

// ==============================
// Tests (in-process REST server)
// ==============================
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::{Request, StatusCode}};
    use http_body_util::BodyExt; // for collect
    use tower::util::ServiceExt; // for oneshot

    async fn test_app() -> Option<Router> {
        // Require a real database; skip tests if DATABASE_URL is not set.
        let url = match std::env::var("DATABASE_URL") {
            Ok(u) => u,
            Err(_) => {
                eprintln!("Skipping REST tests: DATABASE_URL not set");
                return None;
            }
        };
        let pool = match db_pool(&url).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping REST tests: cannot connect to DB: {e}");
                return None;
            }
        };
        Some(build_router(pool))
    }

    async fn resp_json<T: serde::de::DeserializeOwned>(res: axum::response::Response) -> T {
        let bytes = BodyExt::collect(res.into_body()).await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn healthz_works() {
        if let Some(mut app) = test_app().await {
            let res = app
                .clone()
                .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
            let v: serde_json::Value = resp_json(res).await;
            assert_eq!(v["status"], "ok");
        }
    }

    #[tokio::test]
    async fn orgs_crud_and_users() {
        if let Some(mut app) = test_app().await {
            // Create organization
            let name = format!("Test Org {}", Uuid::new_v4());
            let body = serde_json::json!({"name": name, "legal_entity_id": "LEI123"}).to_string();
            let res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/orgs")
                        .header("content-type", "application/json")
                        .body(Body::from(body))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::CREATED);
            let v: serde_json::Value = resp_json(res).await;
            let org_id = Uuid::parse_str(v["id"].as_str().unwrap()).unwrap();

            // Get organization
            let res = app
                .clone()
                .oneshot(Request::builder().uri(format!("/orgs/{org_id}")).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
            let got: serde_json::Value = resp_json(res).await;
            assert_eq!(got["id"].as_str().unwrap(), org_id.to_string());

            // List organizations (ensure appears)
            let res = app
                .clone()
                .oneshot(Request::builder().uri("/orgs").body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
            let list: Vec<serde_json::Value> = resp_json(res).await;
            assert!(list.iter().any(|o| o["id"] == org_id.to_string()));

            // Update organization name
            let new_name = format!("{} - Updated", name);
            let body = serde_json::json!({"name": new_name}).to_string();
            let res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("PATCH")
                        .uri(format!("/orgs/{org_id}"))
                        .header("content-type", "application/json")
                        .body(Body::from(body))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);

            // Create a user under the org
            let email = format!("user+{}@example.com", Uuid::new_v4());
            let body = serde_json::json!({"organization_id": org_id, "email": email}).to_string();
            let res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/users")
                        .header("content-type", "application/json")
                        .body(Body::from(body))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::CREATED);
            let uv: serde_json::Value = resp_json(res).await;
            let _user_id = Uuid::parse_str(uv["id"].as_str().unwrap()).unwrap();

            // List users by org and ensure the user exists
            let res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/users?organization_id={org_id}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
            let users: Vec<serde_json::Value> = resp_json(res).await;
            assert!(users.iter().any(|u| u["email"] == email));

            // Delete organization (cascades to users)
            let res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("DELETE")
                        .uri(format!("/orgs/{org_id}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);

            // Ensure org is gone
            let res = app
                .clone()
                .oneshot(Request::builder().uri(format!("/orgs/{org_id}")).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }
    }
}
